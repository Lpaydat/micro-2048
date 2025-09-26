#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;
mod contract_domain;

use linera_sdk::{
    linera_base_types::{Account, AccountOwner, Amount, ChainId},
    abi::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};
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
        for update in updates {
            // Determine which stream we're processing based on stream name
            let stream_name_bytes = &update.stream_id.stream_name.0;
            let stream_name = String::from_utf8_lossy(stream_name_bytes);
            
            // Process all new events in this stream update
            for event_index in update.previous_index..update.next_index {
                match stream_name.as_ref() {
                    "player_score_update" => {
                        if let Some(GameEvent::PlayerScoreUpdate { 
                            player, 
                            score, 
                            timestamp, 
                            .. 
                        }) = self.read_player_score_event_from_chain(update.chain_id, event_index) {
                            // Update shard state based on player score updates
                            self.update_shard_score(&player, format!("remote_{}", timestamp), score, timestamp).await;
                        }
                    }
                    "shard_score_update" => {
                        if let Some(event) = self.read_shard_score_event_from_chain(update.chain_id, event_index) {
                            if let GameEvent::ShardScoreUpdate { 
                                player_scores,
                                .. 
                            } = event {
                                // 🚀 IMPROVED: Update leaderboard state with smart merging (real-time stream processing)
                                let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
                                for (player, summary) in player_scores.iter() {
                                    let current_score = leaderboard.score.get(player).await.unwrap().unwrap_or(0);
                                    
                                    // Update if better score or equal score with newer timestamp
                                    if summary.best_score >= current_score {
                                        leaderboard.score.insert(player, summary.best_score).unwrap();
                                        leaderboard.board_ids.insert(player, summary.board_id.clone()).unwrap();
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        // Unknown stream, ignore
                    }
                }
            }
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
    ) {
        ShardOperationHandler::update_shard_score(self, player, board_id, score, timestamp).await;
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

    /// Reads latest active tournaments - ascends until error (blockchain-style)
    pub async fn read_active_tournaments(&mut self, leaderboard_chain_id: ChainId) -> Option<GameEvent> {
        TournamentOperationHandler::read_active_tournaments(self, leaderboard_chain_id).await
    }

    /// Reads latest shard workload - ascends until error (blockchain-style)
    pub async fn read_shard_workload(&mut self, shard_chain_id: ChainId) -> Option<GameEvent> {
        TournamentOperationHandler::read_shard_workload(self, shard_chain_id).await
    }

    /// Validates tournament exists and is active
    pub async fn validate_tournament(&mut self, tournament_id: &str, leaderboard_chain_id: ChainId) -> bool {
        TournamentOperationHandler::validate_tournament(self, tournament_id, leaderboard_chain_id).await
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
    pub async fn select_optimal_shard(&mut self, tournament_id: &str, leaderboard_chain_id: ChainId) -> String {
        TournamentOperationHandler::select_optimal_shard(self, tournament_id, leaderboard_chain_id).await
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
}
