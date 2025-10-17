#![cfg_attr(target_arch = "wasm32", no_main)]

mod contract_domain;
mod state;

use linera_sdk::{
    abi::WithContractAbi,
    linera_base_types::{Amount, ChainId},
    views::{RootView, View},
    Contract, ContractRuntime,
};
use state::Leaderboard;
use std::str::FromStr;

use self::state::Game2048;
use contract_domain::events::emitters::EventEmitter;
use contract_domain::{
    ContractHelpers, EventReader, LeaderboardOperationHandler, PlayerOperationHandler,
    ShardOperationHandler, StreamProcessor, SubscriptionManager, TournamentOperationHandler,
};
use game2048::{GameEvent, Message, Operation, RegistrationCheck};

pub struct Game2048Contract {
    state: Game2048,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(Game2048Contract);

impl WithContractAbi for Game2048Contract {
    type Abi = game2048::Game2048Abi;
}

impl Contract for Game2048Contract {
    type Message = Message;
    type Parameters = ();
    type InstantiationArgument = u32;
    type EventValue = GameEvent;

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = Game2048::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        Game2048Contract { state, runtime }
    }

    async fn instantiate(&mut self, _seed: Self::InstantiationArgument) {
        self.runtime.application_parameters();

        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        leaderboard.leaderboard_id.set("".to_string());
        leaderboard
            .chain_id
            .set(self.runtime.chain_id().to_string());
        leaderboard.host.set("".to_string());
        leaderboard.start_time.set(0);
        leaderboard.end_time.set(0);
        leaderboard.name.set("".to_string());
        leaderboard.description.set("".to_string());
        leaderboard.total_boards.set(0);
        leaderboard.total_players.set(0);

        leaderboard.admin_base_triggerer_count.set(5); // Default to 5 triggerers
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        use crate::contract_domain::OperationDispatcher;

        // 🚀 TIER 6 EMERGENCY MODE: Check and activate if needed (works without events!)
        let current_time = self.runtime.system_time().micros();
        let last_update = *self.state.triggerer_list_timestamp.get();
        let threshold = *self.state.trigger_threshold_config.get();

        // Only check tier 6 if we have a valid threshold and triggerer list
        if threshold > 0 && last_update > 0 {
            let time_since_update = current_time.saturating_sub(last_update);

            // Calculate tier (1-5) - same logic as event processor
            let tier = if time_since_update > 0 {
                std::cmp::min(5, (time_since_update / threshold) + 1)
            } else {
                1
            };

            // Tier 6 cutpoint: 5 intervals after last update
            let tier6_cutpoint = last_update + (5 * threshold);
            let should_enter_tier6 = tier == 5 && current_time >= tier6_cutpoint;

            // Enter tier 6 mode if conditions met
            if should_enter_tier6 && !*self.state.is_in_tier6.get() {
                self.state.is_in_tier6.set(true);
                self.state.tier6_start_time.set(current_time);
                self.state.operations_since_tier6.set(0);
            }

            // If in tier 6, increment counter and check if should trigger
            if *self.state.is_in_tier6.get() {
                let op_count = *self.state.operations_since_tier6.get();
                self.state.operations_since_tier6.set(op_count + 1);

                // Simple tier 6 rule: 5 operations = can trigger
                if op_count + 1 >= 5 {
                    // Check last trigger time to prevent spam (2 second minimum)
                    let last_trigger_sent = *self.state.last_trigger_sent.get();
                    let time_since_last_trigger = current_time.saturating_sub(last_trigger_sent);
                    let min_threshold = std::cmp::max(threshold, 2_000_000); // At least 2 seconds

                    if time_since_last_trigger > min_threshold {
                        // Get tournament ID and leaderboard chain ID
                        let mut tournament_id = String::new();
                        let mut leaderboard_chain_id_str = String::new();

                        // Try to get first cached tournament
                        self.state
                            .tournaments_cache_json
                            .for_each_index_value_while(|key, value| {
                                tournament_id = key;
                                // Parse tournament JSON to get leaderboard chain
                                if let Ok(tournament) =
                                    serde_json::from_str::<game2048::TournamentInfo>(&value)
                                {
                                    leaderboard_chain_id_str = tournament.tournament_id;
                                }
                                Ok(false) // Stop after first
                            })
                            .await
                            .unwrap();

                        // Send trigger if we have valid data
                        if !tournament_id.is_empty() && !leaderboard_chain_id_str.is_empty() {
                            if let Ok(leaderboard_chain_id) =
                                ChainId::from_str(&leaderboard_chain_id_str)
                            {
                                let my_chain_id = self.runtime.chain_id().to_string();

                                self.runtime
                                    .prepare_message(game2048::Message::TriggerUpdate {
                                        triggerer_chain_id: my_chain_id,
                                        tournament_id: tournament_id.clone(),
                                        timestamp: current_time,
                                    })
                                    .send_to(leaderboard_chain_id);

                                // Update last trigger sent and reset operation counter
                                self.state.last_trigger_sent.set(current_time);
                                self.state.operations_since_tier6.set(0);
                            }
                        }
                    }
                }
            }
        }

        // 🚀 NORMAL TIER 1-5 TRIGGER LOGIC: Check and send triggers during block production
        // This replaces the event-based trigger logic to ensure latest state is used
        self.check_and_send_trigger_if_needed_in_block_production().await;

        OperationDispatcher::dispatch(self, operation).await;
        ContractHelpers::update_balance(self);
    }
    async fn execute_message(&mut self, message: Self::Message) {
        use crate::contract_domain::MessageDispatcher;

        MessageDispatcher::dispatch(self, message).await;
        ContractHelpers::update_balance(self);
    }

    async fn process_streams(&mut self, updates: Vec<linera_sdk::linera_base_types::StreamUpdate>) {
        StreamProcessor::process_updates(self, updates).await;
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl Game2048Contract {
    // ========================================
    // UTILITY METHODS
    // ========================================

    fn is_main_chain(&mut self) -> bool {
        ContractHelpers::is_main_chain(self)
    }

    // ========================================
    // MESSAGE UTILITY METHODS
    // (Direct message preparation & sending)
    // ========================================

    fn register_player(&mut self, chain_id: ChainId, player: &str, password_hash: &str) {
        self.runtime
            .prepare_message(Message::RegisterPlayer {
                username: player.to_string(),
                password_hash: password_hash.to_string(),
            })
            .send_to(chain_id);
    }

    async fn upsert_leaderboard(
        &mut self,
        chain_id: ChainId,
        name: &str,
        description: &str,
        host: &str,
        start_time: u64,
        end_time: u64,
        send_to: Option<ChainId>,
    ) {
        self.runtime
            .prepare_message(Message::CreateLeaderboard {
                leaderboard_id: chain_id.to_string(),
                name: name.to_string(),
                description: Some(description.to_string()),
                chain_id: chain_id.to_string(),
                host: host.to_string(),
                start_time,
                end_time,
                shard_ids: vec![], // Default empty, filled by operations handler
                base_triggerer_count: 5, // Default value
                total_shard_count: 1, // Default value
            })
            .send_to(send_to.unwrap_or(chain_id));
    }

    fn transfer(&mut self, destination: ChainId, amount: Amount) {
        ContractHelpers::transfer(self, destination, amount);
    }

    fn auto_faucet(&mut self, faucet_amount: Option<u128>) {
        ContractHelpers::auto_faucet(self, faucet_amount);
    }

    // ========================================
    // HANDLER-DELEGATED METHODS
    // (Business logic via handlers)
    // ========================================

    async fn is_leaderboard_active(&mut self, timestamp: u64) -> &mut Leaderboard {
        LeaderboardOperationHandler::is_leaderboard_active(self, timestamp).await
    }

    async fn update_shard_score(
        &mut self,
        player: &str,
        board_id: String,
        score: u64,
        timestamp: u64,
        player_chain_id: String,
        boards_in_tournament: u32,
        leaderboard_id: String,
        game_status: game2048::GameStatus,
        highest_tile: u64,
    ) {
        ShardOperationHandler::update_shard_score(
            self,
            player,
            board_id,
            score,
            timestamp,
            player_chain_id,
            boards_in_tournament,
            leaderboard_id,
            game_status,
            highest_tile,
        )
        .await;
    }

    async fn check_player_registered(
        &mut self,
        player_username: &str,
        check: RegistrationCheck,
    ) -> String {
        PlayerOperationHandler::check_player_registered(self, player_username, check).await
    }

    async fn validate_player_password(
        &mut self,
        player_username: &str,
        provided_password_hash: &str,
    ) {
        PlayerOperationHandler::validate_player_password(
            self,
            player_username,
            provided_password_hash,
        )
        .await;
    }

    /// Reads a player score event from another chain
    pub fn read_player_score_event_from_chain(
        &mut self,
        chain_id: ChainId,
        event_index: u32,
    ) -> Option<GameEvent> {
        EventReader::read_player_score_event_from_chain(self, chain_id, event_index)
    }

    /// Reads a shard score event from another chain
    pub fn read_shard_score_event_from_chain(
        &mut self,
        chain_id: ChainId,
        event_index: u32,
    ) -> Option<GameEvent> {
        EventReader::read_shard_score_event_from_chain(self, chain_id, event_index)
    }

    /// Subscribes to player score events from another chain
    pub fn subscribe_to_player_score_events(&mut self, chain_id: ChainId) {
        SubscriptionManager::subscribe_to_player_score_events(self, chain_id);
    }

    /// Subscribes to shard score events from another chain
    pub fn subscribe_to_shard_score_events(&mut self, chain_id: ChainId) {
        SubscriptionManager::subscribe_to_shard_score_events(self, chain_id);
    }

    pub fn subscribe_to_leaderboard_update_events(&mut self, chain_id: ChainId) {
        SubscriptionManager::subscribe_to_leaderboard_update_events(self, chain_id);
    }

    /// Validates tournament exists and is active
    pub async fn validate_tournament(&mut self, tournament_id: &str) -> bool {
        TournamentOperationHandler::validate_tournament(self, tournament_id).await
    }

    /// Shard chain functionality - aggregates scores with smart player activity tracking
    pub async fn aggregate_scores_from_player_chains(&mut self, player_chain_ids: Vec<ChainId>) {
        ShardOperationHandler::aggregate_scores_from_player_chains(self, player_chain_ids).await;
    }

    /// Emit current active tournaments (for leaderboard chains)
    pub async fn emit_active_tournaments(&mut self) {
        LeaderboardOperationHandler::emit_active_tournaments(self).await;
    }

    /// Register player with shard and update workload tracking
    pub async fn register_player_with_shard(
        &mut self,
        player_chain_id: String,
        tournament_id: String,
        player_name: String,
    ) {
        ShardOperationHandler::register_player_with_shard(
            self,
            player_chain_id,
            tournament_id,
            player_name,
        )
        .await;
    }

    /// Update game count when games are created/ended
    pub async fn track_game_activity(&mut self) {
        ShardOperationHandler::track_game_activity(self).await;
    }

    /// Select optimal shard for a tournament based on workload
    pub async fn select_optimal_shard(&mut self, tournament_id: &str, player_id: &str) -> String {
        TournamentOperationHandler::select_optimal_shard(self, tournament_id, player_id).await
    }

    /// Dynamic Triggerer Management - Updates based on actual scores
    pub async fn update_triggerer_pool(&mut self) {
        LeaderboardOperationHandler::update_triggerer_pool(self).await;
    }

    /// Check if a chain is authorized to trigger
    pub async fn is_authorized_triggerer(&mut self, requester_chain_id: &str) -> bool {
        LeaderboardOperationHandler::is_authorized_triggerer(self, requester_chain_id).await
    }

    /// Handle aggregation trigger request with robust cooldown
    pub async fn handle_aggregation_trigger_request(
        &mut self,
        requester_chain_id: &str,
        timestamp: u64,
    ) -> Result<(), String> {
        LeaderboardOperationHandler::handle_aggregation_trigger_request(
            self,
            requester_chain_id,
            timestamp,
        )
        .await
    }

    /// Emit game creation event helper
    pub async fn emit_game_creation_event(
        &mut self,
        board_id: &str,
        player: &str,
        tournament_id: &str,
        timestamp: u64,
        boards_in_tournament: u32,
    ) {
        EventEmitter::emit_game_creation_event(
            self,
            board_id,
            player,
            tournament_id,
            timestamp,
            boards_in_tournament,
        )
        .await;
    }

    // ========================================
    // TOURNAMENT CACHE MANAGEMENT (STREAMING)
    // ========================================

    /// Get cached tournament info (avoids cross-chain reads)
    pub async fn get_cached_tournament(
        &mut self,
        tournament_id: &str,
    ) -> Option<game2048::TournamentInfo> {
        if let Some(tournament_json) = self
            .state
            .tournaments_cache_json
            .get(tournament_id)
            .await
            .unwrap()
        {
            serde_json::from_str(&tournament_json).ok()
        } else {
            None
        }
    }

    /// List all cached tournaments
    pub async fn list_cached_tournaments(&mut self) -> Vec<game2048::TournamentInfo> {
        let mut tournaments = Vec::new();

        self.state
            .tournaments_cache_json
            .for_each_index_value_while(|_key, tournament_json| {
                if let Ok(tournament) =
                    serde_json::from_str::<game2048::TournamentInfo>(&tournament_json)
                {
                    tournaments.push(tournament);
                }
                Ok(true) // Continue iteration
            })
            .await
            .unwrap();

        tournaments
    }

    /// Get count of cached tournaments
    pub async fn get_cached_tournament_count(&mut self) -> u64 {
        self.state.tournaments_cache_json.count().await.unwrap() as u64
    }

    /// Read active tournaments event from chain using EventReader
    pub fn read_active_tournaments_event_from_chain(
        &mut self,
        chain_id: linera_sdk::linera_base_types::ChainId,
        event_index: u32,
    ) -> Option<game2048::GameEvent> {
        EventReader::read_active_tournaments_event_from_chain(self, chain_id, event_index)
    }

    /// Read leaderboard update event from chain using EventReader
    pub fn read_leaderboard_update_event_from_chain(
        &mut self,
        chain_id: linera_sdk::linera_base_types::ChainId,
        event_index: u32,
    ) -> Option<game2048::GameEvent> {
        EventReader::read_leaderboard_update_event_from_chain(self, chain_id, event_index)
    }

    /// 🚀 BLOCK PRODUCTION TRIGGER LOGIC: Check and send triggers during block production
    /// This replaces event-based triggering to ensure latest state is always used
    async fn check_and_send_trigger_if_needed_in_block_production(&mut self) {
        let current_time = self.runtime.system_time().micros();
        let my_chain_id = self.runtime.chain_id().to_string();

        // Only player chains should send triggers (check if we have triggerer list)
        let triggerer_count = match self.state.triggerer_list.read_front(1).await {
            Ok(items) => items.len(),
            Err(_) => 0,
        };

        if triggerer_count == 0 {
            // This is not a player chain (no triggerer list), skip trigger logic
            return;
        }

        // Get configuration with safe default
        let threshold = match *self.state.trigger_threshold_config.get() {
            0 => {
                // Initialize with safe default if never set (5 seconds for stress test)
                let default_threshold = 5_000_000; // 5 seconds in microseconds
                self.state.trigger_threshold_config.set(default_threshold);
                default_threshold
            }
            value => value,
        };
        
        let last_update_time = *self.state.triggerer_list_timestamp.get();
        let last_trigger_sent = *self.state.last_trigger_sent.get();
        let total_players = *self.state.total_registered_players.get();

        // Check if enough time has passed since last update
        let time_since_update = current_time.saturating_sub(last_update_time);
        let time_since_last_trigger = current_time.saturating_sub(last_trigger_sent);

        // Mathematical tier calculation with overflow protection (tiers 1-5 only)
        let tier = if threshold > 0 && time_since_update > 0 {
            std::cmp::min(5, (time_since_update / threshold) + 1)
        } else if threshold == 0 {
            1 // Minimal tier to prevent infinite triggering
        } else {
            1 // No need for escalated triggering if time_since_update is 0
        };

        // Calculate how many players should be actively triggering
        let admin_base_count = *self.state.admin_base_triggerer_count.get();
        let base_triggerer_count = if admin_base_count > 0 {
            admin_base_count
        } else {
            5 // Default to 5 for stress testing
        };
        let active_triggerer_count =
            std::cmp::min(total_players, base_triggerer_count * tier as u32);

        // Find my position in the triggerer list
        let mut my_position: Option<u32> = None;

        match self
            .state
            .triggerer_list
            .read_front(active_triggerer_count as usize)
            .await
        {
            Ok(triggerers) => {
                for (i, triggerer_id) in triggerers.iter().enumerate() {
                    if triggerer_id == &my_chain_id {
                        my_position = Some(i as u32);
                        break;
                    }
                }
            }
            Err(_) => return,
        }

        // Check if I'm an active triggerer (normal tier 1-5 only)
        let am_i_active_triggerer = match my_position {
            Some(pos) => pos < active_triggerer_count,
            None => false,
        };

        if !am_i_active_triggerer {
            return;
        }

        // Only trigger if enough time has passed since our last trigger
        let min_threshold = std::cmp::max(threshold, 2_000_000); // At least 2 seconds
        let should_trigger = time_since_last_trigger > min_threshold;

        if should_trigger {
            // Get tournament ID from the first cached tournament
            let mut tournament_id = String::new();
            let mut leaderboard_chain_id_str = String::new();

            self.state
                .tournaments_cache_json
                .for_each_index_value_while(|key, value| {
                    tournament_id = key;
                    // Parse tournament JSON to get leaderboard chain
                    if let Ok(tournament) =
                        serde_json::from_str::<game2048::TournamentInfo>(&value)
                    {
                        leaderboard_chain_id_str = tournament.tournament_id;
                    }
                    Ok(false) // Stop after first
                })
                .await
                .unwrap();

            // Send trigger if we have valid data
            if !tournament_id.is_empty() && !leaderboard_chain_id_str.is_empty() {
                if let Ok(leaderboard_chain_id) = ChainId::from_str(&leaderboard_chain_id_str) {
                    // Send trigger message to leaderboard
                    self.runtime
                        .prepare_message(game2048::Message::TriggerUpdate {
                            triggerer_chain_id: my_chain_id,
                            tournament_id: tournament_id.clone(),
                            timestamp: current_time,
                        })
                        .send_to(leaderboard_chain_id);

                    // Update last trigger sent time
                    self.state.last_trigger_sent.set(current_time);
                }
            }
        }
    }
}
