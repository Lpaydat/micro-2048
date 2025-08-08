//! Participant domain service for managing participant lifecycle and business rules.

use crate::core::models::*;
use crate::core::value_objects::*;
use linera_sdk::linera_base_types::{ChainId, Timestamp};

/// Domain service for participant management
pub struct ParticipantService;

impl ParticipantService {
    /// Register a new participant with validation
    pub fn register_participant(
        participant_id: ParticipantId,
        username: String,
        home_chain_id: ChainId,
        current_time: Timestamp,
    ) -> Result<Participant, ParticipantServiceError> {
        // Create participant with validation
        let participant = Participant::new(participant_id, username, home_chain_id, current_time)
            .map_err(ParticipantServiceError::ParticipantError)?;

        Ok(participant)
    }

    /// Update participant profile information
    pub fn update_profile(
        participant: &mut Participant,
        display_name: Option<String>,
        avatar_hash: Option<String>,
        current_time: Timestamp,
    ) -> Result<(), ParticipantServiceError> {
        // Validate and update display name
        participant.update_display_name(display_name)
            .map_err(ParticipantServiceError::ParticipantError)?;

        // Update avatar hash
        participant.set_avatar_hash(avatar_hash);

        // Update activity timestamp
        participant.update_activity(current_time);

        Ok(())
    }

    /// Process game completion for a participant
    pub fn process_game_completion(
        participant: &mut Participant,
        session: &GameSession,
        current_time: Timestamp,
    ) -> Result<Vec<Achievement>, ParticipantServiceError> {
        if !participant.is_active() {
            return Err(ParticipantServiceError::ParticipantNotActive);
        }

        // Calculate final score
        let final_score = crate::core::services::GameSessionService::calculate_final_score(session);

        // Record the game session
        participant.record_game_session(final_score, current_time);

        // Check for new achievements
        let mut new_achievements = Vec::new();

        // First game achievement
        if participant.participation_history.total_sessions_played == 1 {
            let achievement = Achievement {
                achievement_id: "first_game".to_string(),
                name: "First Game".to_string(),
                description: "Completed your first game".to_string(),
                unlocked_at: current_time,
            };
            participant.unlock_achievement(achievement.clone());
            new_achievements.push(achievement);
        }

        // High score achievements
        if final_score >= 10000 && !Self::has_achievement(participant, "high_scorer") {
            let achievement = Achievement {
                achievement_id: "high_scorer".to_string(),
                name: "High Scorer".to_string(),
                description: "Achieved a score of 10,000 or more".to_string(),
                unlocked_at: current_time,
            };
            participant.unlock_achievement(achievement.clone());
            new_achievements.push(achievement);
        }

        // 2048 tile achievement
        if session.board_state.highest_tile_achieved >= 2048 && !Self::has_achievement(participant, "tile_2048") {
            let achievement = Achievement {
                achievement_id: "tile_2048".to_string(),
                name: "2048 Master".to_string(),
                description: "Reached the 2048 tile".to_string(),
                unlocked_at: current_time,
            };
            participant.unlock_achievement(achievement.clone());
            new_achievements.push(achievement);
        }

        // Veteran player achievement
        if participant.participation_history.total_sessions_played >= 100 && !Self::has_achievement(participant, "veteran") {
            let achievement = Achievement {
                achievement_id: "veteran".to_string(),
                name: "Veteran Player".to_string(),
                description: "Played 100 games".to_string(),
                unlocked_at: current_time,
            };
            participant.unlock_achievement(achievement.clone());
            new_achievements.push(achievement);
        }

        Ok(new_achievements)
    }

    /// Process competition entry for a participant
    pub fn process_competition_entry(
        participant: &mut Participant,
        competition: &Competition,
        current_time: Timestamp,
    ) -> Result<(), ParticipantServiceError> {
        if !participant.is_active() {
            return Err(ParticipantServiceError::ParticipantNotActive);
        }

        // Check eligibility
        competition.can_participant_join(participant, current_time)
            .map_err(ParticipantServiceError::CompetitionError)?;

        // Record competition entry
        participant.record_competition_entry(competition.competition_id, current_time);

        // Competition entry achievement
        if participant.participation_history.competitions_entered == 1 {
            let achievement = Achievement {
                achievement_id: "first_competition".to_string(),
                name: "Competitor".to_string(),
                description: "Entered your first competition".to_string(),
                unlocked_at: current_time,
            };
            participant.unlock_achievement(achievement);
        }

        Ok(())
    }

    /// Ban a participant
    pub fn ban_participant(
        participant: &mut Participant,
        reason: BanReason,
        current_time: Timestamp,
    ) -> Result<(), ParticipantServiceError> {
        participant.ban(reason);
        participant.update_activity(current_time);
        Ok(())
    }

    /// Suspend a participant
    pub fn suspend_participant(
        participant: &mut Participant,
        until: Timestamp,
        current_time: Timestamp,
    ) -> Result<(), ParticipantServiceError> {
        if until <= current_time {
            return Err(ParticipantServiceError::InvalidSuspensionPeriod);
        }

        participant.suspend(until);
        participant.update_activity(current_time);
        Ok(())
    }

    /// Reactivate a banned or suspended participant
    pub fn reactivate_participant(
        participant: &mut Participant,
        current_time: Timestamp,
    ) -> Result<(), ParticipantServiceError> {
        participant.reactivate();
        participant.update_activity(current_time);
        Ok(())
    }

    /// Calculate participant skill rating based on performance
    pub fn calculate_skill_rating(participant: &Participant) -> Option<u32> {
        if participant.participation_history.total_sessions_played < 10 {
            // Need at least 10 games for a rating
            return None;
        }

        // Simple rating calculation based on average score and games played
        let base_rating = (participant.skill_metrics.average_score / 10.0) as u32;
        let experience_bonus = (participant.participation_history.total_sessions_played / 10).min(100);
        let personal_best_bonus = (participant.skill_metrics.personal_best_score / 1000).min(500) as u32;

        Some(base_rating + experience_bonus + personal_best_bonus)
    }

    /// Update participant skill rating
    pub fn update_skill_rating(participant: &mut Participant) {
        participant.skill_metrics.skill_rating = Self::calculate_skill_rating(participant);
    }

    /// Check if participant has a specific achievement
    fn has_achievement(participant: &Participant, achievement_id: &str) -> bool {
        participant.participation_history.achievements_earned
            .iter()
            .any(|a| a.achievement_id == achievement_id)
    }

    /// Get participant statistics summary
    pub fn get_statistics_summary(participant: &Participant) -> ParticipantStatistics {
        ParticipantStatistics {
            participant_id: participant.participant_id,
            total_games: participant.participation_history.total_sessions_played,
            total_competitions: participant.participation_history.competitions_entered,
            personal_best: participant.skill_metrics.personal_best_score,
            average_score: participant.skill_metrics.average_score,
            skill_rating: participant.skill_metrics.skill_rating,
            achievements_count: participant.participation_history.achievements_earned.len() as u32,
            account_age_days: Self::calculate_account_age_days(participant),
        }
    }

    /// Calculate account age in days
    fn calculate_account_age_days(participant: &Participant) -> u32 {
        let current_time = participant.participation_history.last_activity_at;
        let account_age_micros = current_time.micros() - participant.participation_history.account_created_at.micros();
        (account_age_micros / (24 * 60 * 60 * 1_000_000)) as u32 // Convert to days
    }
}

/// Statistics summary for a participant
#[derive(Debug, Clone)]
pub struct ParticipantStatistics {
    pub participant_id: ParticipantId,
    pub total_games: u32,
    pub total_competitions: u32,
    pub personal_best: u64,
    pub average_score: f64,
    pub skill_rating: Option<u32>,
    pub achievements_count: u32,
    pub account_age_days: u32,
}

/// Errors that can occur in participant service operations
#[derive(Debug, Clone)]
pub enum ParticipantServiceError {
    ParticipantError(ParticipantError),
    CompetitionError(CompetitionError),
    ParticipantNotActive,
    InvalidSuspensionPeriod,
    UnexpectedError(String),
}

impl std::fmt::Display for ParticipantServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParticipantServiceError::ParticipantError(err) => write!(f, "Participant error: {}", err),
            ParticipantServiceError::CompetitionError(err) => write!(f, "Competition error: {}", err),
            ParticipantServiceError::ParticipantNotActive => write!(f, "Participant is not active"),
            ParticipantServiceError::InvalidSuspensionPeriod => write!(f, "Invalid suspension period"),
            ParticipantServiceError::UnexpectedError(msg) => write!(f, "Unexpected error: {}", msg),
        }
    }
}

impl std::error::Error for ParticipantServiceError {}

#[cfg(test)]
mod tests {
    use super::*;
    use linera_sdk::linera_base_types::{ChainId, Timestamp};

    #[test]
    fn test_register_participant() {
        let participant_id = ParticipantId(1);
        let username = "test_user".to_string();
        let home_chain = ChainId::root(0);
        let current_time = Timestamp::from(1000000);

        let result = ParticipantService::register_participant(
            participant_id,
            username.clone(),
            home_chain,
            current_time,
        );

        assert!(result.is_ok());
        let participant = result.unwrap();
        assert_eq!(participant.participant_id, participant_id);
        assert_eq!(participant.display_identity.username, username);
        assert!(participant.is_active());
    }

    #[test]
    fn test_update_profile() {
        let participant_id = ParticipantId(1);
        let username = "test_user".to_string();
        let home_chain = ChainId::root(0);
        let current_time = Timestamp::from(1000000);

        let mut participant = Participant::new(participant_id, username, home_chain, current_time)
            .expect("Participant creation should succeed");

        let update_time = Timestamp::from(1500000);
        let result = ParticipantService::update_profile(
            &mut participant,
            Some("Display Name".to_string()),
            Some("avatar123".to_string()),
            update_time,
        );

        assert!(result.is_ok());
        assert_eq!(participant.display_identity.display_name, Some("Display Name".to_string()));
        assert_eq!(participant.display_identity.avatar_hash, Some("avatar123".to_string()));
        assert_eq!(participant.participation_history.last_activity_at, update_time);
    }

    #[test]
    fn test_process_game_completion() {
        let participant_id = ParticipantId(1);
        let username = "test_user".to_string();
        let home_chain = ChainId::root(0);
        let current_time = Timestamp::from(1000000);

        let mut participant = Participant::new(participant_id, username, home_chain, current_time)
            .expect("Participant creation should succeed");

        // Create a completed game session
        let session_id = GameSessionId(1);
        let mut session = GameSession::new(
            session_id,
            participant_id,
            GameVariant::Classic2048,
            current_time,
            None,
        );
        session.scoring_metrics.primary_score = 5000;
        session.board_state.highest_tile_achieved = 1024;

        let completion_time = Timestamp::from(1500000);
        let result = ParticipantService::process_game_completion(
            &mut participant,
            &session,
            completion_time,
        );

        assert!(result.is_ok());
        let achievements = result.unwrap();
        
        // Should get first game achievement
        assert_eq!(achievements.len(), 1);
        assert_eq!(achievements[0].achievement_id, "first_game");
        
        assert_eq!(participant.participation_history.total_sessions_played, 1);
        assert_eq!(participant.skill_metrics.personal_best_score, 5000);
        assert_eq!(participant.skill_metrics.average_score, 5000.0);
    }

    #[test]
    fn test_skill_rating_calculation() {
        let participant_id = ParticipantId(1);
        let username = "test_user".to_string();
        let home_chain = ChainId::root(0);
        let current_time = Timestamp::from(1000000);

        let mut participant = Participant::new(participant_id, username, home_chain, current_time)
            .expect("Participant creation should succeed");

        // Not enough games for rating
        let rating = ParticipantService::calculate_skill_rating(&participant);
        assert!(rating.is_none());

        // Set up participant with enough games
        participant.participation_history.total_sessions_played = 20;
        participant.skill_metrics.average_score = 3000.0;
        participant.skill_metrics.personal_best_score = 8000;

        let rating = ParticipantService::calculate_skill_rating(&participant);
        assert!(rating.is_some());
        
        let rating_value = rating.unwrap();
        // Expected: (3000/10) + (20/10) + (8000/1000) = 300 + 2 + 8 = 310
        assert_eq!(rating_value, 310);
    }

    #[test]
    fn test_participant_suspension() {
        let participant_id = ParticipantId(1);
        let username = "test_user".to_string();
        let home_chain = ChainId::root(0);
        let current_time = Timestamp::from(1000000);

        let mut participant = Participant::new(participant_id, username, home_chain, current_time)
            .expect("Participant creation should succeed");

        assert!(participant.is_active());

        // Suspend participant
        let suspension_end = Timestamp::from(2000000);
        let result = ParticipantService::suspend_participant(
            &mut participant,
            suspension_end,
            current_time,
        );

        assert!(result.is_ok());
        assert!(!participant.is_active());
        assert!(participant.is_suspended(current_time));

        // Test invalid suspension period (in the past)
        let past_time = Timestamp::from(500000);
        let result = ParticipantService::suspend_participant(
            &mut participant,
            past_time,
            current_time,
        );

        assert!(matches!(result, Err(ParticipantServiceError::InvalidSuspensionPeriod)));
    }

    #[test]
    fn test_participant_ban_and_reactivate() {
        let participant_id = ParticipantId(1);
        let username = "test_user".to_string();
        let home_chain = ChainId::root(0);
        let current_time = Timestamp::from(1000000);

        let mut participant = Participant::new(participant_id, username, home_chain, current_time)
            .expect("Participant creation should succeed");

        assert!(participant.is_active());

        // Ban participant
        let result = ParticipantService::ban_participant(
            &mut participant,
            BanReason::Cheating,
            current_time,
        );

        assert!(result.is_ok());
        assert!(!participant.is_active());
        assert!(participant.is_banned());

        // Reactivate participant
        let reactivate_time = Timestamp::from(1500000);
        let result = ParticipantService::reactivate_participant(
            &mut participant,
            reactivate_time,
        );

        assert!(result.is_ok());
        assert!(participant.is_active());
        assert!(!participant.is_banned());
    }
}