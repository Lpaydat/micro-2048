//! Shard Operations Handler
//! 
//! Handles shard-related operations including score aggregation, activity tracking, and workload management.

use std::str::FromStr;
use linera_sdk::linera_base_types::ChainId;
use game2048::{GameEvent, GameStatus, PlayerScoreSummary, Message};
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

        // Apply consistent validation to all chains with optional time limits
        // Keep bypass for system operations (111970) - used for game ending without moves
        if timestamp != 111970 {
            let start_time_raw = *start_time;
            let end_time_raw = *end_time;
            
            // Only validate if times are set (non-zero)
            let start_limit = if start_time_raw == 0 { None } else { Some(start_time_raw) };
            let end_limit = if end_time_raw == 0 { None } else { Some(end_time_raw) };
            
            let mut invalid = false;
            if let Some(start) = start_limit {
                if timestamp < start {
                    invalid = true;
                }
            }
            if let Some(end) = end_limit {
                if timestamp > end {
                    invalid = true;
                }
            }
            
            if invalid {
                panic!("Shard is not active for timestamp {}", timestamp);
            }
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
        player_chain_id: String,
        boards_in_tournament: u32,
        leaderboard_id: String,
        game_status: GameStatus,
        highest_tile: u64,
    ) {
        log::info!("📊 SHARD_UPDATE: Updating shard score via streaming - Player: '{}', Score: {}, Board: {}", 
            player, score, board_id);
        
        let shard = Self::is_shard_active(contract, timestamp).await;
        let player_shard_score = shard.score.get(player).await.unwrap();

        if player_shard_score.is_none() || player_shard_score < Some(score) {
            log::info!("📊 SHARD_UPDATE: Score improvement detected - Old: {:?}, New: {}", player_shard_score, score);
            shard.score.insert(player, score).unwrap();
            shard.board_ids.insert(player, board_id).unwrap();
            shard.highest_tiles.insert(player, highest_tile).unwrap();
            shard.game_statuses.insert(player, game_status).unwrap();
            shard.counter.set(*shard.counter.get() + 1);
            log::info!("📊 SHARD_UPDATE: ✅ Shard state updated via streaming system - Counter: {}", *shard.counter.get());
        } else {
            log::info!("📊 SHARD_UPDATE: Score not improved - keeping existing score {:?}", player_shard_score);
        }
        
        // Store the player name → chain ID mapping for later aggregation
        shard.player_chain_ids.insert(player, player_chain_id.clone()).unwrap();
        
        // 🚀 NEW: Store board count for distributed counting (tournament_id:player_chain_id -> board_count)
        let tournament_player_key = format!("{}:{}", leaderboard_id, player_chain_id);
        shard.tournament_player_board_counts.insert(&tournament_player_key, boards_in_tournament).unwrap();
        log::info!("📊 SHARD_UPDATE: Updated board count for player '{}' in tournament '{}': {} boards", 
                  player, leaderboard_id, boards_in_tournament);
        
        // Note: Activity tracking removed for MVP simplicity
    }

    /// Aggregate scores from player chains with smart activity tracking
    pub async fn aggregate_scores_from_player_chains(
        contract: &mut crate::Game2048Contract,
        _player_chain_ids: Vec<ChainId>, // Now reads from cache instead
    ) {
        use std::collections::HashMap;
        
        let mut player_summaries: HashMap<String, PlayerScoreSummary> = HashMap::new();
        let current_time = contract.runtime.system_time().micros();
        
        // 🚀 FIXED: Read from shard cache instead of broken event reading
        log::info!("📊 SHARD_TRIGGER: Aggregating from shard cache (triggered by leaderboard)");
        
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
        let leaderboard_id = shard.leaderboard_id.get().clone();
        

        
        // Collect all player names first
        let mut player_names = Vec::new();
        shard.score.for_each_index_while(|player| {
            player_names.push(player);
            Ok(true)
        }).await.unwrap();
        
        // Process each player from cache
        for player in player_names {
            if let Some(score) = shard.score.get(&player).await.unwrap() {
                let board_id = shard.board_ids.get(&player).await.unwrap().unwrap_or_default();
                
                // Get the actual player chain ID from stored mapping
                let player_chain_id = shard.player_chain_ids.get(&player).await.unwrap()
                    .unwrap_or_else(|| format!("unknown_{}", player));
                
                // Get board count for this player in this tournament
                let tournament_player_key = format!("{}:{}", leaderboard_id, player_chain_id);
                let board_count = shard.tournament_player_board_counts
                    .get(&tournament_player_key).await.unwrap().unwrap_or(0);
                
                // Get stored highest_tile and game_status
                let highest_tile = shard.highest_tiles.get(&player).await.unwrap()
                    .unwrap_or_else(|| (score / 10).max(2)); // Fallback to estimate if not stored
                let game_status = shard.game_statuses.get(&player).await.unwrap()
                    .unwrap_or(GameStatus::Active); // Fallback to Active if not stored
                
                // Create summary from cached data
                let summary = PlayerScoreSummary {
                    player: player.clone(),
                    best_score: score,
                    board_id,
                    chain_id: player_chain_id,
                    highest_tile,
                    last_update: current_time,
                    game_status,
                    boards_in_tournament: board_count,
                };
                
                player_summaries.insert(player.clone(), summary);
                log::info!("📊 CACHE_READ: Player '{}' has cached score: {}", player, score);
            }
        }
        
        log::info!("📊 SHARD_CACHE: Read {} players from shard cache for aggregation", player_summaries.len());
        
        // If we found any scores, emit a shard aggregation event
        if !player_summaries.is_empty() {
            let shard = contract.state.shards.load_entry_mut("").await.unwrap();
            let leaderboard_id = shard.leaderboard_id.get().clone();
            
            // Activity scores removed for MVP simplicity
            
            use linera_sdk::linera_base_types::StreamName;
            let stream_name = StreamName::from("shard_score_update".to_string());
            // Build player board counts map for this tournament
            let mut player_board_counts = std::collections::HashMap::new();
            for (_player, summary) in player_summaries.iter() {
                // Extract player chain ID and use their board count
                player_board_counts.insert(summary.chain_id.clone(), summary.boards_in_tournament);
            }
            
            let aggregation_event = GameEvent::ShardScoreUpdate {
                shard_chain_id: contract.runtime.chain_id().to_string(),
                player_scores: player_summaries.clone(),
                player_activity_scores: std::collections::HashMap::new(), // Empty for MVP simplicity
                player_board_counts, // Board counts for distributed counting
                aggregation_timestamp: contract.runtime.system_time().micros(),
                total_players: player_summaries.len() as u32,
                leaderboard_id,
            };
            
            log::info!("📡 EMIT: Emitting shard_score_update event with {} player scores from chain {}", 
                player_summaries.len(), contract.runtime.chain_id());
            contract.runtime.emit(stream_name, &aggregation_event);
            log::info!("📡 EMIT: ✅ Successfully emitted shard_score_update event");
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
            // Check if this is the first player for this shard
            let is_first_player_in_shard = shard.active_players_count.get() == &0;
            
            // Add to monitoring list
            shard.monitored_player_chains.push_back(player_chain_id.clone());
            
            // Update workload statistics
            shard.active_players_count.set(*shard.active_players_count.get() + 1);
            let current_time = contract.runtime.system_time().micros();
            shard.last_activity.set(current_time);
            
            // Activity tracking removed for MVP simplicity
            
            // Subscribe to this player chain's events
            if let Ok(chain_id) = ChainId::from_str(&player_chain_id) {
                log::info!("🎯 REGISTER_PLAYER: Setting up subscription to player chain {} for tournament {}", player_chain_id, tournament_id);
                log::info!("🎯 REGISTER_PLAYER: Shard will receive player_score_update events via process_streams");
                contract.subscribe_to_player_score_events(chain_id);
                log::info!("🎯 REGISTER_PLAYER: ✅ Successfully subscribed to player chain {}", player_chain_id);
            } else {
                log::error!("🎯 REGISTER_PLAYER: ❌ Failed to parse player_chain_id: {}", player_chain_id);
            }
            
            // 🚀 NEW: If this is the first player in this shard, register as potential triggerer
            if is_first_player_in_shard {
                if let Ok(leaderboard_chain_id) = ChainId::from_str(&tournament_id) {
                    let shard_chain_id = contract.runtime.chain_id().to_string();
                    log::info!("🎯 FIRST_PLAYER: Shard {} registering first player {} with leaderboard", shard_chain_id, player_chain_id);
                    
                    // Send message to leaderboard to register this player chain as potential triggerer
                    contract.runtime
                        .prepare_message(Message::RegisterFirstPlayer {
                            shard_chain_id,
                            player_chain_id: player_chain_id.clone(),
                            tournament_id: tournament_id.clone(),
                        })
                        .send_to(leaderboard_chain_id);
                        
                    log::info!("🎯 FIRST_PLAYER: ✅ Sent first player registration to leaderboard");
                } else {
                    log::error!("🎯 FIRST_PLAYER: ❌ Invalid tournament_id for leaderboard chain: {}", tournament_id);
                }
            }
        }
    }
    
    /// Update game count when games are created/ended
    pub async fn track_game_activity(contract: &mut crate::Game2048Contract) {
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
        shard.total_games_count.set(*shard.total_games_count.get() + 1);
        shard.last_activity.set(contract.runtime.system_time().micros());
    }
}
