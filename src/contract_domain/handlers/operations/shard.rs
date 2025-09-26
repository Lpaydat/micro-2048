//! Shard Operations Handler
//! 
//! Handles shard-related operations including score aggregation, activity tracking, and workload management.

use std::str::FromStr;
use linera_sdk::linera_base_types::ChainId;
use game2048::{GameEvent, GameStatus, GameEndReason, PlayerScoreSummary};
use crate::state::LeaderboardShard;

pub struct ShardOperationHandler;

impl ShardOperationHandler {
    /// Check if shard is active for the given timestamp
    pub async fn is_shard_active(
        contract: &mut crate::Game2048Contract,
        timestamp: u64,
    ) -> &mut LeaderboardShard {
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
        let start_time = shard.start_time.get();
        let end_time = shard.end_time.get();

        // Basic bounds checking: prevent obviously invalid timestamps
        if timestamp > u64::MAX / 2 {
            panic!("Timestamp too large");
        }

        // Apply consistent validation to all chains
        // Keep bypass for system operations (111970) - used for game ending without moves
        if timestamp != 111970
            && (timestamp < *start_time || timestamp > *end_time)
        {
            panic!("Shard is not active");
        }

        shard
    }

    /// Update shard score for a player
    pub async fn update_shard_score(
        contract: &mut crate::Game2048Contract,
        player: &str,
        board_id: String,
        score: u64,
        timestamp: u64,
    ) {
        let shard = Self::is_shard_active(contract, timestamp).await;
        let player_shard_score = shard.score.get(player).await.unwrap();

        if player_shard_score.is_none() || player_shard_score < Some(score) {
            shard.score.insert(player, score).unwrap();
            shard.board_ids.insert(player, board_id).unwrap();
            shard.counter.set(*shard.counter.get() + 1);
        }
    }

    /// Aggregate scores from player chains with smart activity tracking
    pub async fn aggregate_scores_from_player_chains(
        contract: &mut crate::Game2048Contract,
        player_chain_ids: Vec<ChainId>,
    ) {
        use std::collections::HashMap;
        
        let mut player_summaries: HashMap<String, PlayerScoreSummary> = HashMap::new();
        let current_time = contract.runtime.system_time().micros();
        
        // Process each player chain with smart activity-based reading
        for chain_id in player_chain_ids.iter() {
            let chain_id_str = chain_id.to_string();
            
            // Check if we should read this player this round
            if !Self::should_read_player_chain(contract, &chain_id_str, current_time).await {
                continue; // Skip this player this round
            }
            
            // Get last processed index for this chain
            let last_processed_index = contract.state
                .player_score_event_indices
                .get(&chain_id_str)
                .await
                .unwrap()
                .unwrap_or(0);
            
            // Read ascending from last index until error (blockchain-style)
            let mut current_index = last_processed_index;
            
            // Read until we hit error (no more events)
            #[allow(clippy::while_let_loop)]
            loop {
                if let Some(event) = contract.read_player_score_event_from_chain(*chain_id, current_index as u32) {
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
                            // Smart filtering: Only process if score is an improvement
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
                                    // Only keep latest score per player
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
            
            // Update index tracking: save our progress
            if current_index > last_processed_index {
                contract.state
                    .player_score_event_indices
                    .insert(&chain_id_str, current_index)
                    .unwrap();
                
                // Update player activity (found new events)
                Self::update_player_activity(contract, &chain_id_str, current_time, true).await;
            } else {
                // Update player activity (no new events)
                Self::update_player_activity(contract, &chain_id_str, current_time, false).await;
            }
        }
        
        // If we found any scores, emit a shard aggregation event
        if !player_summaries.is_empty() {
            let shard = contract.state.shards.load_entry_mut("").await.unwrap();
            let leaderboard_id = shard.leaderboard_id.get().clone();
            
            use linera_sdk::linera_base_types::StreamName;
            let stream_name = StreamName::from("shard_score_update".to_string());
            let aggregation_event = GameEvent::ShardScoreUpdate {
                shard_chain_id: contract.runtime.chain_id().to_string(),
                player_scores: player_summaries.clone(),
                aggregation_timestamp: contract.runtime.system_time().micros(),
                total_players: player_summaries.len() as u32,
                leaderboard_id,
            };
            
            contract.runtime.emit(stream_name, &aggregation_event);
        }
        
        // Update local shard state with comprehensive tracking
        if !player_summaries.is_empty() {
            let shard = contract.state.shards.load_entry_mut("").await.unwrap();
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

    /// Emit current shard workload
    pub async fn emit_shard_workload(contract: &mut crate::Game2048Contract) {
        use linera_sdk::linera_base_types::StreamName;
        
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
        let tournament_id = shard.leaderboard_id.get().clone();
        
        if !tournament_id.is_empty() {
            // Use the new tracking fields
            let total_players = *shard.active_players_count.get();
            let _total_games = *shard.total_games_count.get();
            let last_activity = *shard.last_activity.get();
            
            // Calculate active players in last 5 minutes (300,000 microseconds)
            let current_time = contract.runtime.system_time().micros();
            let five_minutes_ago = current_time.saturating_sub(300_000_000);
            let active_players_last_5min = if last_activity >= five_minutes_ago {
                (total_players as f32 * 0.8) as u32 // 80% if recent activity
            } else {
                (total_players as f32 * 0.2) as u32 // 20% if stale
            };
            
            let workload_event = GameEvent::ShardWorkload {
                shard_chain_id: contract.runtime.chain_id().to_string(),
                tournament_id,
                total_players,
                active_players_last_5min,
                timestamp: current_time,
            };
            
            let stream_name = StreamName::from("shard_workload".to_string());
            contract.runtime.emit(stream_name, &workload_event);
        }
    }

    /// Register player with shard and update workload tracking
    pub async fn register_player_with_shard(
        contract: &mut crate::Game2048Contract,
        player_chain_id: String,
        tournament_id: String,
        _player_name: String,
    ) {
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
        
        // Check if this is the right tournament
        if shard.leaderboard_id.get() == &tournament_id {
            // Add to monitoring list
            shard.monitored_player_chains.push_back(player_chain_id.clone());
            
            // Update workload statistics
            shard.active_players_count.set(*shard.active_players_count.get() + 1);
            let current_time = contract.runtime.system_time().micros();
            shard.last_activity.set(current_time);
            
            // Initialize smart activity tracking for new player
            shard.player_activity_levels.insert(&player_chain_id, 0).unwrap(); // Start as very_active
            shard.player_read_intervals.insert(&player_chain_id, 1).unwrap(); // Read every round initially
            shard.player_last_seen.insert(&player_chain_id, current_time).unwrap();
            
            // Subscribe to this player chain's events
            if let Ok(chain_id) = ChainId::from_str(&player_chain_id) {
                contract.subscribe_to_player_score_events(chain_id);
            }
        }
    }
    
    /// Update game count when games are created/ended
    pub async fn track_game_activity(contract: &mut crate::Game2048Contract) {
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
        shard.total_games_count.set(*shard.total_games_count.get() + 1);
        shard.last_activity.set(contract.runtime.system_time().micros());
    }

    /// Smart algorithm to decide if we should read a player chain this round
    async fn should_read_player_chain(
        contract: &mut crate::Game2048Contract,
        chain_id_str: &str,
        current_time: u64,
    ) -> bool {
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
        
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

    /// Update player activity level based on event presence
    async fn update_player_activity(
        contract: &mut crate::Game2048Contract,
        chain_id_str: &str,
        current_time: u64,
        found_new_events: bool,
    ) {
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
        
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
}