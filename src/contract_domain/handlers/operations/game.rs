//! Game Operations Handler
//!
//! Handles game-related operations including moves and board creation.

use crate::contract_domain::game_logic::{GameMoveProcessor, GameMoveResult};
use game2048::{hash_seed, Direction, Game, GameEndReason, GameStatus};
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

        if player != *board.player.get() {
            panic!("You can only make move on your own board");
        }

        // 🎵 Move format: (Direction, timestamp_string, beat_number)
        // beat_number: 0 = miss/off-beat, >0 = on-beat (which beat number)
        type MoveInput = (Direction, String, u32);
        let moves: Vec<MoveInput> =
            serde_json::from_str(&moves).unwrap_or_else(|_| panic!("Invalid moves format"));

        let is_ended = *board.is_ended.get();
        let start_time_raw = *board.start_time.get();
        let end_time_raw = *board.end_time.get();
        // Convert 0 or u64::MAX to None (unlimited), otherwise Some(value)
        let start_time = if start_time_raw == 0 || start_time_raw == u64::MAX {
            None
        } else {
            Some(start_time_raw)
        };
        let end_time = if end_time_raw == 0 || end_time_raw == u64::MAX {
            None
        } else {
            Some(end_time_raw)
        };

        if !is_ended && !moves.is_empty() {
            let initial_board = *board.board.get();
            // 🔒 DUPLICATE DETECTION: Get last processed timestamp
            let last_processed_timestamp = *board.last_processed_timestamp.get();

            // FIXED: Convert string timestamps to u64 with error handling
            // 🎵 Now includes beat_number for rhythm mode
            let mut moves_u64: Vec<(Direction, u64, u32)> = Vec::new();
            for (dir, ts, beat_number) in moves {
                match ts.parse::<u64>() {
                    Ok(timestamp) => {
                        // Additional validation: ensure timestamp is reasonable
                        if timestamp > 0 && timestamp < 1_000_000_000_000_000 {
                            // Max ~31 years in microseconds
                            moves_u64.push((dir, timestamp, beat_number));
                        } else {
                            // Use current time if timestamp is invalid
                            let current_time = contract.runtime.system_time().micros();
                            moves_u64.push((dir, current_time, beat_number));
                        }
                    }
                    Err(_) => {
                        // FIXED: Use current system time instead of panicking
                        let current_time = contract.runtime.system_time().micros();
                        moves_u64.push((dir, current_time, beat_number));
                    }
                }
            }

            match GameMoveProcessor::process_moves(
                &board_id,
                &player,
                &moves_u64,
                initial_board,
                last_processed_timestamp, // 🔒 NEW: Pass for duplicate detection
                start_time,
                end_time,
            ) {
                GameMoveResult::Success {
                    final_board,
                    final_score,
                    final_highest_tile,
                    initial_highest_tile: _, // Not needed with simplified score submission
                    is_ended,
                    latest_timestamp,
                    move_history,
                } => {
                    // Update board state
                    board.board.set(final_board);
                    board.score.set(final_score);
                    if is_ended {
                        board.is_ended.set(true);
                    }
                    
                    // 🔒 DUPLICATE PREVENTION: Update last processed timestamp
                    board.last_processed_timestamp.set(latest_timestamp);

                    // 🎮 NEW: Store move history for replay feature
                    let current_move_count = *board.move_count.get();
                    for (idx, processed_move) in move_history.iter().enumerate() {
                        let move_index = current_move_count + idx as u32;
                        let move_record = board
                            .move_history
                            .load_entry_mut(&move_index)
                            .await
                            .unwrap();

                        // Convert Direction enum to u8
                        let direction_u8 = match processed_move.direction {
                            game2048::Direction::Up => 0,
                            game2048::Direction::Down => 1,
                            game2048::Direction::Left => 2,
                            game2048::Direction::Right => 3,
                        };

                        move_record.direction.set(direction_u8);
                        move_record.timestamp.set(processed_move.timestamp);
                        move_record.board_after.set(processed_move.board_after);
                        move_record.score_after.set(processed_move.score_after);
                        // 🎵 Rhythm mode: store beat number for replay
                        move_record.beat_number.set(processed_move.beat_number);
                    }
                    board
                        .move_count
                        .set(current_move_count + move_history.len() as u32);

                    // 🔒 FIX: Get tournament ID from the BOARD, not from local leaderboard
                    // The board knows which tournament it belongs to
                    let leaderboard_id = board.leaderboard_id.get().clone();

                    // 🔒 FIX: Get current best score for THIS TOURNAMENT from player_records
                    // This ensures we track per-tournament best scores, not all-time best
                    let player_record = contract
                        .state
                        .player_records
                        .load_entry_mut(&player)
                        .await
                        .unwrap();
                    let current_best = player_record
                        .best_score
                        .get(&leaderboard_id)
                        .await
                        .unwrap()
                        .unwrap_or(0);

                    // 🚀 SIMPLIFIED SCORE SUBMISSION (Manual-Only)
                    // Only auto-send on critical events:
                    // - Board ended (game over - no moves available)
                    // - Tournament ended (time expired)
                    // All other updates require manual submission via SubmitCurrentScore
                    // Condition: score > 0 AND score > tournament_best

                    let current_time = contract.runtime.system_time().micros();
                    let end_time_val = *board.end_time.get();
                    let tournament_just_ended = end_time_val > 0 && current_time >= end_time_val * 1000; // end_time is in millis, current_time in micros
                    let board_ended = is_ended;
                    
                    // Only send on game end or tournament end
                    let should_send = final_score > 0 
                        && final_score > current_best 
                        && (board_ended || tournament_just_ended);

                    if should_send {
                        // Determine game status
                        let game_status = if board_ended {
                            GameStatus::Ended(GameEndReason::NoMoves)
                        } else {
                            GameStatus::Ended(GameEndReason::TournamentEnded)
                        };

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

                        use linera_sdk::linera_base_types::ChainId;
                        use std::str::FromStr;
                        
                        // Extract values before borrowing runtime
                        let player_chain_id = contract.runtime.chain_id().to_string();
                        
                        if let Ok(leaderboard_chain_id) = ChainId::from_str(&leaderboard_id) {
                            contract
                                .runtime
                                .prepare_message(game2048::Message::SubmitScore {
                                    player: player.clone(),
                                    player_chain_id,
                                    board_id: board_id.clone(),
                                    score: final_score,
                                    highest_tile: final_highest_tile,
                                    game_status,
                                    timestamp: latest_timestamp,
                                    boards_in_tournament: current_board_count,
                                    start_time: start_time_raw,
                                    end_time: end_time_raw,
                                })
                                .send_to(leaderboard_chain_id);
                        }

                        // Update tracking state
                        let board = contract
                            .state
                            .boards
                            .load_entry_mut(&board_id)
                            .await
                            .unwrap();
                        board.highest_tile_sent.set(final_highest_tile);
                        board.last_score_sent_time.set(current_time);

                        // Update player's best score for THIS TOURNAMENT
                        let player_record = contract
                            .state
                            .player_records
                            .load_entry_mut(&player)
                            .await
                            .unwrap();
                        player_record
                            .best_score
                            .insert(&leaderboard_id, final_score)
                            .unwrap();
                    }
                }
                // 🔒 DUPLICATE DETECTION: All moves were already processed (retry scenario)
                // This is NOT an error - silently succeed since moves were already applied
                GameMoveResult::NoNewMoves { skipped_count } => {
                    // Log for debugging but don't panic - this is expected during retries
                    log::info!(
                        "Duplicate batch detected: {} moves skipped for board {}",
                        skipped_count,
                        board_id
                    );
                    // No state changes needed - moves were already processed
                }
                GameMoveResult::Error(msg) => {
                    panic!("{}", msg);
                }
            }
        } else if moves.is_empty() {
            // 🚀 FORCED GAME END (tournament time expired or explicit end)
            // This always sends final score if it beats tournament best
            let score = Game::score(*board.board.get());
            let highest_tile = Game::highest_tile(*board.board.get());
            
            // Get tournament times for SubmitScore message
            let board_start_time = *board.start_time.get();
            let board_end_time = *board.end_time.get();

            // 🚀 MARK GAME AS ENDED
            board.is_ended.set(true);

            // Get tournament ID from the board
            let leaderboard_id = board.leaderboard_id.get().clone();

            // Get current best score for this player from player_records
            let player_record = contract
                .state
                .player_records
                .load_entry_mut(&player)
                .await
                .unwrap();
            let current_best = player_record
                .best_score
                .get(&leaderboard_id)
                .await
                .unwrap()
                .unwrap_or(0);

            // 🚀 MESSAGE-BASED: Send SubmitScore for game end (only if score > 0 and beats current best)
            if score > 0 && score > current_best {
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

                use linera_sdk::linera_base_types::ChainId;
                use std::str::FromStr;
                
                let player_chain_id = contract.runtime.chain_id().to_string();
                let timestamp = contract.runtime.system_time().micros();
                
                if let Ok(leaderboard_chain_id) = ChainId::from_str(&leaderboard_id) {
                    contract
                        .runtime
                        .prepare_message(game2048::Message::SubmitScore {
                            player: player.clone(),
                            player_chain_id,
                            board_id: board_id.clone(),
                            score,
                            highest_tile,
                            game_status: GameStatus::Ended(GameEndReason::TournamentEnded),
                            timestamp,
                            boards_in_tournament: current_board_count,
                            start_time: board_start_time,
                            end_time: board_end_time,
                        })
                        .send_to(leaderboard_chain_id);
                }

                // Update player's best score for THIS TOURNAMENT
                let player_record = contract
                    .state
                    .player_records
                    .load_entry_mut(&player)
                    .await
                    .unwrap();
                player_record
                    .best_score
                    .insert(&leaderboard_id, score)
                    .unwrap();
            }
        } else {
            panic!("Game is ended");
        }
    }

    /// 🚀 MESSAGE-BASED: Create a new board for the player
    /// 
    /// In the message-based architecture:
    /// - No shards needed
    /// - Board is created locally on player chain
    /// - Scores are sent directly to leaderboard via SubmitScore message when player makes moves
    pub async fn handle_new_board(
        contract: &mut crate::Game2048Contract,
        player: String,
        timestamp: u64,
        password_hash: String,
        leaderboard_id: String,
        // 🎵 Rhythm mode: which music track was used (-1 = no rhythm/metronome, 0+ = track index)
        rhythm_track_index: i16,
    ) {
        // Validate password
        contract
            .validate_player_password(&player, &password_hash)
            .await;

        // Get tournament times from cache (if available)
        let (tournament_start_time, tournament_end_time) =
            if let Some(tournament) = contract.get_cached_tournament(&leaderboard_id).await {
                (
                    tournament.start_time.unwrap_or(0),
                    tournament.end_time.unwrap_or(0),
                )
            } else {
                (0, 0) // Default to unlimited if tournament not in cache
            };

        // 🔒 VALIDATION: Reject board creation if tournament hasn't started yet
        if tournament_start_time > 0 {
            let current_time = contract.runtime.system_time().micros();
            if current_time < tournament_start_time {
                panic!("Tournament has not started yet");
            }
        }

        // 🔒 VALIDATION: Reject board creation if tournament has already ended
        if tournament_end_time > 0 {
            let current_time = contract.runtime.system_time().micros();
            if current_time >= tournament_end_time {
                panic!("Tournament has already ended");
            }
        }

        // Create board locally
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
        game.shard_id.set(String::new()); // No shard in message-based architecture
        game.chain_id.set(contract.runtime.chain_id().to_string());
        game.created_at.set(timestamp);
        game.start_time.set(tournament_start_time);
        game.end_time.set(tournament_end_time);
        // 🎵 Rhythm mode: store track index for replay (-1 = no rhythm/metronome)
        game.rhythm_track_index.set(rhythm_track_index);

        contract.state.nonce.set(nonce + 1);
        contract.state.latest_board_id.set(board_id);

        // Increment player's board count for this tournament
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
        player_state
            .boards_per_tournament
            .insert(&leaderboard_id, current_board_count + 1)
            .unwrap();

        // 🚀 MESSAGE-BASED: No registration with shard needed
        // No event emission needed
        // First SubmitScore is sent when player makes moves and score > 0
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

    /// 🚀 MESSAGE-BASED: Handle leaderboard update (manual refresh)
    /// 
    /// With message-based architecture, this operation just needs to:
    /// 1. Check cooldown (10s spam protection)
    /// 2. Trigger block production (which processes all pending SubmitScore messages)
    /// 
    /// The leaderboard chain should run with --listener-skip-process-inbox so
    /// SubmitScore messages queue up and are processed when this operation is called.
    pub async fn handle_update_leaderboard(contract: &mut crate::Game2048Contract) {
        let current_time = contract.runtime.system_time().micros();

        let leaderboard = contract
            .state
            .leaderboards
            .load_entry_mut("")
            .await
            .unwrap();

        // 🚀 COOLDOWN CHECK - 15s spam protection
        let cooldown_until = *leaderboard.trigger_cooldown_until.get();
        if current_time < cooldown_until {
            // Still in cooldown - silently ignore
            return;
        }

        // Set global cooldown (15 seconds)
        let cooldown_duration = 15_000_000; // 15 seconds in microseconds
        leaderboard
            .trigger_cooldown_until
            .set(current_time + cooldown_duration);

        // Update trigger tracking
        leaderboard.last_trigger_time.set(current_time);
        leaderboard
            .last_trigger_by
            .set("manual_refresh".to_string());

        // That's it! The act of calling this operation triggers block production,
        // which processes all pending SubmitScore messages in the inbox.
        // No need to send messages to shards anymore.
    }
    
    /// 🚀 MANUAL SCORE SUBMISSION: Submit current board score to leaderboard
    /// Called when user clicks "refresh leaderboard" button
    /// Only sends if: score > 0 AND score > player's tournament best
    pub async fn handle_submit_current_score(
        contract: &mut crate::Game2048Contract,
        board_id: String,
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

        // Verify ownership
        if player != *board.player.get() {
            panic!("You can only submit score for your own board");
        }

        // Get current board state
        let current_board = *board.board.get();
        let score = Game::score(current_board);
        let highest_tile = Game::highest_tile(current_board);
        let is_ended = *board.is_ended.get();
        let leaderboard_id = board.leaderboard_id.get().clone();
        let board_start_time = *board.start_time.get();
        let board_end_time = *board.end_time.get();

        // Get current best score for this player in this tournament
        let player_record = contract
            .state
            .player_records
            .load_entry_mut(&player)
            .await
            .unwrap();
        let current_best = player_record
            .best_score
            .get(&leaderboard_id)
            .await
            .unwrap()
            .unwrap_or(0);

        // Only send if score > 0 AND score > current_best
        if score == 0 || score <= current_best {
            // Nothing to submit - score not better than current best
            return;
        }

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

        // Determine game status
        let game_status = if is_ended {
            GameStatus::Ended(GameEndReason::NoMoves)
        } else {
            GameStatus::Active
        };

        let player_chain_id = contract.runtime.chain_id().to_string();
        let timestamp = contract.runtime.system_time().micros();

        if let Ok(leaderboard_chain_id) = ChainId::from_str(&leaderboard_id) {
            contract
                .runtime
                .prepare_message(game2048::Message::SubmitScore {
                    player: player.clone(),
                    player_chain_id,
                    board_id: board_id.clone(),
                    score,
                    highest_tile,
                    game_status,
                    timestamp,
                    boards_in_tournament: current_board_count,
                    start_time: board_start_time,
                    end_time: board_end_time,
                })
                .send_to(leaderboard_chain_id);
        }

        // Update player's best score for this tournament
        let player_record = contract
            .state
            .player_records
            .load_entry_mut(&player)
            .await
            .unwrap();
        player_record
            .best_score
            .insert(&leaderboard_id, score)
            .unwrap();
            
        // Update board tracking state
        let board = contract
            .state
            .boards
            .load_entry_mut(&board_id)
            .await
            .unwrap();
        board.highest_tile_sent.set(highest_tile);
        board.last_score_sent_time.set(timestamp);
    }
}
