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
                    is_ended = true;
                    break;
                }
            }

            if *timestamp < latest_timestamp {
                return GameMoveResult::Error(
                    "Timestamp must be after latest timestamp".to_string(),
                );
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
            return GameMoveResult::Error("No valid moves in the sequence".to_string());
        }

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
