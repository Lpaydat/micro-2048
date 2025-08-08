//! Game session domain model with improved naming and extensibility.

use serde::{Deserialize, Serialize};
use linera_sdk::linera_base_types::Timestamp;
use crate::core::value_objects::*;

/// Represents a single game session with improved naming conventions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub session_id: GameSessionId,
    pub participant_id: ParticipantId,
    pub game_variant: GameVariant,
    pub board_state: BoardState,
    pub scoring_metrics: ScoringMetrics,
    pub session_status: SessionStatus,
    pub session_lifecycle: SessionLifecycle,
    pub competition_context: Option<CompetitionContext>,
}

impl GameSession {
    /// Create a new game session with deterministic initial state
    pub fn new(
        session_id: GameSessionId,
        participant_id: ParticipantId,
        game_variant: GameVariant,
        current_time: Timestamp,
        competition_context: Option<CompetitionContext>,
    ) -> Self {
        Self {
            session_id,
            participant_id,
            game_variant,
            board_state: BoardState::new(),
            scoring_metrics: ScoringMetrics::default(),
            session_status: SessionStatus::InProgress,
            session_lifecycle: SessionLifecycle {
                initiated_at: current_time,
                last_activity_at: current_time,
                concluded_at: None,
                duration_limit: Self::get_duration_limit(&game_variant),
            },
            competition_context,
        }
    }

    /// Get duration limit based on game variant
    fn get_duration_limit(variant: &GameVariant) -> Option<u64> {
        match variant {
            GameVariant::Speed2048 { time_limit_seconds } => {
                Some(*time_limit_seconds as u64 * 1_000_000) // Convert to microseconds
            }
            _ => None, // No time limit for other variants
        }
    }

    /// Check if the session is active
    pub fn is_active(&self) -> bool {
        matches!(self.session_status, SessionStatus::InProgress)
    }

    /// Check if the session has exceeded its time limit
    pub fn is_time_expired(&self, current_time: Timestamp) -> bool {
        if let Some(duration_limit) = self.session_lifecycle.duration_limit {
            let elapsed = current_time.micros() - self.session_lifecycle.initiated_at.micros();
            elapsed > duration_limit
        } else {
            false
        }
    }

    /// Update the last activity timestamp
    pub fn update_activity(&mut self, current_time: Timestamp) {
        self.session_lifecycle.last_activity_at = current_time;
    }

    /// Complete the session with the given status
    pub fn complete_session(&mut self, status: SessionStatus, current_time: Timestamp) {
        self.session_status = status;
        self.session_lifecycle.concluded_at = Some(current_time);
        self.session_lifecycle.last_activity_at = current_time;
    }

    /// Get the total play time in microseconds
    pub fn get_play_time(&self, current_time: Option<Timestamp>) -> u64 {
        let end_time = self.session_lifecycle.concluded_at
            .or(current_time)
            .unwrap_or(self.session_lifecycle.last_activity_at);
        
        end_time.micros() - self.session_lifecycle.initiated_at.micros()
    }
}

/// Context information when a game session is part of a competition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionContext {
    pub competition_id: CompetitionId,
    pub competition_phase: CompetitionPhase,
    pub participant_rank: Option<u32>,
    pub elimination_threshold: Option<u64>,
}

/// Result of a move operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveResult {
    pub new_board_state: BoardState,
    pub score_delta: u64,
    pub tiles_merged: Vec<(u32, u32)>, // (position, value) of merged tiles
    pub new_tile_spawned: Option<(u32, u32)>, // (position, value) of new tile
    pub game_ended: bool,
    pub achievements_unlocked: Vec<Achievement>,
}

impl MoveResult {
    pub fn no_change() -> Self {
        Self {
            new_board_state: BoardState::new(),
            score_delta: 0,
            tiles_merged: Vec::new(),
            new_tile_spawned: None,
            game_ended: false,
            achievements_unlocked: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linera_sdk::linera_base_types::Timestamp;

    #[test]
    fn test_game_session_creation() {
        let session_id = GameSessionId(1);
        let participant_id = ParticipantId(100);
        let current_time = Timestamp::from(1000000);
        
        let session = GameSession::new(
            session_id,
            participant_id,
            GameVariant::Classic2048,
            current_time,
            None,
        );

        assert_eq!(session.session_id, session_id);
        assert_eq!(session.participant_id, participant_id);
        assert_eq!(session.game_variant, GameVariant::Classic2048);
        assert_eq!(session.session_status, SessionStatus::InProgress);
        assert!(session.is_active());
        assert_eq!(session.session_lifecycle.initiated_at, current_time);
        assert_eq!(session.session_lifecycle.last_activity_at, current_time);
        assert!(session.session_lifecycle.concluded_at.is_none());
    }

    #[test]
    fn test_speed_game_time_limit() {
        let session_id = GameSessionId(1);
        let participant_id = ParticipantId(100);
        let current_time = Timestamp::from(1000000);
        
        let session = GameSession::new(
            session_id,
            participant_id,
            GameVariant::Speed2048 { time_limit_seconds: 300 }, // 5 minutes
            current_time,
            None,
        );

        assert_eq!(session.session_lifecycle.duration_limit, Some(300_000_000)); // 5 minutes in microseconds
        
        // Test time expiration
        let future_time = Timestamp::from(1000000 + 400_000_000); // 6 minutes later
        assert!(session.is_time_expired(future_time));
        
        let near_future = Timestamp::from(1000000 + 200_000_000); // 3 minutes later
        assert!(!session.is_time_expired(near_future));
    }

    #[test]
    fn test_session_completion() {
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
        
        let completion_time = Timestamp::from(2000000);
        session.complete_session(SessionStatus::CompletedSuccessfully, completion_time);
        
        assert!(!session.is_active());
        assert_eq!(session.session_status, SessionStatus::CompletedSuccessfully);
        assert_eq!(session.session_lifecycle.concluded_at, Some(completion_time));
        assert_eq!(session.get_play_time(None), 1000000); // 1 second in microseconds
    }

    #[test]
    fn test_activity_update() {
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

        let new_time = Timestamp::from(1500000);
        session.update_activity(new_time);
        
        assert_eq!(session.session_lifecycle.last_activity_at, new_time);
        assert_eq!(session.session_lifecycle.initiated_at, current_time); // Should not change
    }
}