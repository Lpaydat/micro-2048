use game2048::{Direction, Game};

pub struct GameMoveProcessor;

pub struct ProcessedMove {
    pub direction: Direction,
    pub timestamp: u64,
    pub board_after: u64,
    pub score_after: u64,
    // 🎵 Rhythm mode: which beat this move was on (0 = miss/off-beat, >0 = on-beat)
    pub beat_number: u32,
}

impl GameMoveProcessor {
    /// Process a batch of moves, skipping any that were already processed (duplicate detection).
    /// 
    /// # Arguments
    /// * `board_id` - The board identifier
    /// * `player` - The player making the moves
    /// * `moves` - The moves to process (direction, timestamp in milliseconds, beat_number)
    ///   - beat_number: 0 = miss/off-beat, >0 = on-beat (which beat number)
    /// * `initial_board` - Current board state
    /// * `last_processed_timestamp` - Last timestamp that was successfully processed (for duplicate detection)
    /// * `start_time` - Tournament start time in microseconds (None = unlimited)
    /// * `end_time` - Tournament end time in microseconds (None = unlimited)
    pub fn process_moves(
        board_id: &str,
        player: &str,
        moves: &[(Direction, u64, u32)],  // 🎵 Added beat_number
        initial_board: u64,
        last_processed_timestamp: u64, // 🔒 NEW: For duplicate detection
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> GameMoveResult {
        let initial_highest_tile = Game::highest_tile(initial_board);
        let mut current_board = initial_board;
        let mut any_change = false;
        let mut latest_timestamp = last_processed_timestamp; // 🔒 FIX: Start from last processed
        let mut is_ended = false;
        let mut move_history: Vec<ProcessedMove> = Vec::new();
        let mut skipped_duplicate_count = 0; // 🔒 NEW: Track skipped duplicates

        for (direction, timestamp, beat_number) in moves.iter() {
            if is_ended {
                break;
            }

            // NOTE: timestamp is in milliseconds (from frontend), start/end_time is in microseconds
            let timestamp_micros = *timestamp * 1000;

            // 🔒 VALIDATION: Reject moves before tournament start
            if let Some(start_time_value) = start_time {
                if timestamp_micros < start_time_value {
                    return GameMoveResult::Error(
                        "Move timestamp is before tournament start time".to_string(),
                    );
                }
            }

            // 🔒 VALIDATION: End game if move is after tournament end
            if let Some(end_time_value) = end_time {
                if timestamp_micros > end_time_value {
                    is_ended = true;
                    break;
                }
            }

            // 🔒 DUPLICATE DETECTION: Skip moves that were already processed
            // This handles retry scenarios where the same batch is sent multiple times
            if *timestamp <= latest_timestamp {
                skipped_duplicate_count += 1;
                continue; // Skip this move, don't error - it was already processed
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
            let current_score = Game::score(current_board);

            // Store this move in history
            // 🎵 beat_number from input: 0 = miss/off-beat, >0 = on-beat
            move_history.push(ProcessedMove {
                direction: *direction,
                timestamp: *timestamp,
                board_after: current_board,
                score_after: current_score,
                beat_number: *beat_number,
            });

            is_ended = Game::is_ended(current_board);

            if is_ended {
                break;
            }
        }

        // 🔒 DUPLICATE DETECTION: If ALL moves were skipped (pure duplicate batch), return success with no changes
        if !any_change && skipped_duplicate_count > 0 {
            return GameMoveResult::NoNewMoves {
                skipped_count: skipped_duplicate_count,
            };
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
            move_history,
        }
    }
}

#[allow(dead_code)]
pub enum GameMoveResult {
    Success {
        final_board: u64,
        final_score: u64,
        final_highest_tile: u64,
        initial_highest_tile: u64, // Kept for potential future use
        is_ended: bool,
        latest_timestamp: u64,
        move_history: Vec<ProcessedMove>,
    },
    /// 🔒 NEW: All moves in the batch were duplicates (already processed)
    /// This is NOT an error - it means a retry succeeded but had no new moves
    NoNewMoves {
        skipped_count: usize,
    },
    Error(String),
}
