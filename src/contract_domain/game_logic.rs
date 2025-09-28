use game2048::{Direction, Game};

pub struct GameMoveProcessor;

impl GameMoveProcessor {
    pub fn process_moves(
        board_id: &str,
        player: &str,
        moves: &[(Direction, u64)],
        initial_board: u64,
        end_time: Option<u64>,
    ) -> GameMoveResult {
        log::info!("🎮 PROCESS_MOVES: Starting move processing for board_id: {}", board_id);
        log::info!("🎮 PROCESS_MOVES: Player: {}, Moves count: {}, End time: {:?}", player, moves.len(), end_time);
        log::info!("🎮 PROCESS_MOVES: Initial board: 0x{:016x}", initial_board);
        
        let initial_highest_tile = Game::highest_tile(initial_board);
        let mut current_board = initial_board;
        let mut any_change = false;
        let mut latest_timestamp = 0;
        let mut is_ended = false;

        for (direction, timestamp) in moves.iter() {
            if is_ended {
                break;
            }

            // Only validate end_time if it's set (Some value)
            if let Some(end_time_value) = end_time {
                if *timestamp > end_time_value {
                    log::info!("🎮 PROCESS_MOVES: Timestamp {} > end_time {}, ending game", timestamp, end_time_value);
                    is_ended = true;
                    break;
                }
            }
            
            if *timestamp < latest_timestamp {
                log::error!("🎮 PROCESS_MOVES: Timestamp {} < latest_timestamp {}, rejecting", timestamp, latest_timestamp);
                return GameMoveResult::Error("Timestamp must be after latest timestamp".to_string());
            }
            latest_timestamp = *timestamp;

            let mut game = Game {
                board: current_board,
                board_id: board_id.to_string(),
                username: player.to_string(),
                timestamp: *timestamp,
            };

            let new_board = game.execute(*direction);
            
            if current_board == new_board {
                continue;
            }

            any_change = true;
            current_board = new_board;
            is_ended = Game::is_ended(current_board);
            
            if is_ended {
                break;
            }
        }

        if !any_change {
            log::error!("🎮 PROCESS_MOVES: ❌ FAILED - No valid moves in the sequence of {} moves", moves.len());
            log::error!("🎮 PROCESS_MOVES: Final board state: 0x{:016x}", current_board);
            return GameMoveResult::Error("No valid moves in the sequence".to_string());
        }
        
        log::info!("🎮 PROCESS_MOVES: ✅ SUCCESS - At least one move was valid");

        let final_score = Game::score(current_board);
        let final_highest_tile = Game::highest_tile(current_board);

        GameMoveResult::Success {
            final_board: current_board,
            final_score,
            final_highest_tile,
            initial_highest_tile,
            is_ended,
            latest_timestamp,
        }
    }
}

pub enum GameMoveResult {
    Success {
        final_board: u64,
        final_score: u64,
        final_highest_tile: u64,
        initial_highest_tile: u64,
        is_ended: bool,
        latest_timestamp: u64,
    },
    Error(String),
}