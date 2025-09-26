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
use state::{Leaderboard, LeaderboardShard};

use self::state::Game2048;
use game2048::{GameEndReason, GameEvent, GameStatus, Message, Operation, PlayerScoreSummary, RegistrationCheck, TournamentInfo, TournamentStatus};

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
                        if let Some(event) = self.read_player_score_event_from_chain(update.chain_id, event_index) {
                            if let GameEvent::PlayerScoreUpdate { 
                                player, 
                                score, 
                                timestamp, 
                                .. 
                            } = event {
                                // Update shard state based on player score updates
                                self.update_shard_score(&player, format!("remote_{}", timestamp), score, timestamp).await;
                            }
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
    fn is_main_chain(&mut self) -> bool {
        self.runtime.chain_id().to_string()
            == self.runtime.application_creator_chain_id().to_string()
    }

    async fn is_leaderboard_active(&mut self, timestamp: u64) -> &mut Leaderboard {
        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        let start_time = leaderboard.start_time.get();
        let end_time = leaderboard.end_time.get();

        // Basic bounds checking: prevent obviously invalid timestamps
        if timestamp > u64::MAX / 2 {
            panic!("Timestamp too large");
        }

        // Apply timestamp validation to all chains for consistency
        // Keep bypass for system operations (111970) - used for game ending without moves
        if timestamp != 111970
            && (timestamp < *start_time || timestamp > *end_time)
        {
            panic!("Leaderboard is not active");
        }

        leaderboard
    }

    async fn is_shard_active(&mut self, timestamp: u64) -> &mut LeaderboardShard {
        let shard = self.state.shards.load_entry_mut("").await.unwrap();
        let start_time = shard.start_time.get();
        let end_time = shard.end_time.get();

        // Basic bounds checking: prevent obviously invalid timestamps
        if timestamp > u64::MAX / 2 {
            panic!("Timestamp too large");
        }

        // Apply consistent validation to all chains (removed !is_main_chain check)
        // Keep bypass for system operations (111970) - used for game ending without moves
        if timestamp != 111970
            && (timestamp < *start_time || timestamp > *end_time)
        {
            panic!("Shard is not active");
        }

        shard
    }

    async fn update_shard_score(
        &mut self,
        player: &str,
        board_id: String,
        score: u64,
        timestamp: u64,
    ) {
        let shard = self.is_shard_active(timestamp).await;
        let player_shard_score = shard.score.get(player).await.unwrap();

        if player_shard_score.is_none() || player_shard_score < Some(score) {
            shard.score.insert(player, score).unwrap();
            shard.board_ids.insert(player, board_id).unwrap();
            shard.counter.set(*shard.counter.get() + 1);
        }
    }

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

    #[allow(dead_code)]
    fn update_score(
        &mut self,
        chain_id: ChainId,
        player: &str,
        board_id: &str,
        score: u64,
        is_end: bool,
        timestamp: u64,
    ) {
        self.runtime
            .prepare_message(Message::UpdateScore {
                player: player.to_string(),
                board_id: board_id.to_string(),
                score,
                is_end,
                timestamp,
            })
            .send_to(chain_id);
    }

    async fn check_player_registered(
        &mut self,
        player_username: &str,
        check: RegistrationCheck,
    ) -> String {
        let player = self
            .state
            .players
            .load_entry_or_insert(player_username)
            .await
            .unwrap();
        let username = player.username.get();

        let is_registered = !username.trim().is_empty();

        match check {
            RegistrationCheck::EnsureRegistered if !is_registered => {
                panic!("Player not registered");
            }
            RegistrationCheck::EnsureNotRegistered if is_registered => {
                panic!("Player already registered");
            }
            _ => {}
        }

        player.password_hash.get().to_string()
    }

    async fn validate_player_password(&mut self, player_username: &str, provided_password_hash: &str) {
        let stored_password_hash = self.check_player_registered(player_username, RegistrationCheck::EnsureRegistered).await;
        if stored_password_hash != provided_password_hash {
            panic!("Invalid password");
        }
    }

    /// Reads a player score event from another chain
    pub fn read_player_score_event_from_chain(&mut self, chain_id: ChainId, event_index: u32) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("player_score_update".to_string());
        
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.runtime.read_event(chain_id, stream_name, event_index)
        })) {
            Ok(event) => Some(event),
            Err(_) => None
        }
    }

    /// Reads a shard score event from another chain
    pub fn read_shard_score_event_from_chain(&mut self, chain_id: ChainId, event_index: u32) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("shard_score_update".to_string());
        
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.runtime.read_event(chain_id, stream_name, event_index)
        })) {
            Ok(event) => Some(event),
            Err(_) => None
        }
    }

    /// Subscribes to player score events from another chain
    pub fn subscribe_to_player_score_events(&mut self, chain_id: ChainId) {
        use linera_sdk::linera_base_types::{StreamName, ApplicationId};
        let stream_name = StreamName::from("player_score_update".to_string());
        let application_id = ApplicationId::new(self.runtime.application_id().application_description_hash);
        
        self.runtime.subscribe_to_events(chain_id, application_id, stream_name);
    }

    /// Subscribes to shard score events from another chain
    pub fn subscribe_to_shard_score_events(&mut self, chain_id: ChainId) {
        use linera_sdk::linera_base_types::{StreamName, ApplicationId};
        let stream_name = StreamName::from("shard_score_update".to_string());
        let application_id = ApplicationId::new(self.runtime.application_id().application_description_hash);
        
        self.runtime.subscribe_to_events(chain_id, application_id, stream_name);
    }

    /// 🚀 CORRECT: Reads latest active tournaments - ascends until error (blockchain-style)
    pub async fn read_active_tournaments(&mut self, leaderboard_chain_id: ChainId) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("active_tournaments".to_string());
        
        // Get last processed index
        let mut current_index = *self.state.active_tournaments_event_index.get();
        let mut latest_event: Option<GameEvent> = None;
        
        // 🚀 CRITICAL: Read ascending until error (latest has highest index)
        loop {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.runtime.read_event(leaderboard_chain_id, stream_name.clone(), current_index as u32)
            })) {
                Ok(event) => {
                    // Found an event - this could be the latest
                    latest_event = Some(event);
                    current_index += 1; // Try next index
                }
                Err(_) => {
                    // Hit error - no more events, current_index-1 was the latest
                    break;
                }
            }
        }
        
        // 🚀 CRITICAL: Update state with the latest index we successfully read
        if latest_event.is_some() {
            self.state.active_tournaments_event_index.set(current_index);
        }
        
        latest_event
    }

    /// 🚀 CORRECT: Reads latest shard workload - ascends until error (blockchain-style)
    pub async fn read_shard_workload(&mut self, shard_chain_id: ChainId) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("shard_workload".to_string());
        let chain_id_str = shard_chain_id.to_string();
        
        // Get last processed index for this shard
        let mut current_index = self.state
            .shard_workload_event_indices
            .get(&chain_id_str)
            .await
            .unwrap()
            .unwrap_or(0);
        
        let mut latest_event: Option<GameEvent> = None;
        
        // 🚀 CRITICAL: Read ascending until error (latest has highest index)
        loop {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.runtime.read_event(shard_chain_id, stream_name.clone(), current_index as u32)
            })) {
                Ok(event) => {
                    // Found an event - this could be the latest
                    latest_event = Some(event);
                    current_index += 1; // Try next index
                }
                Err(_) => {
                    // Hit error - no more events, current_index-1 was the latest
                    break;
                }
            }
        }
        
        // 🚀 CRITICAL: Update state with the latest index we successfully read
        if latest_event.is_some() {
            self.state
                .shard_workload_event_indices
                .insert(&chain_id_str, current_index)
                .unwrap();
        }
        
        latest_event
    }

    /// 🚀 NEW: Validates tournament exists and is active
    pub async fn validate_tournament(&mut self, tournament_id: &str, leaderboard_chain_id: ChainId) -> bool {
        if let Some(GameEvent::ActiveTournaments { tournaments, .. }) = self.read_active_tournaments(leaderboard_chain_id).await {
            return tournaments.iter().any(|t| 
                t.tournament_id == tournament_id && 
                matches!(t.status, TournamentStatus::Active)
            );
        }
        false
    }

    /// 🚀 PERFORMANCE: Shard chain functionality - aggregates scores with smart player activity tracking
    pub async fn aggregate_scores_from_player_chains(&mut self, player_chain_ids: Vec<ChainId>) {
        use std::collections::HashMap;
        
        let mut player_summaries: HashMap<String, PlayerScoreSummary> = HashMap::new();
        let current_time = self.runtime.system_time().micros();
        
        // 🚀 Get shard for activity tracking (accessed via self in methods)
        
        // Process each player chain with smart activity-based reading
        for chain_id in player_chain_ids.iter() {
            let chain_id_str = chain_id.to_string();
            
            // 🚀 PERFORMANCE: Check if we should read this player this round
            if !self.should_read_player_chain(&chain_id_str, current_time).await {
                continue; // Skip this player this round
            }
            
            // Get last processed index for this chain
            let last_processed_index = self.state
                .player_score_event_indices
                .get(&chain_id_str)
                .await
                .unwrap()
                .unwrap_or(0);
            
            // 🚀 CORRECT: Read ascending from last index until error (blockchain-style)
            let mut current_index = last_processed_index;
            
            // Read until we hit error (no more events)
            loop {
                if let Some(event) = self.read_player_score_event_from_chain(*chain_id, current_index as u32) {
                    match event {
                        GameEvent::PlayerScoreUpdate { 
                            player, 
                            score, 
                            board_id, 
                            chain_id: event_chain_id, 
                            timestamp,
                            game_status,
                            highest_tile,
                            current_leaderboard_best,
                            .. 
                        } => {
                            // 🚀 SMART FILTERING: Only process if score is an improvement
                            let is_improvement = score > current_leaderboard_best;
                            let is_game_lifecycle = matches!(
                                game_status, 
                                GameStatus::Created | 
                                GameStatus::Ended(GameEndReason::NoMoves) | 
                                GameStatus::Ended(GameEndReason::TournamentEnded)
                            );
                            
                            // Process if it's an improvement OR important lifecycle events
                            if is_improvement || is_game_lifecycle {
                                let should_update = if let Some(existing) = player_summaries.get(&player) {
                                    score > existing.best_score || timestamp > existing.last_update
                                } else {
                                    true
                                };
                                
                                if should_update {
                                    // 🚀 IMPROVED: Only keep latest score per player
                                    let new_summary = PlayerScoreSummary {
                                        player: player.clone(),
                                        best_score: score,
                                        board_id,
                                        chain_id: event_chain_id,
                                        highest_tile,
                                        last_update: timestamp,
                                        game_status,
                                    };
                                    
                                    // Insert or update - HashMap will replace existing entry
                                    player_summaries.insert(player.clone(), new_summary);
                                }
                            }
                            // 🗑️ FILTERED: Non-improvements are ignored by shard
                        },
                        _ => {
                            // Ignore other event types for score aggregation
                        }
                    }
                    
                    current_index += 1;
                } else {
                    // Hit error - no more events available
                    break;
                }
            }
            
            // 🚀 UPDATE INDEX TRACKING: Save our progress (current_index is where we stopped)
            if current_index > last_processed_index {
                self.state
                    .player_score_event_indices
                    .insert(&chain_id_str, current_index)
                    .unwrap();
                
                // 🚀 PERFORMANCE: Update player activity (found new events)
                self.update_player_activity(&chain_id_str, current_time, true).await;
            } else {
                // 🚀 PERFORMANCE: Update player activity (no new events)
                self.update_player_activity(&chain_id_str, current_time, false).await;
            }
        }
        
        // If we found any scores, emit a shard aggregation event
        if !player_summaries.is_empty() {
            let shard = self.state.shards.load_entry_mut("").await.unwrap();
            let leaderboard_id = shard.leaderboard_id.get().clone();
            
            use linera_sdk::linera_base_types::StreamName;
            let stream_name = StreamName::from("shard_score_update".to_string());
            let aggregation_event = GameEvent::ShardScoreUpdate {
                shard_chain_id: self.runtime.chain_id().to_string(),
                player_scores: player_summaries.clone(),
                aggregation_timestamp: self.runtime.system_time().micros(),
                total_players: player_summaries.len() as u32,
                leaderboard_id,
            };
            
            self.runtime.emit(stream_name, &aggregation_event);
        }
        
        // 🚀 IMPROVED: Update local shard state with comprehensive tracking
        if !player_summaries.is_empty() {
            let shard = self.state.shards.load_entry_mut("").await.unwrap();
            let mut shard_updated_players = 0u32;
            
            for (player, summary) in player_summaries.iter() {
                let current_score = shard.score.get(player).await.unwrap().unwrap_or(0);
                
                // Update if we have better score OR equal score (to keep latest board_id)
                if summary.best_score >= current_score {
                    shard.score.insert(player, summary.best_score).unwrap();
                    shard.board_ids.insert(player, summary.board_id.clone()).unwrap();
                    
                    if summary.best_score > current_score {
                        shard_updated_players += 1;
                    }
                }
            }
            
            // Update shard metadata
            shard.last_activity.set(current_time);
            if shard_updated_players > 0 {
                // Increment total games processed (rough metric)
                let current_games = *shard.total_games_count.get();
                shard.total_games_count.set(current_games + shard_updated_players);
            }
        }
    }

    /// 🚀 IMPROVED: Leaderboard chain functionality - aggregates scores from multiple shard chains with proper index tracking
    pub async fn update_leaderboard_from_shard_chains(&mut self, shard_chain_ids: Vec<ChainId>) {
        use std::collections::HashMap;
        
        let mut all_player_summaries: HashMap<String, PlayerScoreSummary> = HashMap::new();
        
        // Process each shard chain with index tracking
        for chain_id in shard_chain_ids.iter() {
            let chain_id_str = chain_id.to_string();
            
            // Get last processed index for this shard chain
            let last_processed_index = self.state
                .shard_score_event_indices
                .get(&chain_id_str)
                .await
                .unwrap()
                .unwrap_or(0);
            
            // 🚀 CORRECT: Read ascending from last index until error (blockchain-style)  
            let mut current_index = last_processed_index;
            
            // Read until we hit error (no more events)
            loop {
                if let Some(event) = self.read_shard_score_event_from_chain(*chain_id, current_index as u32) {
                    match event {
                        GameEvent::ShardScoreUpdate { 
                            player_scores,
                            .. 
                        } => {
                            // 🚀 IMPROVED: Smart merge player summaries from this shard
                            for (player, summary) in player_scores.iter() {
                                let should_update = if let Some(existing) = all_player_summaries.get(player) {
                                    // Update if better score OR more recent timestamp
                                    summary.best_score > existing.best_score || 
                                    (summary.best_score == existing.best_score && summary.last_update > existing.last_update)
                                } else {
                                    true // New player
                                };
                                
                                if should_update {
                                    // 🚀 CRITICAL: Merge with the BEST data from any shard
                                    let merged_summary = if let Some(existing) = all_player_summaries.get(player) {
                                        PlayerScoreSummary {
                                            player: player.clone(),
                                            best_score: summary.best_score.max(existing.best_score),
                                            board_id: if summary.best_score >= existing.best_score { 
                                                summary.board_id.clone() 
                                            } else { 
                                                existing.board_id.clone() 
                                            },
                                            chain_id: if summary.best_score >= existing.best_score { 
                                                summary.chain_id.clone() 
                                            } else { 
                                                existing.chain_id.clone() 
                                            },
                                            highest_tile: summary.highest_tile.max(existing.highest_tile),
                                            last_update: summary.last_update.max(existing.last_update),
                                            game_status: if summary.last_update >= existing.last_update { 
                                                summary.game_status.clone() 
                                            } else { 
                                                existing.game_status.clone() 
                                            },
                                        }
                                    } else {
                                        summary.clone()
                                    };
                                    
                                    all_player_summaries.insert(player.clone(), merged_summary);
                                }
                            }
                        },
                        _ => {
                            // Ignore other event types
                        }
                    }
                    
                    current_index += 1;
                } else {
                    // Hit error - no more events available
                    break;
                }
            }
            
            // 🚀 UPDATE INDEX TRACKING: Save our progress (current_index is where we stopped)
            if current_index > last_processed_index {
                self.state
                    .shard_score_event_indices
                    .insert(&chain_id_str, current_index)
                    .unwrap();
            }
        }
        
        // 🚀 IMPROVED: Update leaderboard state with comprehensive tracking
        if !all_player_summaries.is_empty() {
            let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
            let mut players_updated = 0u32;
            let mut total_unique_players = 0u32;
            
            // Update leaderboard state with all player data
            for (player, summary) in all_player_summaries.iter() {
                let current_score = leaderboard.score.get(player).await.unwrap().unwrap_or(0);
                total_unique_players += 1;
                
                // Always update if we have better score OR if this is a new player
                if summary.best_score >= current_score {
                    leaderboard.score.insert(player, summary.best_score).unwrap();
                    leaderboard.board_ids.insert(player, summary.board_id.clone()).unwrap();
                    
                    if summary.best_score > current_score {
                        players_updated += 1;
                    }
                }
            }
            
            // Update leaderboard metadata
            leaderboard.total_players.set(total_unique_players);
            
            // 🚀 ENSURE: Subscribe to all shard chains for real-time updates
            for chain_id in shard_chain_ids.iter() {
                self.subscribe_to_shard_score_events(*chain_id);
            }
            
            // 🚀 DYNAMIC: Update triggerer pool based on latest scores
            self.update_triggerer_pool().await;
            
            // 🚀 LOG: Report update statistics (for debugging)
            if players_updated > 0 {
                // Could emit an event here for monitoring
                // For now, just ensure state is updated
            }
        }
    }

    /// 🚀 NEW: Emit current active tournaments (for leaderboard chains)
    pub async fn emit_active_tournaments(&mut self) {
        use linera_sdk::linera_base_types::StreamName;
        
        // For now, create a simple tournament list from current leaderboard state
        // In a full implementation, this would read from a tournaments registry
        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        let tournament_id = leaderboard.leaderboard_id.get().clone();
        
        if !tournament_id.is_empty() {
            let tournament_info = TournamentInfo {
                tournament_id: tournament_id.clone(),
                name: leaderboard.name.get().clone(),
                shard_chain_ids: leaderboard.shard_ids.read_front(100).await.unwrap_or_default(),
                start_time: *leaderboard.start_time.get(),
                end_time: *leaderboard.end_time.get(),
                status: TournamentStatus::Active,
                total_players: *leaderboard.total_players.get(),
            };
            
            let tournaments_event = GameEvent::ActiveTournaments {
                tournaments: vec![tournament_info],
                timestamp: self.runtime.system_time().micros(),
            };
            
            let stream_name = StreamName::from("active_tournaments".to_string());
            self.runtime.emit(stream_name, &tournaments_event);
        }
    }

    /// 🚀 IMPROVED: Emit current shard workload (for shard chains)
    pub async fn emit_shard_workload(&mut self) {
        use linera_sdk::linera_base_types::StreamName;
        
        let shard = self.state.shards.load_entry_mut("").await.unwrap();
        let tournament_id = shard.leaderboard_id.get().clone();
        
        if !tournament_id.is_empty() {
            // Use the new tracking fields
            let total_players = *shard.active_players_count.get();
            let _total_games = *shard.total_games_count.get();
            let last_activity = *shard.last_activity.get();
            
            // Calculate active players in last 5 minutes (300,000 microseconds)
            let current_time = self.runtime.system_time().micros();
            let five_minutes_ago = current_time.saturating_sub(300_000_000);
            let active_players_last_5min = if last_activity >= five_minutes_ago {
                (total_players as f32 * 0.8) as u32 // 80% if recent activity
            } else {
                (total_players as f32 * 0.2) as u32 // 20% if stale
            };
            
            let workload_event = GameEvent::ShardWorkload {
                shard_chain_id: self.runtime.chain_id().to_string(),
                tournament_id,
                total_players,
                active_players_last_5min,
                timestamp: current_time,
            };
            
            let stream_name = StreamName::from("shard_workload".to_string());
            self.runtime.emit(stream_name, &workload_event);
        }
    }

    /// 🚀 NEW: Register player with shard and update workload tracking
    pub async fn register_player_with_shard(&mut self, player_chain_id: String, tournament_id: String, _player_name: String) {
        let shard = self.state.shards.load_entry_mut("").await.unwrap();
        
        // Check if this is the right tournament
        if shard.leaderboard_id.get() == &tournament_id {
            // Add to monitoring list (simple approach for now - duplicates will be handled elsewhere)
            shard.monitored_player_chains.push_back(player_chain_id.clone());
            
            // Update workload statistics
            shard.active_players_count.set(*shard.active_players_count.get() + 1);
            let current_time = self.runtime.system_time().micros();
            shard.last_activity.set(current_time);
            
            // 🚀 PERFORMANCE: Initialize smart activity tracking for new player
            shard.player_activity_levels.insert(&player_chain_id, 0).unwrap(); // Start as very_active
            shard.player_read_intervals.insert(&player_chain_id, 1).unwrap(); // Read every round initially
            shard.player_last_seen.insert(&player_chain_id, current_time).unwrap();
            
            // Subscribe to this player chain's events
            if let Ok(chain_id) = ChainId::from_str(&player_chain_id) {
                self.subscribe_to_player_score_events(chain_id);
            }
        }
    }
    
    /// 🚀 NEW: Update game count when games are created/ended
    pub async fn track_game_activity(&mut self) {
        let shard = self.state.shards.load_entry_mut("").await.unwrap();
        shard.total_games_count.set(*shard.total_games_count.get() + 1);
        shard.last_activity.set(self.runtime.system_time().micros());
    }

    /// 🚀 NEW: Select optimal shard for a tournament based on workload
    pub async fn select_optimal_shard(&mut self, tournament_id: &str, leaderboard_chain_id: ChainId) -> String {
        
        // Get tournament info to find available shards
        if let Some(GameEvent::ActiveTournaments { tournaments, .. }) = self.read_active_tournaments(leaderboard_chain_id).await {
            if let Some(tournament) = tournaments.iter().find(|t| t.tournament_id == tournament_id) {
                // Read workload from each shard and select the least loaded
                let mut best_shard = tournament.shard_chain_ids.first().cloned().unwrap_or_else(|| self.runtime.chain_id().to_string()); // Use first available
                let mut lowest_load = u32::MAX;
                
                for shard_chain_id_str in tournament.shard_chain_ids.iter() {
                    if let Ok(shard_chain_id) = ChainId::from_str(shard_chain_id_str) {
                        if let Some(GameEvent::ShardWorkload { 
                            active_players_last_5min, 
                            total_players,
                            .. 
                        }) = self.read_shard_workload(shard_chain_id).await {
                            // Calculate load score (active players + 20% buffer for total players)
                            let load_score = active_players_last_5min + (total_players / 5);
                            
                            if load_score < lowest_load {
                                lowest_load = load_score;
                                best_shard = shard_chain_id_str.clone();
                            }
                        }
                    }
                }
                
                return best_shard;
            }
        }
        
        // Fallback: Use first available shard from leaderboard state
        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        match leaderboard.shard_ids.read_front(1).await {
            Ok(shard_ids) => {
                if let Some(first_shard_id) = shard_ids.first() {
                    first_shard_id.clone()
                } else {
                    // Ultimate fallback if no shards registered
                    self.runtime.chain_id().to_string() 
                }
            }
            Err(_) => {
                // Error fallback - use current chain as shard
                self.runtime.chain_id().to_string()
            }
        }
    }

    /// 🚀 PERFORMANCE: Smart algorithm to decide if we should read a player chain this round
    async fn should_read_player_chain(&mut self, chain_id_str: &str, current_time: u64) -> bool {
        let shard = self.state.shards.load_entry_mut("").await.unwrap();
        
        // Get player's read interval multiplier (1, 5, 15)
        let read_interval = shard
            .player_read_intervals
            .get(chain_id_str)
            .await
            .unwrap()
            .unwrap_or(1); // Default: read every round
        
        // Get last time we read this player
        let last_seen = shard
            .player_last_seen
            .get(chain_id_str)
            .await
            .unwrap()
            .unwrap_or(0);
        
        // Calculate time since last read (in seconds, roughly)
        let time_since_read = current_time.saturating_sub(last_seen) / 1_000_000;
        
        // Should we read based on interval?
        let should_read = match read_interval {
            1 => true, // Every round (very active players)
            5 => time_since_read >= 10, // Every ~10 seconds (active players)
            15 => time_since_read >= 30, // Every ~30 seconds (inactive players)
            _ => time_since_read >= 60, // Every ~60 seconds (very inactive players)
        };
        
        should_read
    }

    /// 🚀 PERFORMANCE: Update player activity level based on event presence
    async fn update_player_activity(&mut self, chain_id_str: &str, current_time: u64, found_new_events: bool) {
        let shard = self.state.shards.load_entry_mut("").await.unwrap();
        
        // Update last seen time
        shard.player_last_seen.insert(chain_id_str, current_time).unwrap();
        
        // Get current activity level (0=very_active, 1=active, 2=inactive, 3=very_inactive)
        let current_level = shard
            .player_activity_levels
            .get(chain_id_str)
            .await
            .unwrap()
            .unwrap_or(0); // Default: very active
        
        let new_level = if found_new_events {
            // Found events - promote activity level
            match current_level {
                2 | 3 => 1, // inactive/very_inactive -> active
                _ => 0,     // already active -> very_active
            }
        } else {
            // No events - demote activity level
            match current_level {
                0 => 1, // very_active -> active  
                1 => 2, // active -> inactive
                2 => 3, // inactive -> very_inactive
                _ => 3, // stay very_inactive
            }
        };
        
        // Update activity level
        shard.player_activity_levels.insert(chain_id_str, new_level).unwrap();
        
        // Update read interval based on activity level
        let new_interval = match new_level {
            0 => 1,  // very_active: read every round
            1 => 1,  // active: read every round
            2 => 5,  // inactive: read every 5 rounds (10 seconds)
            _ => 15, // very_inactive: read every 15 rounds (30 seconds)
        };
        
        shard.player_read_intervals.insert(chain_id_str, new_interval).unwrap();
    }

    /// 🚀 IMPROVED: Dynamic Triggerer Management - Updates based on ACTUAL SCORES
    pub async fn update_triggerer_pool(&mut self) {
        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        
        // Collect top 5 players by score
        let mut top_players: Vec<(String, u64)> = Vec::new();
        
        // Iterate through all scores to find top 5
        leaderboard.score.for_each_index_value(|player, score| {
            top_players.push((player.clone(), *score));
            Ok(())
        }).await.unwrap();
        
        // Sort by score (descending)
        top_players.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Take top 5 (or fewer if less players)
        top_players.truncate(5);
        
        // Update triggerer pool
        if !top_players.is_empty() {
            // First player is primary
            if let Some((top_player, _)) = top_players.first() {
                // Get board_id for the top player to get their chain_id
                if let Some(board_id) = leaderboard.board_ids.get(top_player).await.unwrap() {
                    // Extract chain_id from board_id (format: "chain_id.hash")
                    let chain_id = board_id.split('.').next().unwrap_or(top_player).to_string();
                    leaderboard.primary_triggerer.set(chain_id);
                }
            }
            
            // Clear and rebuild backup pool with players 2-5
            // Clear old backups (QueueView doesn't have clear, so we delete all)
            while leaderboard.backup_triggerers.count() > 0 {
                leaderboard.backup_triggerers.delete_front();
            }
            
            // Add new backups (positions 2-5)
            for i in 1..top_players.len().min(5) {
                if let Some((player, _)) = top_players.get(i) {
                    if let Some(board_id) = leaderboard.board_ids.get(player).await.unwrap() {
                        let chain_id = board_id.split('.').next().unwrap_or(player).to_string();
                        leaderboard.backup_triggerers.push_back(chain_id);
                    }
                }
            }
        }
        
        // Update rotation counter for tracking
        let counter = *leaderboard.trigger_rotation_counter.get();
        leaderboard.trigger_rotation_counter.set(counter + 1);
    }
    
    /// 🚀 NEW: Check if a chain is authorized to trigger
    pub async fn is_authorized_triggerer(&mut self, requester_chain_id: &str) -> bool {
        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        
        // Check if primary triggerer
        if leaderboard.primary_triggerer.get() == requester_chain_id {
            return true;
        }
        
        // Check if in backup pool
        let backup_triggerers = leaderboard.backup_triggerers.read_front(5).await.unwrap_or_default();
        backup_triggerers.contains(&requester_chain_id.to_string())
    }
    
    /// 🚀 IMPROVED: Handle aggregation trigger request with robust cooldown
    pub async fn handle_aggregation_trigger_request(&mut self, requester_chain_id: &str, _timestamp: u64) -> Result<(), String> {
        // Check if requester is authorized
        if !self.is_authorized_triggerer(requester_chain_id).await {
            return Err(format!("Chain {} is not authorized to trigger aggregation", requester_chain_id));
        }
        
        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        let current_time = self.runtime.system_time().micros();
        
        // 🚀 ROBUST: Multi-layer cooldown checks
        let cooldown_until = *leaderboard.trigger_cooldown_until.get();
        let last_trigger_time = *leaderboard.last_trigger_time.get();
        
        // Check explicit cooldown
        if current_time < cooldown_until {
            let _remaining_seconds = (cooldown_until - current_time) / 1_000_000;
            // Don't error, just silently ignore (prevents error spam)
            return Ok(());
        }
        
        // Check minimum time between triggers (3 seconds hard minimum)
        let time_since_last = current_time.saturating_sub(last_trigger_time);
        if time_since_last < 3_000_000 {
            // Too frequent - silently ignore
            return Ok(());
        }
        
        // 🚀 CRITICAL: Set cooldown IMMEDIATELY before doing any work
        leaderboard.trigger_cooldown_until.set(current_time + 5_000_000); // 5 second cooldown
        leaderboard.last_trigger_time.set(current_time);
        leaderboard.last_trigger_by.set(requester_chain_id.to_string());
        
        // 🚀 SMART: Only trigger if there's likely new data (optional optimization)
        // Could add additional check here for last shard update times
        
        // Send trigger messages to all shards
        let shard_ids = leaderboard.shard_ids.read_front(100).await.unwrap_or_default();
        if shard_ids.is_empty() {
            return Err("No shards registered for this leaderboard".to_string());
        }
        
        for shard_id_str in shard_ids {
            if let Ok(shard_chain_id) = ChainId::from_str(&shard_id_str) {
                self.runtime
                    .prepare_message(Message::TriggerShardAggregation { 
                        timestamp: current_time  // Use current time, not passed timestamp
                    })
                    .send_to(shard_chain_id);
            }
        }
        
        // 🚀 TRACKING: Update aggregation counter for monitoring
        let rotation_counter = *leaderboard.trigger_rotation_counter.get();
        leaderboard.trigger_rotation_counter.set(rotation_counter + 1);
        
        Ok(())
    }
    
    /// 🚀 NEW: Emit game creation event helper
    pub async fn emit_game_creation_event(&mut self, board_id: &str, player: &str, tournament_id: &str, timestamp: u64) {
        use linera_sdk::linera_base_types::StreamName;
        
        // Get current best score for this player in this tournament
        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        let current_best = leaderboard.score.get(player).await.unwrap().unwrap_or(0);
        
        let score_event = GameEvent::PlayerScoreUpdate {
            player: player.to_string(),
            board_id: board_id.to_string(),
            score: 0, // Initial score is 0
            chain_id: self.runtime.chain_id().to_string(),
            timestamp,
            game_status: GameStatus::Created,
            highest_tile: 2, // Initial highest tile
            moves_count: 0,
            leaderboard_id: tournament_id.to_string(),
            current_leaderboard_best: current_best,
        };
        
        let stream_name = StreamName::from("player_score_update".to_string());
        self.runtime.emit(stream_name, &score_event);
    }
}
