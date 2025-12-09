//! Shard Operations Handler
//!
//! Handles shard-related operations including score aggregation and activity tracking.

use game2048::{ActiveBoardSummary, GameStatus, PlayerScoreSummary};
use linera_sdk::linera_base_types::ChainId;

pub struct ShardOperationHandler;

impl ShardOperationHandler {
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

        // Group active boards per player for summary propagation
        let mut active_boards_by_player: HashMap<String, Vec<ActiveBoardSummary>> = HashMap::new();
        shard
            .active_boards
            .for_each_index_value(|board_id, info| {
                if !info.is_ended {
                    active_boards_by_player
                        .entry(info.player.clone())
                        .or_default()
                        .push(ActiveBoardSummary {
                            board_id: board_id.clone(),
                            player: info.player.clone(),
                            score: info.score,
                            is_ended: info.is_ended,
                        });
                }
                Ok(())
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

                let active_boards = active_boards_by_player.remove(&player).unwrap_or_default();

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
                    active_boards,
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
                player_board_counts,              // Board counts for distributed counting
                timestamp,
                player_summaries.len() as u32,
                leaderboard_id,
            )
            .await;
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
