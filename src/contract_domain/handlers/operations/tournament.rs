//! Tournament Operations Handler
//! 
//! Handles tournament-related operations including validation and shard selection.

use std::str::FromStr;
use linera_sdk::linera_base_types::ChainId;
use game2048::{GameEvent, TournamentStatus};

pub struct TournamentOperationHandler;

impl TournamentOperationHandler {
    /// Validate tournament exists and is active
    pub async fn validate_tournament(
        contract: &mut crate::Game2048Contract,
        tournament_id: &str,
        leaderboard_chain_id: ChainId,
    ) -> bool {
        if let Some(GameEvent::ActiveTournaments { tournaments, .. }) = Self::read_active_tournaments(contract, leaderboard_chain_id).await {
            return tournaments.iter().any(|t| 
                t.tournament_id == tournament_id && 
                matches!(t.status, TournamentStatus::Active)
            );
        }
        false
    }

    /// Select optimal shard for a tournament based on workload
    pub async fn select_optimal_shard(
        contract: &mut crate::Game2048Contract,
        tournament_id: &str,
        leaderboard_chain_id: ChainId,
    ) -> String {
        // Get tournament info to find available shards
        if let Some(GameEvent::ActiveTournaments { tournaments, .. }) = Self::read_active_tournaments(contract, leaderboard_chain_id).await {
            if let Some(tournament) = tournaments.iter().find(|t| t.tournament_id == tournament_id) {
                // Read workload from each shard and select the least loaded
                let mut best_shard = tournament.shard_chain_ids.first().cloned().unwrap_or_else(|| contract.runtime.chain_id().to_string());
                let mut lowest_load = u32::MAX;
                
                for shard_chain_id_str in tournament.shard_chain_ids.iter() {
                    if let Ok(shard_chain_id) = ChainId::from_str(shard_chain_id_str) {
                        if let Some(GameEvent::ShardWorkload { 
                            active_players_last_5min, 
                            total_players,
                            .. 
                        }) = Self::read_shard_workload(contract, shard_chain_id).await {
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
        let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
        match leaderboard.shard_ids.read_front(1).await {
            Ok(shard_ids) => {
                if let Some(first_shard_id) = shard_ids.first() {
                    first_shard_id.clone()
                } else {
                    // Ultimate fallback if no shards registered
                    contract.runtime.chain_id().to_string() 
                }
            }
            Err(_) => {
                // Error fallback - use current chain as shard
                contract.runtime.chain_id().to_string()
            }
        }
    }

    /// Read latest active tournaments - ascends until error (blockchain-style)
    pub async fn read_active_tournaments(
        contract: &mut crate::Game2048Contract,
        leaderboard_chain_id: ChainId,
    ) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("active_tournaments".to_string());
        
        // Get last processed index
        let mut current_index = *contract.state.active_tournaments_event_index.get();
        let mut latest_event: Option<GameEvent> = None;
        
        // Read ascending until error (latest has highest index)
        loop {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                contract.runtime.read_event(leaderboard_chain_id, stream_name.clone(), current_index as u32)
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
        
        // Update state with the latest index we successfully read
        if latest_event.is_some() {
            contract.state.active_tournaments_event_index.set(current_index);
        }
        
        latest_event
    }

    /// Read latest shard workload - ascends until error (blockchain-style)
    pub async fn read_shard_workload(
        contract: &mut crate::Game2048Contract,
        shard_chain_id: ChainId,
    ) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("shard_workload".to_string());
        let chain_id_str = shard_chain_id.to_string();
        
        // Get last processed index for this shard
        let mut current_index = contract.state
            .shard_workload_event_indices
            .get(&chain_id_str)
            .await
            .unwrap()
            .unwrap_or(0);
        
        let mut latest_event: Option<GameEvent> = None;
        
        // Read ascending until error (latest has highest index)
        loop {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                contract.runtime.read_event(shard_chain_id, stream_name.clone(), current_index as u32)
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
        
        // Update state with the latest index we successfully read
        if latest_event.is_some() {
            contract.state
                .shard_workload_event_indices
                .insert(&chain_id_str, current_index)
                .unwrap();
        }
        
        latest_event
    }
}