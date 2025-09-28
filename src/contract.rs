#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;
mod contract_domain;

use linera_sdk::{
    linera_base_types::{Account, AccountOwner, Amount, ChainId},
    abi::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};
use std::str::FromStr;
use state::Leaderboard;

use self::state::Game2048;
use game2048::{GameEvent, Message, Operation, RegistrationCheck};
use contract_domain::{
    PlayerOperationHandler, LeaderboardOperationHandler,
    ShardOperationHandler, TournamentOperationHandler, StreamProcessingHandler
};

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
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        use crate::contract_domain::OperationDispatcher;
        
        OperationDispatcher::dispatch(self, operation).await;

        self.state
            .balance
            .set(self.runtime.chain_balance().to_string());
    }
    async fn execute_message(&mut self, message: Self::Message) {
        use crate::contract_domain::MessageDispatcher;
        
        MessageDispatcher::dispatch(self, message).await;

        self.state
            .balance
            .set(self.runtime.chain_balance().to_string());
    }

    async fn process_streams(&mut self, updates: Vec<linera_sdk::linera_base_types::StreamUpdate>) {
        log::info!("🔥 PROCESS_STREAMS: Received {} stream updates", updates.len());
        let mut processed_player_updates = false;

        for (update_idx, update) in updates.iter().enumerate() {
            // Determine which stream we're processing based on stream name
            let stream_name_bytes = &update.stream_id.stream_name.0;
            let stream_name = String::from_utf8_lossy(stream_name_bytes);
            
            log::info!("🔥 PROCESS_STREAMS: Update #{} - Chain: {}, Stream: '{}', Events: {}..{} (count: {})", 
                update_idx + 1,
                update.chain_id,
                stream_name,
                update.previous_index,
                update.next_index,
                update.next_index - update.previous_index
            );

            // Process all new events in this stream update
            let event_count = update.next_index - update.previous_index;
            if event_count == 0 {
                log::info!("🔥 PROCESS_STREAMS: ⏭️ No new events in update #{} (next_index {} <= previous_index {})", 
                    update_idx + 1, update.next_index, update.previous_index);
                continue;
            }

            for event_index in update.previous_index..update.next_index {
                log::info!("🔥 PROCESS_STREAMS: Processing event index {} from stream '{}'", event_index, stream_name);
                
                match stream_name.as_ref() {
                    "player_score_update" => {
                        log::info!("🔥 PROCESS_STREAMS: 🎯 Processing player_score_update event at index {}", event_index);
                        
                        // Read the player score event data and update shard state
                        if let Some(event) = self.read_player_score_event_from_chain(update.chain_id, event_index) {
                            if let game2048::GameEvent::PlayerScoreUpdate {
                                player,
                                score,
                                board_id,
                                timestamp,
                                game_status,
                                highest_tile,
                                leaderboard_id,
                                boards_in_tournament,
                                ..
                            } = event {
                                // Update shard state with the received player score
                                let player_chain_id = update.chain_id.to_string();
                                self.update_shard_score(&player, board_id, score, timestamp, player_chain_id, boards_in_tournament, leaderboard_id, game_status, highest_tile).await;
                                processed_player_updates = true;
                            } else {
                                log::warn!("🔥 PROCESS_STREAMS: ⚠️ Event was not a PlayerScoreUpdate event");
                            }
                        } else {
                            log::warn!("🔥 PROCESS_STREAMS: ⚠️ Could not read player_score_update event data");
                        }
                    }
                    "shard_score_update" => {
                        // Read the shard score event data and update leaderboard state
                        if let Some(event) = self.read_shard_score_event_from_chain(update.chain_id, event_index) {
                            if let game2048::GameEvent::ShardScoreUpdate { 
                                player_scores,
                                player_activity_scores,
                                player_board_counts,
                                shard_chain_id,
                                total_players: _,
                                leaderboard_id,
                                .. 
                            } = event {
                                log::info!("🏆 SHARD_TO_LB: Received {} player scores from shard {} for tournament {}", 
                                    player_scores.len(), shard_chain_id, leaderboard_id);
                                
                                // 🚀 IMPROVED: Update leaderboard state with smart merging (real-time stream processing)
                                let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();

                                
                                for (player, summary) in player_scores.iter() {
                                    let current_score = leaderboard.score.get(player).await.unwrap().unwrap_or(0);
                                    
                                     // Update if better score or equal score with newer timestamp
                                     if summary.best_score >= current_score {
                                         log::info!("🏆 LEADERBOARD_UPDATE: Player '{}' score: {} → {} (improvement: +{})", 
                                             player, current_score, summary.best_score, summary.best_score - current_score);
                                         leaderboard.score.insert(player, summary.best_score).unwrap();
                                         leaderboard.board_ids.insert(player, summary.board_id.clone()).unwrap();

                                         
                                         // Log final leaderboard state for this player
                                         log::info!("🏆 LEADERBOARD_STATE: Player '{}' now has score {} in leaderboard {}", 
                                             player, summary.best_score, leaderboard_id);
                                     }
                                }
                                
                                // 🚀 NEW: Update activity scores for triggerer ranking
                                let mut activity_updated_players = 0u32;
                                for (player, activity_score) in player_activity_scores.iter() {
                                    // Always update activity score (it's time-based, not cumulative)
                                    leaderboard.player_activity_scores.insert(player, *activity_score).unwrap();
                                    activity_updated_players += 1;
                                }
                                
                                // 🚀 NEW: Update total board and player counts (distributed counting)
                                let total_boards: u32 = player_board_counts.values().sum();
                                let total_players = player_board_counts.len() as u32;
                                
                                leaderboard.total_boards.set(total_boards);
                                leaderboard.total_players.set(total_players);
                                
                                log::info!("🏆 BOARD_COUNT_UPDATE: Tournament '{}' totals - Boards: {}, Players: {}", 
                                          leaderboard_id, total_boards, total_players);
                                
                                // 🚀 NEW: Update triggerer list based on activity scores (not game scores)
                                if activity_updated_players > 0 {
                                    use crate::contract_domain::handlers::messages::LeaderboardMessageHandler;
                                    LeaderboardMessageHandler::update_triggerer_list_by_activity(self, &leaderboard_id).await;
                                }
                                
                                log::info!("🔥 PROCESS_STREAMS: ✅ Leaderboard updated via streaming - tournament: {}", leaderboard_id);
                            } else {
                                log::warn!("🔥 PROCESS_STREAMS: ⚠️ Event was not a ShardScoreUpdate event");
                            }
                        } else {
                            log::warn!("🔥 PROCESS_STREAMS: ⚠️ Could not read shard_score_update event data");
                        }
                    }
                    "active_tournaments" => {
                        log::info!("🔥 PROCESS_STREAMS: 🏆 Processing active_tournaments event at index {} from main chain {}", event_index, update.chain_id);
                        
                        // Read the tournament event data and update local cache
                        if let Some(event) = self.read_active_tournaments_event_from_chain(update.chain_id, event_index) {
                            if let game2048::GameEvent::ActiveTournaments { tournaments, timestamp } = event {
                                log::info!("🔥 PROCESS_STREAMS: 📋 Updating local tournament cache with {} tournaments", tournaments.len());
                                
                                // Update local tournament cache with the new data
                                self.update_local_tournament_cache(tournaments, timestamp).await;
                                
                                log::info!("🔥 PROCESS_STREAMS: ✅ Local tournament cache updated successfully");
                            } else {
                                log::warn!("🔥 PROCESS_STREAMS: ⚠️ Event was not an ActiveTournaments event");
                            }
                        } else {
                            log::warn!("🔥 PROCESS_STREAMS: ⚠️ Could not read active_tournaments event data");
                        }
                    }
                    "leaderboard_update" => {
                        // Read the leaderboard update event and handle triggerer logic
                        if let Some(event) = self.read_leaderboard_update_event_from_chain(update.chain_id, event_index) {
                            if let game2048::GameEvent::LeaderboardUpdate { 
                                leaderboard_id,
                                triggerer_list,
                                last_update_timestamp,
                                threshold_config,
                                ..
                            } = event {
                                log::info!("⏰ TRIGGERER_UPDATE: Received triggerer list with {} players for tournament {}", 
                                    triggerer_list.len(), leaderboard_id);
                                
                                // Update local triggerer configuration
                                self.update_triggerer_config(leaderboard_id, triggerer_list, last_update_timestamp, threshold_config).await;
                                
                                // Check if this player should send a trigger
                                self.check_and_send_trigger_if_needed(update.chain_id).await;
                            } else {
                                log::warn!("🔥 PROCESS_STREAMS: ⚠️ Event was not a LeaderboardUpdate event");
                            }
                        } else {
                            log::warn!("🔥 PROCESS_STREAMS: ⚠️ Could not read leaderboard_update event data");
                        }
                    }
                    _ => {
                        log::info!("🔥 PROCESS_STREAMS: ❓ Unknown stream '{}' - ignoring", stream_name);
                    }
                }
            }
        }

        // After processing all streams, emit aggregated shard scores if we processed player updates
        if processed_player_updates {
            log::info!("🔥 PROCESS_STREAMS: 🚀 Triggering score aggregation (processed_player_updates = true)");
            
            // Get monitored player chains from shard state and aggregate their scores
            let shard = self.state.shards.load_entry_mut("").await.unwrap();
            let mut player_chain_ids = Vec::new();

            // Collect all monitored player chain IDs from the queue
            match shard.monitored_player_chains.read_front(100).await {
                Ok(chain_id_strings) => {
                    log::info!("🔥 PROCESS_STREAMS: 📋 Found {} monitored player chains", chain_id_strings.len());
                    for chain_id_str in chain_id_strings {
                        if let Ok(chain_id) = ChainId::from_str(&chain_id_str) {
                            player_chain_ids.push(chain_id);
                        }
                    }
                }
                Err(_) => {
                    log::warn!("🔥 PROCESS_STREAMS: ⚠️ No monitored player chains found or error reading them");
                }
            }

            // Emit aggregated player scores from this shard
            self.aggregate_scores_from_player_chains(player_chain_ids).await;
        }
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
        self.runtime.chain_id().to_string()
            == self.runtime.application_creator_chain_id().to_string()
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
            })
            .send_to(send_to.unwrap_or(chain_id));
    }

    fn transfer(&mut self, destination: ChainId, amount: Amount) {
        let account = Account {
            chain_id: destination,
            owner: AccountOwner::CHAIN,
        };
        self.runtime.transfer(AccountOwner::CHAIN, account, amount);
    }

    fn auto_faucet(&mut self, faucet_amount: Option<u128>) {
        let current_balance = self.runtime.chain_balance();
        if current_balance.saturating_mul(10) < Amount::from_tokens(5) {
            let app_chain_id = self.runtime.application_creator_chain_id();
            let chain_id = self.runtime.chain_id();

            self.runtime
                .prepare_message(Message::Transfer {
                    chain_id,
                    amount: Amount::from_tokens(faucet_amount.unwrap_or(1)),
                })
                .send_to(app_chain_id);
        }
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
        ShardOperationHandler::update_shard_score(self, player, board_id, score, timestamp, player_chain_id, boards_in_tournament, leaderboard_id, game_status, highest_tile).await;
    }

    async fn check_player_registered(
        &mut self,
        player_username: &str,
        check: RegistrationCheck,
    ) -> String {
        PlayerOperationHandler::check_player_registered(self, player_username, check).await
    }

    async fn validate_player_password(&mut self, player_username: &str, provided_password_hash: &str) {
        PlayerOperationHandler::validate_player_password(self, player_username, provided_password_hash).await;
    }

    /// Reads a player score event from another chain
    pub fn read_player_score_event_from_chain(&mut self, chain_id: ChainId, event_index: u32) -> Option<GameEvent> {
        StreamProcessingHandler::read_player_score_event_from_chain(self, chain_id, event_index)
    }

    /// Reads a shard score event from another chain
    pub fn read_shard_score_event_from_chain(&mut self, chain_id: ChainId, event_index: u32) -> Option<GameEvent> {
        StreamProcessingHandler::read_shard_score_event_from_chain(self, chain_id, event_index)
    }

    /// Subscribes to player score events from another chain
    pub fn subscribe_to_player_score_events(&mut self, chain_id: ChainId) {
        StreamProcessingHandler::subscribe_to_player_score_events(self, chain_id);
    }

    /// Subscribes to shard score events from another chain
    pub fn subscribe_to_shard_score_events(&mut self, chain_id: ChainId) {
        StreamProcessingHandler::subscribe_to_shard_score_events(self, chain_id);
    }

    pub fn subscribe_to_leaderboard_update_events(&mut self, chain_id: ChainId) {
        StreamProcessingHandler::subscribe_to_leaderboard_update_events(self, chain_id);
    }

    /// Reads latest active tournaments from main chain
    pub async fn read_active_tournaments(&mut self) -> Option<GameEvent> {
        TournamentOperationHandler::read_active_tournaments(self).await
    }

    /// Reads latest shard workload - ascends until error (blockchain-style)
    pub async fn read_shard_workload(&mut self, shard_chain_id: ChainId) -> Option<GameEvent> {
        match TournamentOperationHandler::read_shard_workload(self, shard_chain_id).await {
            Ok(event) => event,
            Err(_) => None,
        }
    }

    /// Validates tournament exists and is active
    pub async fn validate_tournament(&mut self, tournament_id: &str) -> bool {
        TournamentOperationHandler::validate_tournament(self, tournament_id).await
    }

    /// Shard chain functionality - aggregates scores with smart player activity tracking
    pub async fn aggregate_scores_from_player_chains(&mut self, player_chain_ids: Vec<ChainId>) {
        ShardOperationHandler::aggregate_scores_from_player_chains(self, player_chain_ids).await;
    }

    /// Leaderboard chain functionality - aggregates scores from multiple shard chains with proper index tracking
    pub async fn update_leaderboard_from_shard_chains(&mut self, shard_chain_ids: Vec<ChainId>) {
        LeaderboardOperationHandler::update_leaderboard_from_shard_chains(self, shard_chain_ids).await;
    }

    /// Emit current active tournaments (for leaderboard chains)
    pub async fn emit_active_tournaments(&mut self) {
        LeaderboardOperationHandler::emit_active_tournaments(self).await;
    }

    /// Emit current shard workload (for shard chains)
    pub async fn emit_shard_workload(&mut self) {
        ShardOperationHandler::emit_shard_workload(self).await;
    }

    /// Register player with shard and update workload tracking
    pub async fn register_player_with_shard(&mut self, player_chain_id: String, tournament_id: String, player_name: String) {
        ShardOperationHandler::register_player_with_shard(self, player_chain_id, tournament_id, player_name).await;
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
    pub async fn handle_aggregation_trigger_request(&mut self, requester_chain_id: &str, timestamp: u64) -> Result<(), String> {
        LeaderboardOperationHandler::handle_aggregation_trigger_request(self, requester_chain_id, timestamp).await
    }
    
    /// Emit game creation event helper
    pub async fn emit_game_creation_event(&mut self, board_id: &str, player: &str, tournament_id: &str, timestamp: u64) {
        StreamProcessingHandler::emit_game_creation_event(self, board_id, player, tournament_id, timestamp).await;
    }

    // ========================================
    // TOURNAMENT CACHE MANAGEMENT (STREAMING)
    // ========================================
    
    /// Update local tournament cache with latest data from main chain
    async fn update_local_tournament_cache(&mut self, tournaments: Vec<game2048::TournamentInfo>, timestamp: u64) {
        log::info!("📋 CACHE: Updating local tournament cache with {} tournaments", tournaments.len());
        
        // Clear existing cache
        let mut keys_to_remove = Vec::new();
        self.state.tournaments_cache_json.for_each_index_while(|key| {
            keys_to_remove.push(key);
            Ok(true) // Continue iteration
        }).await.unwrap();
        
        for key in keys_to_remove {
            self.state.tournaments_cache_json.remove(&key).unwrap();
        }
        
        // Add all tournaments to cache as JSON
        for tournament in tournaments {
            log::info!("📋 CACHE: Storing tournament '{}' ({})", 
                tournament.name, tournament.tournament_id);
            
            let tournament_id = tournament.tournament_id.clone();
            if let Ok(tournament_json) = serde_json::to_string(&tournament) {
                self.state.tournaments_cache_json
                    .insert(&tournament_id, tournament_json)
                    .unwrap();
            } else {
                log::error!("📋 CACHE: Failed to serialize tournament {}", tournament_id);
            }
        }
        
        // Update timestamp
        self.state.last_tournament_update.set(timestamp);
        
        let total_count = self.state.tournaments_cache_json.count().await.unwrap();
        log::info!("📋 CACHE: Tournament cache updated - {} tournaments stored", total_count);
    }
    
    /// Get cached tournament info (avoids cross-chain reads)
    pub async fn get_cached_tournament(&mut self, tournament_id: &str) -> Option<game2048::TournamentInfo> {
        if let Some(tournament_json) = self.state.tournaments_cache_json.get(tournament_id).await.unwrap() {
            serde_json::from_str(&tournament_json).ok()
        } else {
            None
        }
    }
    
    /// List all cached tournaments
    pub async fn list_cached_tournaments(&mut self) -> Vec<game2048::TournamentInfo> {
        let mut tournaments = Vec::new();
        
        self.state.tournaments_cache_json.for_each_index_value_while(|_key, tournament_json| {
            if let Ok(tournament) = serde_json::from_str::<game2048::TournamentInfo>(&tournament_json) {
                tournaments.push(tournament);
            }
            Ok(true) // Continue iteration
        }).await.unwrap();
        
        tournaments
    }
    
    /// Get count of cached tournaments
    pub async fn get_cached_tournament_count(&mut self) -> u64 {
        self.state.tournaments_cache_json.count().await.unwrap() as u64
    }
    
    /// Read active tournaments event from chain using StreamProcessingHandler
    pub fn read_active_tournaments_event_from_chain(&mut self, chain_id: linera_sdk::linera_base_types::ChainId, event_index: u32) -> Option<game2048::GameEvent> {
        use crate::contract_domain::handlers::operations::StreamProcessingHandler;
        StreamProcessingHandler::read_active_tournaments_event_from_chain(self, chain_id, event_index)
    }

    /// Read leaderboard update event from chain using StreamProcessingHandler
    pub fn read_leaderboard_update_event_from_chain(&mut self, chain_id: linera_sdk::linera_base_types::ChainId, event_index: u32) -> Option<game2048::GameEvent> {
        use crate::contract_domain::handlers::operations::StreamProcessingHandler;
        StreamProcessingHandler::read_leaderboard_update_event_from_chain(self, chain_id, event_index)
    }

    /// Update local triggerer configuration from leaderboard_update event
    async fn update_triggerer_config(
        &mut self,
        leaderboard_id: String,
        triggerer_list: Vec<(String, u32)>,
        last_update_timestamp: u64,
        threshold_config: u64,
    ) {
        log::info!("⏰ TRIGGERER_CONFIG: Updating triggerer config for tournament {} with {} triggerers", 
                   leaderboard_id, triggerer_list.len());
        
        // Clear existing triggerer list - delete items until empty
        loop {
            match self.state.triggerer_list.read_front(1).await {
                Ok(items) if !items.is_empty() => {
                    self.state.triggerer_list.delete_front();
                }
                _ => break,
            }
        }
        
        // Clear existing activity scores - delete items until empty
        loop {
            match self.state.triggerer_activity_scores.read_front(1).await {
                Ok(items) if !items.is_empty() => {
                    self.state.triggerer_activity_scores.delete_front();
                }
                _ => break,
            }
        }
        
        // Add new triggerer list (activity-sorted)
        for (triggerer_id, activity_score) in &triggerer_list {
            self.state.triggerer_list.push_back(triggerer_id.clone());
            self.state.triggerer_activity_scores.push_back(*activity_score);
        }
        
        // Update configuration
        self.state.triggerer_list_timestamp.set(last_update_timestamp);
        self.state.trigger_threshold_config.set(threshold_config);
        self.state.total_registered_players.set(triggerer_list.len() as u32);
        
        log::info!("⏰ TRIGGERER_CONFIG: Updated {} players, threshold: {} μs", 
                   triggerer_list.len(), threshold_config);
    }

    /// Check if this player should send a trigger and send it if needed
    async fn check_and_send_trigger_if_needed(&mut self, leaderboard_chain_id: linera_sdk::linera_base_types::ChainId) {
        let current_time = self.runtime.system_time().micros();
        let my_chain_id = self.runtime.chain_id().to_string();
        
        // Get configuration
        let threshold = *self.state.trigger_threshold_config.get();
        let last_update_time = *self.state.triggerer_list_timestamp.get();
        let last_trigger_sent = *self.state.last_trigger_sent.get();
        let total_players = *self.state.total_registered_players.get();
        
        // Check if enough time has passed since last update
        let time_since_update = current_time.saturating_sub(last_update_time);
        let time_since_last_trigger = current_time.saturating_sub(last_trigger_sent);
        
        // 🚀 MATHEMATICAL TIER CALCULATION
        // tier = (time_since_update) / threshold  
        // Tier 1 (0-1x): N most active players trigger
        // Tier 2 (1-2x): 2N most active players trigger  
        // Tier 3 (2-3x): 3N most active players trigger
        // etc. up to Tier 5 (4-5x): 5N most active players trigger
        let tier = if threshold > 0 {
            std::cmp::min(5, (time_since_update / threshold) + 1)
        } else {
            1 // Default to tier 1 if threshold is 0
        };
        
        // Calculate how many players should be actively triggering
        let base_triggerer_count = std::cmp::max(2, total_players / 10); // At least 2, or 10% of players
        let active_triggerer_count = std::cmp::min(total_players, base_triggerer_count * tier as u32);
        
        log::info!("⏰ TRIGGER_CALC: Tier {} calculation - time_since_update: {} μs, threshold: {} μs", 
                  tier, time_since_update, threshold);
        log::info!("⏰ TRIGGER_CALC: Active triggerers: {} of {} total players (base: {})", 
                  active_triggerer_count, total_players, base_triggerer_count);
        
        // Find my position in the triggerer list
        let mut my_position: Option<u32> = None;
        
        match self.state.triggerer_list.read_front(active_triggerer_count as usize).await {
            Ok(triggerers) => {
                for (i, triggerer_id) in triggerers.iter().enumerate() {
                    if triggerer_id == &my_chain_id {
                        my_position = Some(i as u32);
                        break;
                    }
                }
            }
            Err(_) => {
                log::info!("⏰ TRIGGER_CHECK: Could not read triggerer list");
                return;
            }
        }
        
        let am_i_active_triggerer = match my_position {
            Some(pos) => pos < active_triggerer_count,
            None => false,
        };
        
        if !am_i_active_triggerer {
            if let Some(pos) = my_position {
                log::info!("⏰ TRIGGER_CHECK: Not an active triggerer (position {} >= {})", 
                          pos, active_triggerer_count);
            } else {
                log::info!("⏰ TRIGGER_CHECK: Not in triggerer list, skipping trigger check");
            }
            return;
        }
        
        // Only trigger if enough time has passed since our last trigger
        let should_trigger = time_since_last_trigger > threshold;
        
        if should_trigger {
            let my_pos = my_position.unwrap();
            log::info!("⏰ TRIGGER_SEND: Sending trigger update (position {}/{}, tier {}, time_since_update: {} μs)", 
                      my_pos, active_triggerer_count, tier, time_since_update);
            
            // Get tournament ID from the first cached tournament (assuming single tournament for now)
            let mut tournament_id = String::new();
            self.state.tournaments_cache_json.for_each_index_while(|key| {
                tournament_id = key;
                Ok(false) // Stop after first tournament
            }).await.unwrap();
            
            if !tournament_id.is_empty() {
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
                
                log::info!("⏰ TRIGGER_SEND: ✅ Sent trigger update for tournament {} (Tier {}, Position {})", 
                          tournament_id, tier, my_pos);
            } else {
                log::warn!("⏰ TRIGGER_SEND: ❌ No tournament found in cache, cannot send trigger");
            }
        } else {
            log::info!("⏰ TRIGGER_CHECK: Not time to trigger yet (time_since_last_trigger: {} μs, threshold: {} μs)", 
                      time_since_last_trigger, threshold);
        }
    }
}
