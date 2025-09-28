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
        let end_time_raw = *board.end_time.get();
        // Convert 0 or u64::MAX to None (unlimited), otherwise Some(value)
        let end_time = if end_time_raw == 0 || end_time_raw == u64::MAX {
            None
        } else {
            Some(end_time_raw)
        };
        
        if !is_ended && !moves.is_empty() {
            let initial_board = *board.board.get();
            log::info!("🎯 MAKE_MOVES: Processing {} moves for board {}", moves.len(), board_id);
            log::info!("🎯 MAKE_MOVES: Board is_ended: {}, end_time: {:?}", is_ended, end_time);
            log::info!("🎯 MAKE_MOVES: Initial board state: 0x{:016x}", initial_board);
            
            // Convert string timestamps to u64
            let moves_u64: Vec<(Direction, u64)> = moves
                .into_iter()
                .map(|(dir, ts)| (dir, ts.parse::<u64>().unwrap()))
                .collect();
            
            log::info!("🎯 MAKE_MOVES: Parsed moves: {:?}", moves_u64);

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

                    // Get player's current board count for this tournament
                    let player_state = contract.state.players.load_entry_mut(&player).await.unwrap();
                    let current_board_count = player_state.boards_per_tournament
                        .get(&leaderboard_id).await.unwrap().unwrap_or(0);
                    
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
                        boards_in_tournament: current_board_count,
                    };

                    use linera_sdk::linera_base_types::StreamName;
                    let stream_name = StreamName::from("player_score_update".to_string());
                    if let GameEvent::PlayerScoreUpdate { player, score, game_status, .. } = &score_event {
                        log::info!("📡 EMIT: Emitting player_score_update event - Player: '{}', Score: {}, Status: {:?}, Chain: {}", 
                            player, score, game_status, contract.runtime.chain_id());
                        log::info!("📡 EMIT: Event will be delivered to subscribed shard chains via process_streams");
                    }
                    contract.runtime.emit(stream_name, &score_event);
                    log::info!("📡 EMIT: ✅ Successfully emitted player_score_update event");
                    
                    // 🚀 NEW: Update shard workload when scores change significantly
                    let score_improvement = final_score.saturating_sub(current_best);
                    if score_improvement > 2000 || is_ended {
                        // Temporarily commented out to isolate move processing from system_time() issues
                        // contract.emit_shard_workload().await;
                        
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
                    log::error!("🎯 MAKE_MOVES: Game move processing failed: {}", msg);
                    panic!("{}", msg);
                }
            }
        } else if moves.is_empty() {
            let score = Game::score(*board.board.get());
            
            // 🚀 NEW: Emit player score update for tournament end
            // This case handles when tournament time expires and game ends
            let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
            let leaderboard_id = leaderboard.leaderboard_id.get().clone();
            
            // Get current best score for this player in this leaderboard
            let current_best = leaderboard.score.get(&player).await.unwrap().unwrap_or(0);
            
            // Get player's current board count for this tournament
            let player_state = contract.state.players.load_entry_mut(&player).await.unwrap();
            let current_board_count = player_state.boards_per_tournament
                .get(&leaderboard_id).await.unwrap().unwrap_or(0);
            
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
                boards_in_tournament: current_board_count,
            };
            
            use linera_sdk::linera_base_types::StreamName;
            let stream_name = StreamName::from("player_score_update".to_string());
            log::info!("📡 EMIT: Emitting final player_score_update event - Player: '{}', Final Score: {}, Chain: {}", 
                player, score, contract.runtime.chain_id());
            log::info!("📡 EMIT: Final score event will be delivered to subscribed shard chains via process_streams");
            contract.runtime.emit(stream_name, &score_event);
            log::info!("📡 EMIT: ✅ Successfully emitted final player_score_update event");
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

        // 🚀 NEW: Show cached tournament information
        log::info!("🎯 NEW_BOARD: Creating board for tournament '{}'", leaderboard_id);
        
        // Display cached tournaments from streaming system
        let cached_tournaments = contract.list_cached_tournaments().await;
        let cache_count = contract.get_cached_tournament_count().await;
        
        log::info!("🎯 NEW_BOARD: Player chain has {} cached tournaments from streaming system", cache_count);
        
        if !cached_tournaments.is_empty() {
            log::info!("🎯 NEW_BOARD: Available tournaments in cache:");
            for (idx, tournament) in cached_tournaments.iter().enumerate() {
                let status = if tournament.tournament_id == leaderboard_id { "🎯 TARGET" } else { "📋" };
                log::info!("🎯 NEW_BOARD:   {}. {} '{}' (ID: {}...)", 
                    idx + 1, status, tournament.name, &tournament.tournament_id[..16]);
            }
        } else {
            log::warn!("🎯 NEW_BOARD: ⚠️ No tournaments found in cache - may need to wait for stream updates");
        }
        
        // Check if target tournament is in cache
        if let Some(target_tournament) = contract.get_cached_tournament(&leaderboard_id).await {
            log::info!("🎯 NEW_BOARD: ✅ Target tournament '{}' found in cache", target_tournament.name);
            log::info!("🎯 NEW_BOARD:   - Shards: {} available", target_tournament.shard_chain_ids.len());
        } else {
            log::warn!("🎯 NEW_BOARD: ⚠️ Target tournament '{}' not found in cache", leaderboard_id);
        }

        // 🚀 NEW: Validate tournament exists and is active
        log::info!("🎯 VALIDATION: Checking if tournament '{}' is active and valid", leaderboard_id);
        let is_valid_tournament = contract.validate_tournament(&leaderboard_id).await;

        if !is_valid_tournament {
            panic!("Tournament '{}' is not active, expired, or does not exist", leaderboard_id);
        }
        
        log::info!("🎯 VALIDATION: ✅ Tournament '{}' is valid and active", leaderboard_id);

        // 🚀 NEW: Get leaderboard info and select optimal shard using hash-based distribution
        let selected_shard_id = contract.select_optimal_shard(&leaderboard_id, &player).await;
        
        // 🚀 FIX: Get tournament data before creating board to avoid borrow conflicts
        let tournament_end_time = if let Some(tournament) = contract.get_cached_tournament(&leaderboard_id).await {
            let end_time_value = tournament.end_time.unwrap_or(0); // None -> 0 (unlimited)
            log::info!("🎯 NEW_BOARD: Found tournament in cache with end_time: {:?} -> {}", tournament.end_time, end_time_value);
            end_time_value
        } else {
            log::warn!("🎯 NEW_BOARD: Tournament not found in cache, using unlimited end_time (0)");
            0 // Default to unlimited if tournament not in cache
        };
        
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
        game.end_time.set(tournament_end_time);
        
        contract.state.nonce.set(nonce + 1);
        contract.state.latest_board_id.set(board_id.clone());

        // 🚀 NEW: Increment player's board count for this tournament (distributed counting)
        let player_state = contract.state.players.load_entry_mut(&player).await.unwrap();
        let current_board_count = player_state.boards_per_tournament
            .get(&leaderboard_id).await.unwrap().unwrap_or(0);
        let new_board_count = current_board_count + 1;
        player_state.boards_per_tournament.insert(&leaderboard_id, new_board_count).unwrap();
        
        log::info!("🏆 BOARD_COUNT: Player '{}' now has {} boards in tournament '{}'", 
                  player, new_board_count, leaderboard_id);

        // 🚀 NEW: Register with selected shard (one-time registration)
        log::info!("🎯 PLAYER_REGISTRATION: Sending registration message to shard {}", selected_shard_id);
        log::info!("🎯 PLAYER_REGISTRATION: Player chain: {}, Tournament: {}, Player: {}", 
                  contract.runtime.chain_id(), leaderboard_id, player);
        
        let registration_message = Message::RegisterPlayerWithShard {
            player_chain_id: contract.runtime.chain_id().to_string(),
            tournament_id: leaderboard_id.clone(),
            player_name: player.clone(),
        };
        
        if let Ok(shard_chain_id) = ChainId::from_str(&selected_shard_id) {
            contract.runtime
                .prepare_message(registration_message)
                .send_to(shard_chain_id);
            log::info!("🎯 PLAYER_REGISTRATION: ✅ Registration message sent to shard {}", selected_shard_id);
        } else {
            log::error!("🎯 PLAYER_REGISTRATION: ❌ Invalid shard chain ID: {}", selected_shard_id);
        }
        
        // 🚀 NEW: Subscribe to leaderboard updates for triggerer system
        if let Ok(leaderboard_chain_id) = ChainId::from_str(&leaderboard_id) {
            log::info!("🔔 PLAYER_REGISTRATION: Subscribing to leaderboard_update events from tournament {}", leaderboard_id);
            contract.subscribe_to_leaderboard_update_events(leaderboard_chain_id);
            log::info!("🔔 PLAYER_REGISTRATION: ✅ Subscribed to triggerer list updates from leaderboard");
        } else {
            log::error!("🔔 PLAYER_REGISTRATION: ❌ Invalid leaderboard chain ID: {}", leaderboard_id);
        }
        
        // 🚀 BOOTSTRAP: Shard chains now handle first-player registration (not player chains)

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
