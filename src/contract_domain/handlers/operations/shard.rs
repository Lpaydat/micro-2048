//! Shard Operations Handler
//!
//! Handles shard-related operations including score aggregation, activity tracking, and workload management.

use crate::state::LeaderboardShard;
use game2048::{GameStatus, Message, PlayerScoreSummary};
use linera_sdk::linera_base_types::ChainId;
use std::str::FromStr;

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
            let start_limit = if start_time_raw == 0 {
                None
            } else {
                Some(start_time_raw)
            };
            let end_limit = if end_time_raw == 0 {
                None
            } else {
                Some(end_time_raw)
            };

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
        let shard = Self::is_shard_active(contract, timestamp).await;
        let player_shard_score = shard.score.get(player).await.unwrap();

        if player_shard_score.is_none() || player_shard_score < Some(score) {
            shard.score.insert(player, score).unwrap();
            shard.board_ids.insert(player, board_id).unwrap();
            shard.highest_tiles.insert(player, highest_tile).unwrap();
            shard.game_statuses.insert(player, game_status).unwrap();
            shard.counter.set(*shard.counter.get() + 1);
        }

        // Store the player name → chain ID mapping for later aggregation
        shard
            .player_chain_ids
            .insert(player, player_chain_id.clone())
            .unwrap();

        // 🚀 NEW: Store board count for distributed counting (tournament_id:player_chain_id -> board_count)
        let tournament_player_key = format!("{}:{}", leaderboard_id, player_chain_id);
        shard
            .tournament_player_board_counts
            .insert(&tournament_player_key, boards_in_tournament)
            .unwrap();

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

        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
        let leaderboard_id = shard.leaderboard_id.get().clone();

        // Collect all player names first
        let mut player_names = Vec::new();
        shard
            .score
            .for_each_index_while(|player| {
                player_names.push(player);
                Ok(true)
            })
            .await
            .unwrap();

        // Process each player from cache
        for player in player_names {
            if let Some(score) = shard.score.get(&player).await.unwrap() {
                let board_id = shard
                    .board_ids
                    .get(&player)
                    .await
                    .unwrap()
                    .unwrap_or_default();

                // Get the actual player chain ID from stored mapping
                let player_chain_id = shard
                    .player_chain_ids
                    .get(&player)
                    .await
                    .unwrap()
                    .unwrap_or_else(|| format!("unknown_{}", player));

                // Get board count for this player in this tournament
                let tournament_player_key = format!("{}:{}", leaderboard_id, player_chain_id);
                let board_count = shard
                    .tournament_player_board_counts
                    .get(&tournament_player_key)
                    .await
                    .unwrap()
                    .unwrap_or(0);

                // Get stored highest_tile and game_status
                let highest_tile = shard
                    .highest_tiles
                    .get(&player)
                    .await
                    .unwrap()
                    .unwrap_or_else(|| (score / 10).max(2)); // Fallback to estimate if not stored
                let game_status = shard
                    .game_statuses
                    .get(&player)
                    .await
                    .unwrap()
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
            }
        }

        // If we found any scores, emit a shard aggregation event
        if !player_summaries.is_empty() {
            let shard = contract.state.shards.load_entry_mut("").await.unwrap();
            let leaderboard_id = shard.leaderboard_id.get().clone();

            // Activity scores removed for MVP simplicity

            // Build player board counts map for this tournament
            let mut player_board_counts = std::collections::HashMap::new();
            for (_player, summary) in player_summaries.iter() {
                // Extract player chain ID and use their board count
                player_board_counts.insert(summary.chain_id.clone(), summary.boards_in_tournament);
            }

            use crate::contract_domain::events::emitters::EventEmitter;
            let chain_id = contract.runtime.chain_id().to_string();
            let timestamp = contract.runtime.system_time().micros();
            EventEmitter::emit_shard_score_update(
                contract,
                chain_id,
                player_summaries.clone(),
                std::collections::HashMap::new(), // Empty for MVP simplicity
                player_board_counts, // Board counts for distributed counting
                timestamp,
                player_summaries.len() as u32,
                leaderboard_id,
            ).await;
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
                    shard
                        .board_ids
                        .insert(player, summary.board_id.clone())
                        .unwrap();

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
                shard
                    .total_games_count
                    .set(current_games + shard_updated_players);
            }
        }
    }

    /// Register player with shard and update workload tracking
    pub async fn register_player_with_shard(
        contract: &mut crate::Game2048Contract,
        player_chain_id: String,
        tournament_id: String,
        _player_name: String,
    ) {
        // Collect data we need from shard first (to avoid borrow conflicts)
        let (_is_first_player, player_count, registered_players) = {
            let shard = contract.state.shards.load_entry_mut("").await.unwrap();

            // Check if this is the right tournament
            if shard.leaderboard_id.get() != &tournament_id {
                return; // Wrong tournament, exit early
            }

            // Check if this is the first player for this shard
            let is_first_player_in_shard = shard.active_players_count.get() == &0;

            // Add to monitoring list
            shard
                .monitored_player_chains
                .push_back(player_chain_id.clone());

            // Update workload statistics
            shard
                .active_players_count
                .set(*shard.active_players_count.get() + 1);
            let current_time = contract.runtime.system_time().micros();
            shard.last_activity.set(current_time);

            let player_count = *shard.active_players_count.get();
            
            // Get all registered players for this shard
            let registered_players = shard
                .monitored_player_chains
                .read_front(100) // Get up to 100 players
                .await
                .unwrap_or_default();

            (is_first_player_in_shard, player_count, registered_players)
        }; // Shard borrow is dropped here

        // Now we can use contract freely

        // Subscribe to this player chain's events
        if let Ok(chain_id) = ChainId::from_str(&player_chain_id) {
            contract.subscribe_to_player_score_events(chain_id);
        }

        // 🚀 DYNAMIC: Send triggerer updates only until threshold reached
        // Calculate triggerers_per_shard from tournament config
        let triggerers_per_shard = {
            let shard = contract.state.shards.load_entry_mut("").await.unwrap();
            let base_count = *shard.base_triggerer_count.get();
            let shard_count = *shard.total_shard_count.get();
            
            // If config not yet set (race condition: player registers before CreateLeaderboard processed)
            // Use safe default: 20 (covers 5 triggerers * 4 backup ratio)
            if base_count == 0 || shard_count == 0 {
                20
            } else {
                ((base_count + shard_count - 1) / shard_count).max(1) // Ceiling division
            }
        };
        
        if let Ok(leaderboard_chain_id) = ChainId::from_str(&tournament_id) {
            // Send updates only until we reach triggerers_per_shard
            // This is dynamic based on base_triggerer_count / total_shard_count
            if player_count <= triggerers_per_shard {
                let shard_chain_id = contract.runtime.chain_id().to_string();
                
                // Send ALL registered players so far
                // Leaderboard calculates triggerers_per_shard and selects top N
                contract
                    .runtime
                    .prepare_message(Message::UpdateShardTriggerCandidates {
                        shard_chain_id,
                        player_chain_ids: registered_players,
                        tournament_id: tournament_id.clone(),
                    })
                    .send_to(leaderboard_chain_id);
            }
            // After threshold: Stop sending, we have enough triggerers for this shard
        }
    }

    /// Update game count when games are created/ended
    pub async fn track_game_activity(contract: &mut crate::Game2048Contract) {
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
        shard
            .total_games_count
            .set(*shard.total_games_count.get() + 1);
        shard
            .last_activity
            .set(contract.runtime.system_time().micros());
    }
}
