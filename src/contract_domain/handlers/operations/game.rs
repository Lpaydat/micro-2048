//! Game Operations Handler
//! 
//! Handles game-related operations including moves and board creation.

use std::str::FromStr;
use linera_sdk::linera_base_types::ChainId;
use game2048::{Direction, Game, GameEvent, GameEndReason, GameStatus, Message, hash_seed};
use crate::contract_domain::game_logic::{GameMoveProcessor, GameMoveResult};

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
        contract.validate_player_password(&player, &password_hash).await;
        let board = contract.state.boards.load_entry_mut(&board_id).await.unwrap();
        let shard_id = board.shard_id.get().clone();

        if player != *board.player.get() {
            panic!("You can only make move on your own board");
        }

        type MoveInput = (Direction, String);
        let moves: Vec<MoveInput> =
            serde_json::from_str(&moves).unwrap_or_else(|_| panic!("Invalid moves format"));

        let is_ended = *board.is_ended.get();
        let end_time = *board.end_time.get();
        
        if !is_ended && !moves.is_empty() {
            let initial_board = *board.board.get();
            
            // Convert string timestamps to u64
            let moves_u64: Vec<(Direction, u64)> = moves
                .into_iter()
                .map(|(dir, ts)| (dir, ts.parse::<u64>().unwrap()))
                .collect();

            match GameMoveProcessor::process_moves(&board_id, &player, &moves_u64, initial_board, end_time) {
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

                    let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
                    let leaderboard_id = leaderboard.leaderboard_id.get().clone();

                    // Get current best score for this player in this leaderboard
                    let current_best = leaderboard.score.get(&player).await.unwrap().unwrap_or(0);

                    let score_event = GameEvent::PlayerScoreUpdate {
                        player: player.clone(),
                        board_id: board_id.clone(),
                        score: final_score,
                        chain_id: contract.runtime.chain_id().to_string(),
                        timestamp: latest_timestamp,
                        game_status,
                        highest_tile: final_highest_tile,
                        moves_count: 0, // TODO: Track actual move count
                        leaderboard_id: leaderboard_id.clone(),
                        current_leaderboard_best: current_best,
                    };

                    use linera_sdk::linera_base_types::StreamName;
                    let stream_name = StreamName::from("player_score_update".to_string());
                    contract.runtime.emit(stream_name, &score_event);
                    
                    // 🚀 NEW: Update shard workload when scores change significantly
                    let score_improvement = final_score.saturating_sub(current_best);
                    if score_improvement > 2000 || is_ended {
                        contract.emit_shard_workload().await;
                        
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
                GameMoveResult::Error(msg) => panic!("{}", msg),
            }
        } else if moves.is_empty() {
            let score = Game::score(*board.board.get());
            
            // 🚀 NEW: Emit player score update for tournament end
            // This case handles when tournament time expires and game ends
            let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
            let leaderboard_id = leaderboard.leaderboard_id.get().clone();
            
            // Get current best score for this player in this leaderboard
            let current_best = leaderboard.score.get(&player).await.unwrap().unwrap_or(0);
            
            let score_event = GameEvent::PlayerScoreUpdate {
                player: player.clone(),
                board_id: board_id.clone(),
                score,
                chain_id: contract.runtime.chain_id().to_string(),
                timestamp: 111970,
                game_status: GameStatus::Ended(GameEndReason::TournamentEnded),
                highest_tile: Game::highest_tile(*board.board.get()),
                moves_count: 0, // TODO: Track actual move count
                leaderboard_id: leaderboard_id.clone(),
                current_leaderboard_best: current_best,
            };
            
            use linera_sdk::linera_base_types::StreamName;
            let stream_name = StreamName::from("player_score_update".to_string());
            contract.runtime.emit(stream_name, &score_event);
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
        contract.validate_player_password(&player, &password_hash).await;

        // 🚀 NEW: Get main chain ID (central registry)
        let main_chain_id = contract.runtime.application_creator_chain_id();

        // 🚀 NEW: Validate leaderboard exists and is active
        // Read active tournaments from main chain (central registry)
        let is_valid_leaderboard = contract.validate_tournament(&leaderboard_id, main_chain_id).await;

        if !is_valid_leaderboard {
            panic!("Leaderboard '{}' is not active or does not exist", leaderboard_id);
        }

        // 🚀 NEW: Get leaderboard info and select optimal shard
        // Select optimal shard from main chain registry
        let selected_shard_id = contract.select_optimal_shard(&leaderboard_id, main_chain_id).await;
        
        // 🚀 NEW: Create board locally (no cross-chain message needed)
        let nonce = contract.state.nonce.get();
        let board_id = format!("{}.{}", contract.runtime.chain_id(), hash_seed(&nonce.to_string(), &player, timestamp));
        
        let new_board = Game::new(&board_id, &player, timestamp).board;
        let game = contract.state.boards.load_entry_mut(&board_id).await.unwrap();
        game.board_id.set(board_id.clone());
        game.board.set(new_board);
        game.player.set(player.clone());
        game.leaderboard_id.set(leaderboard_id.clone());
        game.shard_id.set(selected_shard_id.clone());
        game.chain_id.set(contract.runtime.chain_id().to_string());
        game.created_at.set(timestamp);
        
        contract.state.nonce.set(nonce + 1);
        contract.state.latest_board_id.set(board_id.clone());

        // 🚀 NEW: Register with selected shard (one-time registration)
        let registration_message = Message::RegisterPlayerWithShard {
            player_chain_id: contract.runtime.chain_id().to_string(),
            tournament_id: leaderboard_id.clone(),
            player_name: player.clone(),
        };
        contract.runtime
            .prepare_message(registration_message)
            .send_to(ChainId::from_str(&selected_shard_id).unwrap());
        
        // 🚀 BOOTSTRAP: First player triggers initial aggregation
        let is_first_player = {
            let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
            leaderboard.primary_triggerer.get().is_empty()
        };
        
        if is_first_player {
            // This is the first player - trigger initial aggregation to bootstrap
            if let Ok(leaderboard_chain_id) = ChainId::from_str(&leaderboard_id) {
                let requester_id = contract.runtime.chain_id().to_string();
                contract.runtime
                    .prepare_message(Message::RequestAggregationTrigger {
                        requester_chain_id: requester_id,
                        timestamp,
                    })
                    .send_to(leaderboard_chain_id);
            }
        }

        // 🚀 NEW: Emit game creation event
        contract.emit_game_creation_event(&board_id, &player, &leaderboard_id, timestamp).await;
        
        // 🚀 NEW: Track activity for workload statistics
        contract.track_game_activity().await;
        
        // 🚀 NEW: Emit workload update when new games are created
        contract.emit_shard_workload().await;
    }

    /// 🚀 IMPROVED: Handle score aggregation using monitored player chains from shard state
    pub async fn handle_aggregate_scores(
        contract: &mut crate::Game2048Contract,
    ) {
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
        contract.aggregate_scores_from_player_chains(player_chain_ids).await;
        
        // 🚀 NEW: Emit workload update after aggregation
        contract.emit_shard_workload().await;
    }

    /// 🚀 IMPROVED: Handle leaderboard update using registered shard chains from leaderboard state
    pub async fn handle_update_leaderboard(
        contract: &mut crate::Game2048Contract,
    ) {
        // Get registered shard chain IDs from leaderboard state
        let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
        let mut shard_chain_ids = Vec::new();
        
        // 🚀 FIXED: Collect ALL registered shard chain IDs (no limit)
        let mut read_count = 0;
        #[allow(clippy::while_let_loop)]
        loop {
            match leaderboard.shard_ids.read_front(1000).await { // Large batch size
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
        contract.update_leaderboard_from_shard_chains(shard_chain_ids).await;
    }
}
