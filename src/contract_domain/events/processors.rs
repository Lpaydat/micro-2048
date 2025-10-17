//! Stream Processors
//!
//! Logic for processing incoming stream updates and events.

use crate::contract_domain::contract_helpers::CollectionHelpers;
use crate::contract_domain::events::EventReader;
use crate::state::ActiveBoardInfo;
use linera_sdk::linera_base_types::{ChainId, StreamUpdate};
use std::collections::HashSet;
use std::str::FromStr;

/// Stream processing utilities
pub struct StreamProcessor;

impl StreamProcessor {
    /// Process all stream updates for the contract
    pub async fn process_updates(
        contract: &mut crate::Game2048Contract,
        updates: Vec<StreamUpdate>,
    ) {
        let mut processed_player_updates = false;

        for update in updates.iter() {
            // Determine which stream we're processing based on stream name
            let stream_name_bytes = &update.stream_id.stream_name.0;
            let stream_name = String::from_utf8_lossy(stream_name_bytes);

            // Process all new events in this stream update
            let event_count = update.next_index - update.previous_index;
            if event_count == 0 {
                continue;
            }

            for event_index in update.previous_index..update.next_index {
                match stream_name.as_ref() {
                    "player_score_update" => {
                        if Self::process_player_score_update(contract, update, event_index).await {
                            processed_player_updates = true;
                        }
                    }
                    "shard_score_update" => {
                        Self::process_shard_score_update(contract, update, event_index).await;
                    }
                    "active_tournaments" => {
                        Self::process_active_tournaments(contract, update, event_index).await;
                    }
                    "leaderboard_update" => {
                        Self::process_leaderboard_update(contract, update, event_index).await;
                    }
                    _ => {}
                }
            }
        }

        // After processing all streams, emit aggregated shard scores if we processed player updates
        if processed_player_updates {
            Self::emit_shard_aggregation_if_needed(contract).await;
        }
    }

    /// Process player score update events
    async fn process_player_score_update(
        contract: &mut crate::Game2048Contract,
        update: &StreamUpdate,
        event_index: u32,
    ) -> bool {
        if let Some(game2048::GameEvent::PlayerScoreUpdate {
            player,
            score,
            board_id,
            timestamp,
            game_status,
            highest_tile,
            leaderboard_id,
            boards_in_tournament,
            ..
        }) =
            EventReader::read_player_score_event_from_chain(contract, update.chain_id, event_index)
        {
            // Update shard state with the received player score
            let player_chain_id = update.chain_id.to_string();
            contract
                .update_shard_score(
                    &player,
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
            return true;
        }
        false
    }

    /// Process shard score update events
    async fn process_shard_score_update(
        contract: &mut crate::Game2048Contract,
        update: &StreamUpdate,
        event_index: u32,
    ) {
        if let Some(game2048::GameEvent::ShardScoreUpdate {
            player_scores,
            player_activity_scores,
            player_board_counts,
            leaderboard_id,
            ..
        }) =
            EventReader::read_shard_score_event_from_chain(contract, update.chain_id, event_index)
        {
            // Update leaderboard state with smart merging (real-time stream processing)
            let leaderboard = contract
                .state
                .leaderboards
                .load_entry_mut("")
                .await
                .unwrap();

            for (player, summary) in player_scores.iter() {
                let current_score = leaderboard.score.get(player).await.unwrap().unwrap_or(0);

                // Update if better score or equal score with newer timestamp
                if summary.best_score >= current_score {
                    let is_ended = matches!(summary.game_status, game2048::GameStatus::Ended(_));

                    leaderboard
                        .score
                        .insert(player, summary.best_score)
                        .unwrap();
                    leaderboard
                        .board_ids
                        .insert(player, summary.board_id.clone())
                        .unwrap();
                    leaderboard.is_ended.insert(player, is_ended).unwrap();
                }
            }

            // Track active boards reported by shards (board_id -> player, score)
            let mut new_active_board_ids = HashSet::new();
            for summary in player_scores.values() {
                for active_board in summary.active_boards.iter() {
                    if active_board.is_ended {
                        continue;
                    }
                    new_active_board_ids.insert(active_board.board_id.clone());
                    leaderboard
                        .active_boards
                        .insert(
                            &active_board.board_id,
                            ActiveBoardInfo {
                                player: active_board.player.clone(),
                                score: active_board.score,
                                is_ended: active_board.is_ended,
                            },
                        )
                        .unwrap();
                }
            }

            // Remove boards no longer active according to latest shard snapshot
            let mut existing_active_board_ids = Vec::new();
            leaderboard
                .active_boards
                .for_each_index_while(|board_id| {
                    existing_active_board_ids.push(board_id);
                    Ok(true)
                })
                .await
                .unwrap();

            for board_id in existing_active_board_ids {
                if !new_active_board_ids.contains(&board_id) {
                    let _ = leaderboard.active_boards.remove(&board_id);
                }
            }

            // Update activity scores for triggerer ranking
            let mut activity_updated_players = 0u32;
            for (player, activity_score) in player_activity_scores.iter() {
                // Always update activity score (it's time-based, not cumulative)
                leaderboard
                    .player_activity_scores
                    .insert(player, *activity_score)
                    .unwrap();
                activity_updated_players += 1;
            }

            // 🚀 MERGE board counts from this shard (accumulate, don't replace)
            for (player_chain_id, board_count) in player_board_counts.iter() {
                let current_count = leaderboard
                    .player_board_counts
                    .get(player_chain_id)
                    .await
                    .unwrap()
                    .unwrap_or(0);

                // Take the maximum (shard reports latest count for this player)
                let new_count = (*board_count).max(current_count);
                leaderboard
                    .player_board_counts
                    .insert(player_chain_id, new_count)
                    .unwrap();
            }

            // Calculate total counts from merged board counts
            let mut total_boards = 0u32;
            let mut total_players = 0u32;
            leaderboard
                .player_board_counts
                .for_each_index_value(|_player, board_count| {
                    total_boards += *board_count;
                    total_players += 1;
                    Ok(())
                })
                .await
                .unwrap();

            leaderboard.total_boards.set(total_boards);
            leaderboard.total_players.set(total_players);

            // Update triggerer list based on activity scores (not game scores)
            if activity_updated_players > 0 {
                use crate::contract_domain::handlers::messages::LeaderboardMessageHandler;
                LeaderboardMessageHandler::update_triggerer_list_by_activity(
                    contract,
                    &leaderboard_id,
                )
                .await;
            }
        }
    }

    /// Process active tournaments events
    async fn process_active_tournaments(
        contract: &mut crate::Game2048Contract,
        update: &StreamUpdate,
        event_index: u32,
    ) {
        if let Some(game2048::GameEvent::ActiveTournaments {
            tournaments,
            timestamp,
        }) = EventReader::read_active_tournaments_event_from_chain(
            contract,
            update.chain_id,
            event_index,
        ) {
            // Update local tournament cache with the new data
            Self::update_local_tournament_cache(contract, tournaments, timestamp).await;
        }
    }

    /// Process leaderboard update events
    async fn process_leaderboard_update(
        contract: &mut crate::Game2048Contract,
        update: &StreamUpdate,
        event_index: u32,
    ) {
        if let Some(game2048::GameEvent::LeaderboardUpdate {
            leaderboard_id,
            triggerer_list,
            last_update_timestamp,
            threshold_config,
            ..
        }) = EventReader::read_leaderboard_update_event_from_chain(
            contract,
            update.chain_id,
            event_index,
        ) {
            // Update local triggerer configuration
            Self::update_triggerer_config(
                contract,
                leaderboard_id,
                triggerer_list,
                last_update_timestamp,
                threshold_config,
            )
            .await;

            // 🚀 MOVED: Trigger logic moved to block production (execute_operation)
            // This ensures triggers always use the latest state and eliminates race conditions
            // Player chains now handle triggers in contract.rs during block production
        }
    }

    /// Emit shard aggregation if player updates were processed
    async fn emit_shard_aggregation_if_needed(contract: &mut crate::Game2048Contract) {
        // FIXED: Only run aggregation on SHARD chains, not player chains
        // Check if this chain has shard state (non-empty leaderboard_id indicates shard chain)
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
        let leaderboard_id = shard.leaderboard_id.get();

        // Guard: Only proceed if this is actually a shard chain
        if leaderboard_id.is_empty() {
            // This is a player chain, not a shard chain - skip aggregation
            return;
        }

        let mut player_chain_ids = Vec::new();

        // Collect all monitored player chain IDs from the queue
        if let Ok(chain_id_strings) = shard.monitored_player_chains.read_front(100).await {
            for chain_id_str in chain_id_strings {
                if let Ok(chain_id) = ChainId::from_str(&chain_id_str) {
                    player_chain_ids.push(chain_id);
                }
            }
        }

        // Emit aggregated player scores from this shard
        contract
            .aggregate_scores_from_player_chains(player_chain_ids)
            .await;
    }

    /// Update local tournament cache with latest data from main chain
    async fn update_local_tournament_cache(
        contract: &mut crate::Game2048Contract,
        tournaments: Vec<game2048::TournamentInfo>,
        timestamp: u64,
    ) {
        // Clear existing cache
        let mut keys_to_remove = Vec::new();
        contract
            .state
            .tournaments_cache_json
            .for_each_index_while(|key| {
                keys_to_remove.push(key);
                Ok(true) // Continue iteration
            })
            .await
            .unwrap();

        for key in keys_to_remove {
            contract.state.tournaments_cache_json.remove(&key).unwrap();
        }

        // Add all tournaments to cache as JSON
        for tournament in tournaments {
            let tournament_id = tournament.tournament_id.clone();
            if let Ok(tournament_json) = serde_json::to_string(&tournament) {
                contract
                    .state
                    .tournaments_cache_json
                    .insert(&tournament_id, tournament_json)
                    .unwrap();
            }
        }

        // Update timestamp
        contract.state.last_tournament_update.set(timestamp);
    }

    /// Update local triggerer configuration from leaderboard_update event
    async fn update_triggerer_config(
        contract: &mut crate::Game2048Contract,
        _leaderboard_id: String,
        triggerer_list: Vec<(String, u32)>,
        last_update_timestamp: u64,
        threshold_config: u64,
    ) {
        // Clear existing triggerer list and activity scores
        let _ = CollectionHelpers::clear_string_queue(&mut contract.state.triggerer_list).await;
        let _ =
            CollectionHelpers::clear_u32_queue(&mut contract.state.triggerer_activity_scores).await;

        // Add new triggerer list (activity-sorted)
        for (triggerer_id, activity_score) in &triggerer_list {
            contract
                .state
                .triggerer_list
                .push_back(triggerer_id.clone());
            contract
                .state
                .triggerer_activity_scores
                .push_back(*activity_score);
        }

        // Update configuration
        contract
            .state
            .triggerer_list_timestamp
            .set(last_update_timestamp);
        contract
            .state
            .trigger_threshold_config
            .set(threshold_config);
        contract
            .state
            .total_registered_players
            .set(triggerer_list.len() as u32);

        // 🚀 TIER 6: Reset emergency mode state on leaderboard update
        // (Tier 6 activation and triggering now handled in execute_operation)
        contract.state.is_in_tier6.set(false);
        contract.state.operations_since_tier6.set(0);
        contract.state.tier6_start_time.set(0);
    }


}
