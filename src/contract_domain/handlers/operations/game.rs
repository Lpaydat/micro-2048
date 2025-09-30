//! Game Operations Handler
//!
//! Handles game-related operations including moves and board creation.

use crate::contract_domain::game_logic::{GameMoveProcessor, GameMoveResult};
use game2048::{hash_seed, Direction, Game, GameEndReason, GameStatus, Message};
use linera_sdk::linera_base_types::ChainId;
use std::str::FromStr;

pub struct GameOperationHandler;

impl GameOperationHandler {
    pub async fn handle_make_moves(
        contract: &mut crate::Game2048Contract,
        board_id: String,
        moves: String,
        player: String,
        password_hash: String,
    ) {
        // Validate password
        contract
            .validate_player_password(&player, &password_hash)
            .await;
        let board = contract
            .state
            .boards
            .load_entry_mut(&board_id)
            .await
            .unwrap();
        let shard_id = board.shard_id.get().clone();

        if player != *board.player.get() {
            panic!("You can only make move on your own board");
        }

        type MoveInput = (Direction, String);
        let moves: Vec<MoveInput> =
            serde_json::from_str(&moves).unwrap_or_else(|_| panic!("Invalid moves format"));

        let is_ended = *board.is_ended.get();
        let end_time_raw = *board.end_time.get();
        // Convert 0 or u64::MAX to None (unlimited), otherwise Some(value)
        let end_time = if end_time_raw == 0 || end_time_raw == u64::MAX {
            None
        } else {
            Some(end_time_raw)
        };

        if !is_ended && !moves.is_empty() {
            let initial_board = *board.board.get();

            // FIXED: Convert string timestamps to u64 with error handling
            let mut moves_u64: Vec<(Direction, u64)> = Vec::new();
            for (dir, ts) in moves {
                match ts.parse::<u64>() {
                    Ok(timestamp) => {
                        // Additional validation: ensure timestamp is reasonable
                        if timestamp > 0 && timestamp < 1_000_000_000_000_000 { // Max ~31 years in microseconds
                            moves_u64.push((dir, timestamp));
                        } else {
                            // Use current time if timestamp is invalid
                            let current_time = contract.runtime.system_time().micros();
                            moves_u64.push((dir, current_time));
                        }
                    }
                    Err(_) => {
                        // FIXED: Use current system time instead of panicking
                        let current_time = contract.runtime.system_time().micros();
                        moves_u64.push((dir, current_time));
                    }
                }
            }

            match GameMoveProcessor::process_moves(
                &board_id,
                &player,
                &moves_u64,
                initial_board,
                end_time,
            ) {
                GameMoveResult::Success {
                    final_board,
                    final_score,
                    final_highest_tile,
                    initial_highest_tile,
                    is_ended,
                    latest_timestamp,
                } => {
                    // Update board state
                    board.board.set(final_board);
                    board.score.set(final_score);
                    if is_ended {
                        board.is_ended.set(true);
                    }

                    // 🚀 NEW: Always emit score update on every score change!
                    let game_status = if is_ended {
                        // Game ends only when no moves available (board is full)
                        // Players can continue indefinitely for higher scores (2048 -> 4096 -> 8192 -> ...)
                        GameStatus::Ended(GameEndReason::NoMoves)
                    } else {
                        GameStatus::Active
                    };

                    let leaderboard = contract
                        .state
                        .leaderboards
                        .load_entry_mut("")
                        .await
                        .unwrap();
                    let leaderboard_id = leaderboard.leaderboard_id.get().clone();

                    // Get current best score for this player in this leaderboard
                    let current_best = leaderboard.score.get(&player).await.unwrap().unwrap_or(0);

                    // Get player's current board count for this tournament
                    let player_state = contract
                        .state
                        .players
                        .load_entry_mut(&player)
                        .await
                        .unwrap();
                    let current_board_count = player_state
                        .boards_per_tournament
                        .get(&leaderboard_id)
                        .await
                        .unwrap()
                        .unwrap_or(0);

                    use crate::contract_domain::events::emitters::EventEmitter;
                    let chain_id = contract.runtime.chain_id().to_string();
                    EventEmitter::emit_player_score_update(
                        contract,
                        player.clone(),
                        board_id.clone(),
                        final_score,
                        chain_id,
                        latest_timestamp,
                        game_status,
                        final_highest_tile,
                        0, // TODO: Track actual move count
                        leaderboard_id.clone(),
                        current_best,
                        current_board_count,
                    ).await;

                    // 🚀 NEW: Update shard workload when scores change significantly
                    let score_improvement = final_score.saturating_sub(current_best);
                    if score_improvement > 2000 || is_ended {
                        // Temporarily commented out to isolate move processing from system_time() issues

                        // 🚀 REMOVED: Dynamic update happens in leaderboard update instead
                    }

                    // Update player record for significant improvements
                    let player_record = contract
                        .state
                        .player_records
                        .load_entry_mut(&player)
                        .await
                        .unwrap();
                    let prev_score = player_record
                        .best_score
                        .get(&shard_id)
                        .await
                        .unwrap()
                        .unwrap_or(0);

                    let score_threshold = prev_score + 1000;
                    if final_score > score_threshold
                        || final_highest_tile > initial_highest_tile
                        || is_ended
                    {
                        player_record
                            .best_score
                            .insert(&shard_id, final_score)
                            .unwrap();
                    }
                }
                GameMoveResult::Error(msg) => {
                    panic!("{}", msg);
                }
            }
        } else if moves.is_empty() {
            let score = Game::score(*board.board.get());

            // 🚀 NEW: Emit player score update for tournament end
            // This case handles when tournament time expires and game ends
            let leaderboard = contract
                .state
                .leaderboards
                .load_entry_mut("")
                .await
                .unwrap();
            let leaderboard_id = leaderboard.leaderboard_id.get().clone();

            // Get current best score for this player in this leaderboard
            let current_best = leaderboard.score.get(&player).await.unwrap().unwrap_or(0);

            // Get player's current board count for this tournament
            let player_state = contract
                .state
                .players
                .load_entry_mut(&player)
                .await
                .unwrap();
            let current_board_count = player_state
                .boards_per_tournament
                .get(&leaderboard_id)
                .await
                .unwrap()
                .unwrap_or(0);

            use crate::contract_domain::events::emitters::EventEmitter;
            let chain_id = contract.runtime.chain_id().to_string();
            let highest_tile = Game::highest_tile(*board.board.get());
            EventEmitter::emit_player_score_update(
                contract,
                player.clone(),
                board_id.clone(),
                score,
                chain_id,
                111970,
                GameStatus::Ended(GameEndReason::TournamentEnded),
                highest_tile,
                0, // TODO: Track actual move count
                leaderboard_id.clone(),
                current_best,
                current_board_count,
            ).await;
        } else {
            panic!("Game is ended");
        }
    }

    pub async fn handle_new_board(
        contract: &mut crate::Game2048Contract,
        player: String,
        timestamp: u64,
        password_hash: String,
        leaderboard_id: String, // Leaderboard ID parameter
    ) {
        // Validate password
        contract
            .validate_player_password(&player, &password_hash)
            .await;

        // 🚀 NEW: Show cached tournament information

        // Display cached tournaments from streaming system
        let cached_tournaments = contract.list_cached_tournaments().await;
        let _cache_count = contract.get_cached_tournament_count().await;

        if !cached_tournaments.is_empty() {
            for tournament in cached_tournaments.iter() {
                let _status = if tournament.tournament_id == leaderboard_id {
                    "🎯 TARGET"
                } else {
                    "📋"
                };
            }
        }

        // Check if target tournament is in cache
        let _target_tournament = contract.get_cached_tournament(&leaderboard_id).await;

        // 🚀 NEW: Validate tournament exists and is active
        let is_valid_tournament = contract.validate_tournament(&leaderboard_id).await;

        if !is_valid_tournament {
            panic!(
                "Tournament '{}' is not active, expired, or does not exist",
                leaderboard_id
            );
        }

        // 🚀 NEW: Get leaderboard info and select optimal shard using hash-based distribution
        let selected_shard_id = contract
            .select_optimal_shard(&leaderboard_id, &player)
            .await;

        // 🚀 FIX: Get tournament data before creating board to avoid borrow conflicts
        let tournament_end_time =
            if let Some(tournament) = contract.get_cached_tournament(&leaderboard_id).await {
                // None -> 0 (unlimited)
                tournament.end_time.unwrap_or(0)
            } else {
                0 // Default to unlimited if tournament not in cache
            };

        // 🚀 NEW: Create board locally (no cross-chain message needed)
        let nonce = contract.state.nonce.get();
        let board_id = format!(
            "{}.{}",
            contract.runtime.chain_id(),
            hash_seed(&nonce.to_string(), &player, timestamp)
        );

        let new_board = Game::new(&board_id, &player, timestamp).board;
        let game = contract
            .state
            .boards
            .load_entry_mut(&board_id)
            .await
            .unwrap();
        game.board_id.set(board_id.clone());
        game.board.set(new_board);
        game.player.set(player.clone());
        game.leaderboard_id.set(leaderboard_id.clone());
        game.shard_id.set(selected_shard_id.clone());
        game.chain_id.set(contract.runtime.chain_id().to_string());
        game.created_at.set(timestamp);
        game.end_time.set(tournament_end_time);

        contract.state.nonce.set(nonce + 1);
        contract.state.latest_board_id.set(board_id.clone());

        // 🚀 NEW: Increment player's board count for this tournament (distributed counting)
        let player_state = contract
            .state
            .players
            .load_entry_mut(&player)
            .await
            .unwrap();
        let current_board_count = player_state
            .boards_per_tournament
            .get(&leaderboard_id)
            .await
            .unwrap()
            .unwrap_or(0);
        let new_board_count = current_board_count + 1;
        player_state
            .boards_per_tournament
            .insert(&leaderboard_id, new_board_count)
            .unwrap();

        // 🚀 NEW: Register with selected shard (one-time registration)

        let registration_message = Message::RegisterPlayerWithShard {
            player_chain_id: contract.runtime.chain_id().to_string(),
            tournament_id: leaderboard_id.clone(),
            player_name: player.clone(),
        };

        if let Ok(shard_chain_id) = ChainId::from_str(&selected_shard_id) {
            contract
                .runtime
                .prepare_message(registration_message)
                .send_to(shard_chain_id);
        }

        // 🚀 NEW: Subscribe to leaderboard updates for triggerer system
        if let Ok(leaderboard_chain_id) = ChainId::from_str(&leaderboard_id) {
            contract.subscribe_to_leaderboard_update_events(leaderboard_chain_id);
        }

        // 🚀 BOOTSTRAP: Shard chains now handle first-player registration (not player chains)

        // 🚀 NEW: Emit game creation event
        contract
            .emit_game_creation_event(&board_id, &player, &leaderboard_id, timestamp)
            .await;

        // 🚀 NEW: Track activity for workload statistics
        contract.track_game_activity().await;
    }

    /// 🚀 IMPROVED: Handle score aggregation using monitored player chains from shard state
    pub async fn handle_aggregate_scores(contract: &mut crate::Game2048Contract) {
        // Get monitored player chains from shard state
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
        let mut player_chain_ids = Vec::new();

        // Collect all monitored player chain IDs from the queue
        match shard.monitored_player_chains.read_front(100).await {
            Ok(chain_id_strings) => {
                for chain_id_str in chain_id_strings {
                    if let Ok(chain_id) = ChainId::from_str(&chain_id_str) {
                        player_chain_ids.push(chain_id);
                    }
                }
            }
            Err(_) => {
                // No entries or error - proceed with empty list
            }
        }

        // Aggregate scores from monitored player chains
        contract
            .aggregate_scores_from_player_chains(player_chain_ids)
            .await;
    }

    /// 🚀 IMPROVED: Handle leaderboard update using registered shard chains from leaderboard state
    pub async fn handle_update_leaderboard(contract: &mut crate::Game2048Contract) {
        // Get registered shard chain IDs from leaderboard state
        let leaderboard = contract
            .state
            .leaderboards
            .load_entry_mut("")
            .await
            .unwrap();
        let mut shard_chain_ids = Vec::new();

        // 🚀 FIXED: Collect ALL registered shard chain IDs (no limit)
        let mut read_count = 0;
        #[allow(clippy::while_let_loop)]
        loop {
            match leaderboard.shard_ids.read_front(1000).await {
                // Large batch size
                Ok(shard_id_strings) => {
                    if shard_id_strings.is_empty() {
                        break; // No more shards to read
                    }

                    for shard_id_str in shard_id_strings {
                        if let Ok(chain_id) = ChainId::from_str(&shard_id_str) {
                            shard_chain_ids.push(chain_id);
                        }
                    }
                    read_count += 1;

                    // Safety valve - prevent infinite loops
                    if read_count > 100 {
                        break;
                    }
                }
                Err(_) => break, // Error or end of queue
            }
        }

        // Update leaderboard from registered shard chains
        contract
            .update_leaderboard_from_shard_chains(shard_chain_ids)
            .await;
    }
}
