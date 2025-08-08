//! Game session domain service for managing game session lifecycle and business rules.

use crate::core::models::*;
use crate::core::value_objects::*;
use linera_sdk::linera_base_types::Timestamp;

/// Domain service for game session management
pub struct GameSessionService;

impl GameSessionService {
    /// Create a new game session with proper validation
    pub fn create_session(
        session_id: GameSessionId,
        participant_id: ParticipantId,
        game_variant: GameVariant,
        current_time: Timestamp,
        competition_context: Option<CompetitionContext>,
    ) -> Result<GameSession, GameSessionError> {
        // Validate game variant
        Self::validate_game_variant(&game_variant)?;

        // Create the session
        let session = GameSession::new(
            session_id,
            participant_id,
            game_variant,
            current_time,
            competition_context,
        );

        Ok(session)
    }

    /// Process a move for a game session
    pub fn process_move(
        session: &mut GameSession,
        direction: Direction,
        current_time: Timestamp,
    ) -> Result<MoveResult, GameSessionError> {
        // Validate session state
        if !session.is_active() {
            return Err(GameSessionError::SessionNotActive);
        }

        // Check time limits
        if session.is_time_expired(current_time) {
            session.complete_session(SessionStatus::CompletedWithFailure, current_time);
            return Err(GameSessionError::TimeExpired);
        }

        // Update activity timestamp
        session.update_activity(current_time);

        // Process the move based on game variant
        let move_result = match session.game_variant {
            GameVariant::Classic2048 => Self::process_classic_move(&session.board_state, direction)?,
            GameVariant::Speed2048 { .. } => Self::process_speed_move(&session.board_state, direction)?,
            GameVariant::Elimination { .. } => Self::process_elimination_move(&session.board_state, direction)?,
            GameVariant::Collaborative { .. } => Self::process_collaborative_move(&session.board_state, direction)?,
        };

        // Update session state
        session.board_state = move_result.new_board_state;
        session.scoring_metrics.primary_score += move_result.score_delta;
        session.board_state.move_count += 1;

        // Update highest tile achieved
        if move_result.new_board_state.highest_tile_achieved > session.board_state.highest_tile_achieved {
            session.board_state.highest_tile_achieved = move_result.new_board_state.highest_tile_achieved;
        }

        // Check for game end conditions
        if move_result.game_ended {
            let status = if Self::is_winning_condition(&session.board_state) {
                SessionStatus::CompletedSuccessfully
            } else {
                SessionStatus::CompletedWithFailure
            };
            session.complete_session(status, current_time);
        }

        // Add any achievements
        for achievement in &move_result.achievements_unlocked {
            session.scoring_metrics.achievement_unlocks.push(achievement.clone());
        }

        Ok(move_result)
    }

    /// Validate game variant configuration
    fn validate_game_variant(variant: &GameVariant) -> Result<(), GameSessionError> {
        match variant {
            GameVariant::Speed2048 { time_limit_seconds } => {
                if *time_limit_seconds == 0 || *time_limit_seconds > 3600 {
                    return Err(GameSessionError::InvalidVariantConfig(
                        "Speed game time limit must be between 1 and 3600 seconds".to_string()
                    ));
                }
            }
            GameVariant::Elimination { lives_remaining } => {
                if *lives_remaining == 0 || *lives_remaining > 10 {
                    return Err(GameSessionError::InvalidVariantConfig(
                        "Elimination game must have between 1 and 10 lives".to_string()
                    ));
                }
            }
            GameVariant::Collaborative { team_size } => {
                if *team_size < 2 || *team_size > 8 {
                    return Err(GameSessionError::InvalidVariantConfig(
                        "Collaborative game must have between 2 and 8 team members".to_string()
                    ));
                }
            }
            GameVariant::Classic2048 => {
                // No additional validation needed for classic variant
            }
        }
        Ok(())
    }

    /// Process a move for classic 2048 variant
    fn process_classic_move(board_state: &BoardState, direction: Direction) -> Result<MoveResult, GameSessionError> {
        // For now, return a placeholder result
        // This will be implemented with the actual game logic in later tasks
        Ok(MoveResult {
            new_board_state: *board_state,
            score_delta: 0,
            tiles_merged: Vec::new(),
            new_tile_spawned: None,
            game_ended: false,
            achievements_unlocked: Vec::new(),
        })
    }

    /// Process a move for speed 2048 variant
    fn process_speed_move(board_state: &BoardState, direction: Direction) -> Result<MoveResult, GameSessionError> {
        // Speed variant uses same logic as classic but with time pressure
        Self::process_classic_move(board_state, direction)
    }

    /// Process a move for elimination variant
    fn process_elimination_move(board_state: &BoardState, direction: Direction) -> Result<MoveResult, GameSessionError> {
        // Elimination variant has additional failure conditions
        Self::process_classic_move(board_state, direction)
    }

    /// Process a move for collaborative variant
    fn process_collaborative_move(board_state: &BoardState, direction: Direction) -> Result<MoveResult, GameSessionError> {
        // Collaborative variant may have different scoring
        Self::process_classic_move(board_state, direction)
    }

    /// Check if the current board state represents a winning condition
    fn is_winning_condition(board_state: &BoardState) -> bool {
        // For classic 2048, winning means reaching the 2048 tile
        board_state.highest_tile_achieved >= 2048
    }

    /// Calculate final score with bonuses and multipliers
    pub fn calculate_final_score(session: &GameSession) -> u64 {
        let mut final_score = session.scoring_metrics.primary_score;

        // Apply bonus multipliers
        for bonus in &session.scoring_metrics.bonus_multipliers {
            final_score = (final_score as f64 * bonus.value) as u64;
        }

        // Add achievement bonuses
        let achievement_bonus: u64 = session.scoring_metrics.achievement_unlocks
            .len() as u64 * 100; // 100 points per achievement

        final_score + achievement_bonus
    }

    /// Abandon a game session
    pub fn abandon_session(
        session: &mut GameSession,
        current_time: Timestamp,
    ) -> Result<(), GameSessionError> {
        if !session.is_active() {
            return Err(GameSessionError::SessionNotActive);
        }

        session.complete_session(SessionStatus::AbandonedByPlayer, current_time);
        Ok(())
    }

    /// Force terminate a session (for administrative purposes)
    pub fn terminate_session(
        session: &mut GameSession,
        current_time: Timestamp,
    ) -> Result<(), GameSessionError> {
        session.complete_session(SessionStatus::TerminatedBySystem, current_time);
        Ok(())
    }
}

/// Errors that can occur in game session operations
#[derive(Debug, Clone)]
pub enum GameSessionError {
    SessionNotActive,
    TimeExpired,
    InvalidMove(String),
    InvalidVariantConfig(String),
    BoardStateCorrupted,
    UnexpectedError(String),
}

impl std::fmt::Display for GameSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameSessionError::SessionNotActive => write!(f, "Game session is not active"),
            GameSessionError::TimeExpired => write!(f, "Game session time limit expired"),
            GameSessionError::InvalidMove(msg) => write!(f, "Invalid move: {}", msg),
            GameSessionError::InvalidVariantConfig(msg) => write!(f, "Invalid game variant configuration: {}", msg),
            GameSessionError::BoardStateCorrupted => write!(f, "Board state is corrupted"),
            GameSessionError::UnexpectedError(msg) => write!(f, "Unexpected error: {}", msg),
        }
    }
}

impl std::error::Error for GameSessionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use linera_sdk::linera_base_types::Timestamp;

    #[test]
    fn test_create_classic_session() {
        let session_id = GameSessionId(1);
        let participant_id = ParticipantId(100);
        let current_time = Timestamp::from(1000000);

        let result = GameSessionService::create_session(
            session_id,
            participant_id,
            GameVariant::Classic2048,
            current_time,
            None,
        );

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.session_id, session_id);
        assert_eq!(session.participant_id, participant_id);
        assert_eq!(session.game_variant, GameVariant::Classic2048);
        assert!(session.is_active());
    }

    #[test]
    fn test_create_speed_session() {
        let session_id = GameSessionId(1);
        let participant_id = ParticipantId(100);
        let current_time = Timestamp::from(1000000);

        let result = GameSessionService::create_session(
            session_id,
            participant_id,
            GameVariant::Speed2048 { time_limit_seconds: 300 },
            current_time,
            None,
        );

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.session_lifecycle.duration_limit, Some(300_000_000)); // 5 minutes in microseconds
    }

    #[test]
    fn test_invalid_speed_variant() {
        let session_id = GameSessionId(1);
        let participant_id = ParticipantId(100);
        let current_time = Timestamp::from(1000000);

        // Test invalid time limit (too high)
        let result = GameSessionService::create_session(
            session_id,
            participant_id,
            GameVariant::Speed2048 { time_limit_seconds: 5000 },
            current_time,
            None,
        );

        assert!(matches!(result, Err(GameSessionError::InvalidVariantConfig(_))));

        // Test invalid time limit (zero)
        let result = GameSessionService::create_session(
            session_id,
            participant_id,
            GameVariant::Speed2048 { time_limit_seconds: 0 },
            current_time,
            None,
        );

        assert!(matches!(result, Err(GameSessionError::InvalidVariantConfig(_))));
    }

    #[test]
    fn test_process_move_inactive_session() {
        let session_id = GameSessionId(1);
        let participant_id = ParticipantId(100);
        let current_time = Timestamp::from(1000000);

        let mut session = GameSession::new(
            session_id,
            participant_id,
            GameVariant::Classic2048,
            current_time,
            None,
        );

        // Complete the session first
        session.complete_session(SessionStatus::CompletedSuccessfully, current_time);

        // Try to process a move on inactive session
        let result = GameSessionService::process_move(&mut session, Direction::Up, current_time);
        assert!(matches!(result, Err(GameSessionError::SessionNotActive)));
    }

    #[test]
    fn test_abandon_session() {
        let session_id = GameSessionId(1);
        let participant_id = ParticipantId(100);
        let current_time = Timestamp::from(1000000);

        let mut session = GameSession::new(
            session_id,
            participant_id,
            GameVariant::Classic2048,
            current_time,
            None,
        );

        assert!(session.is_active());

        let abandon_time = Timestamp::from(1500000);
        let result = GameSessionService::abandon_session(&mut session, abandon_time);
        
        assert!(result.is_ok());
        assert!(!session.is_active());
        assert_eq!(session.session_status, SessionStatus::AbandonedByPlayer);
        assert_eq!(session.session_lifecycle.concluded_at, Some(abandon_time));
    }

    #[test]
    fn test_calculate_final_score() {
        let session_id = GameSessionId(1);
        let participant_id = ParticipantId(100);
        let current_time = Timestamp::from(1000000);

        let mut session = GameSession::new(
            session_id,
            participant_id,
            GameVariant::Classic2048,
            current_time,
            None,
        );

        // Set base score
        session.scoring_metrics.primary_score = 1000;

        // Add a multiplier
        session.scoring_metrics.bonus_multipliers.push(BonusMultiplier {
            multiplier_type: MultiplierType::SpeedBonus,
            value: 1.5,
            applied_at: current_time,
        });

        // Add achievements
        session.scoring_metrics.achievement_unlocks.push(Achievement {
            achievement_id: "first_move".to_string(),
            name: "First Move".to_string(),
            description: "Made your first move".to_string(),
            unlocked_at: current_time,
        });

        let final_score = GameSessionService::calculate_final_score(&session);
        
        // Expected: (1000 * 1.5) + (1 * 100) = 1500 + 100 = 1600
        assert_eq!(final_score, 1600);
    }
}