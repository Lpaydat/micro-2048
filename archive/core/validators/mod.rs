//! Domain validators for enforcing business rules and data integrity.
//! These validators contain pure functions that validate domain entities and operations.

use crate::core::models::*;
use crate::core::value_objects::*;
use linera_sdk::linera_base_types::Timestamp;

/// Validation errors that can occur during domain validation
#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidInput(String),
    BusinessRuleViolation(String),
    TimeConstraintViolation(String),
    StateConstraintViolation(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ValidationError::BusinessRuleViolation(msg) => write!(f, "Business rule violation: {}", msg),
            ValidationError::TimeConstraintViolation(msg) => write!(f, "Time constraint violation: {}", msg),
            ValidationError::StateConstraintViolation(msg) => write!(f, "State constraint violation: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validator for game session operations
pub struct GameSessionValidator;

impl GameSessionValidator {
    /// Validate that a move is valid for the current board state
    pub fn validate_move(
        board_state: &BoardState,
        direction: Direction,
    ) -> Result<(), ValidationError> {
        // Check if board state is valid
        if board_state.tiles == 0 {
            return Err(ValidationError::StateConstraintViolation(
                "Board state is empty".to_string()
            ));
        }

        // For now, we'll assume all moves are potentially valid
        // The actual move validation will be implemented with the game logic
        Ok(())
    }

    /// Validate game session timing constraints
    pub fn validate_session_timing(
        session: &GameSession,
        current_time: Timestamp,
    ) -> Result<(), ValidationError> {
        // Check if session is not in the future
        if session.session_lifecycle.initiated_at > current_time {
            return Err(ValidationError::TimeConstraintViolation(
                "Session cannot be initiated in the future".to_string()
            ));
        }

        // Check if last activity is not before initiation
        if session.session_lifecycle.last_activity_at < session.session_lifecycle.initiated_at {
            return Err(ValidationError::TimeConstraintViolation(
                "Last activity cannot be before session initiation".to_string()
            ));
        }

        // Check if concluded time is valid
        if let Some(concluded_at) = session.session_lifecycle.concluded_at {
            if concluded_at < session.session_lifecycle.initiated_at {
                return Err(ValidationError::TimeConstraintViolation(
                    "Session conclusion cannot be before initiation".to_string()
                ));
            }
        }

        // Check time limits for timed variants
        if let Some(duration_limit) = session.session_lifecycle.duration_limit {
            let elapsed = current_time.micros() - session.session_lifecycle.initiated_at.micros();
            if elapsed > duration_limit && session.is_active() {
                return Err(ValidationError::TimeConstraintViolation(
                    "Session has exceeded its time limit".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Validate game variant configuration
    pub fn validate_game_variant(variant: &GameVariant) -> Result<(), ValidationError> {
        match variant {
            GameVariant::Speed2048 { time_limit_seconds } => {
                if *time_limit_seconds == 0 {
                    return Err(ValidationError::InvalidInput(
                        "Speed game time limit cannot be zero".to_string()
                    ));
                }
                if *time_limit_seconds > 7200 {
                    return Err(ValidationError::InvalidInput(
                        "Speed game time limit cannot exceed 2 hours".to_string()
                    ));
                }
            }
            GameVariant::Elimination { lives_remaining } => {
                if *lives_remaining == 0 {
                    return Err(ValidationError::InvalidInput(
                        "Elimination game must have at least 1 life".to_string()
                    ));
                }
                if *lives_remaining > 20 {
                    return Err(ValidationError::InvalidInput(
                        "Elimination game cannot have more than 20 lives".to_string()
                    ));
                }
            }
            GameVariant::Collaborative { team_size } => {
                if *team_size < 2 {
                    return Err(ValidationError::InvalidInput(
                        "Collaborative game must have at least 2 team members".to_string()
                    ));
                }
                if *team_size > 10 {
                    return Err(ValidationError::InvalidInput(
                        "Collaborative game cannot have more than 10 team members".to_string()
                    ));
                }
            }
            GameVariant::Classic2048 => {
                // No additional validation needed for classic variant
            }
        }
        Ok(())
    }

    /// Validate session state consistency
    pub fn validate_session_state(session: &GameSession) -> Result<(), ValidationError> {
        // Check that move count is consistent with board state
        if session.board_state.move_count > 10000 {
            return Err(ValidationError::StateConstraintViolation(
                "Move count is unreasonably high".to_string()
            ));
        }

        // Check that highest tile achieved is reasonable
        if session.board_state.highest_tile_achieved > 131072 {
            return Err(ValidationError::StateConstraintViolation(
                "Highest tile achieved is unreasonably high".to_string()
            ));
        }

        // Check that score is reasonable relative to moves
        let max_reasonable_score = session.board_state.move_count as u64 * 1000;
        if session.scoring_metrics.primary_score > max_reasonable_score {
            return Err(ValidationError::StateConstraintViolation(
                "Score is unreasonably high for the number of moves".to_string()
            ));
        }

        Ok(())
    }
}

/// Validator for participant operations
pub struct ParticipantValidator;

impl ParticipantValidator {
    /// Validate participant registration data
    pub fn validate_registration(
        username: &str,
        current_time: Timestamp,
    ) -> Result<(), ValidationError> {
        // Username validation
        if username.is_empty() {
            return Err(ValidationError::InvalidInput(
                "Username cannot be empty".to_string()
            ));
        }

        if username.len() < 3 {
            return Err(ValidationError::InvalidInput(
                "Username must be at least 3 characters long".to_string()
            ));
        }

        if username.len() > 30 {
            return Err(ValidationError::InvalidInput(
                "Username cannot exceed 30 characters".to_string()
            ));
        }

        // Check for valid characters
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ValidationError::InvalidInput(
                "Username can only contain letters, numbers, underscores, and hyphens".to_string()
            ));
        }

        // Check for reserved usernames
        let reserved_usernames = ["admin", "system", "root", "moderator", "support"];
        if reserved_usernames.contains(&username.to_lowercase().as_str()) {
            return Err(ValidationError::BusinessRuleViolation(
                "Username is reserved".to_string()
            ));
        }

        Ok(())
    }

    /// Validate participant profile update
    pub fn validate_profile_update(
        display_name: &Option<String>,
        avatar_hash: &Option<String>,
    ) -> Result<(), ValidationError> {
        // Display name validation
        if let Some(name) = display_name {
            if name.is_empty() {
                return Err(ValidationError::InvalidInput(
                    "Display name cannot be empty".to_string()
                ));
            }
            if name.len() > 50 {
                return Err(ValidationError::InvalidInput(
                    "Display name cannot exceed 50 characters".to_string()
                ));
            }
            // Check for inappropriate content (basic check)
            if name.to_lowercase().contains("admin") || name.to_lowercase().contains("system") {
                return Err(ValidationError::BusinessRuleViolation(
                    "Display name contains reserved words".to_string()
                ));
            }
        }

        // Avatar hash validation
        if let Some(hash) = avatar_hash {
            if hash.len() != 64 {
                return Err(ValidationError::InvalidInput(
                    "Avatar hash must be 64 characters long".to_string()
                ));
            }
            if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(ValidationError::InvalidInput(
                    "Avatar hash must contain only hexadecimal characters".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Validate participant account status change
    pub fn validate_account_status_change(
        current_status: &AccountStatus,
        new_status: &AccountStatus,
        current_time: Timestamp,
    ) -> Result<(), ValidationError> {
        match (current_status, new_status) {
            (AccountStatus::Active, AccountStatus::Suspended { until }) => {
                if *until <= current_time {
                    return Err(ValidationError::TimeConstraintViolation(
                        "Suspension end time must be in the future".to_string()
                    ));
                }
            }
            (AccountStatus::Banned { .. }, AccountStatus::Active) => {
                // Unbanning is allowed
            }
            (AccountStatus::Suspended { .. }, AccountStatus::Active) => {
                // Unsuspending is allowed
            }
            (AccountStatus::Active, AccountStatus::Banned { .. }) => {
                // Banning is allowed
            }
            _ => {
                // Other transitions might need validation
            }
        }

        Ok(())
    }

    /// Validate participant eligibility for activities
    pub fn validate_activity_eligibility(
        participant: &Participant,
        current_time: Timestamp,
    ) -> Result<(), ValidationError> {
        if !participant.is_active() {
            if participant.is_banned() {
                return Err(ValidationError::BusinessRuleViolation(
                    "Participant is banned and cannot participate in activities".to_string()
                ));
            }
            if participant.is_suspended(current_time) {
                return Err(ValidationError::BusinessRuleViolation(
                    "Participant is suspended and cannot participate in activities".to_string()
                ));
            }
            return Err(ValidationError::BusinessRuleViolation(
                "Participant account is not active".to_string()
            ));
        }

        Ok(())
    }
}

/// Validator for competition operations
pub struct CompetitionValidator;

impl CompetitionValidator {
    /// Validate competition creation parameters
    pub fn validate_competition_creation(
        title: &str,
        start_time: Timestamp,
        end_time: Timestamp,
        current_time: Timestamp,
    ) -> Result<(), ValidationError> {
        // Title validation
        if title.is_empty() {
            return Err(ValidationError::InvalidInput(
                "Competition title cannot be empty".to_string()
            ));
        }

        if title.len() > 100 {
            return Err(ValidationError::InvalidInput(
                "Competition title cannot exceed 100 characters".to_string()
            ));
        }

        // Time validation
        if start_time <= current_time {
            return Err(ValidationError::TimeConstraintViolation(
                "Competition start time must be in the future".to_string()
            ));
        }

        if end_time <= start_time {
            return Err(ValidationError::TimeConstraintViolation(
                "Competition end time must be after start time".to_string()
            ));
        }

        // Check minimum duration (1 hour)
        let min_duration = 3600_000_000; // 1 hour in microseconds
        if end_time.micros() - start_time.micros() < min_duration {
            return Err(ValidationError::TimeConstraintViolation(
                "Competition must last at least 1 hour".to_string()
            ));
        }

        // Check maximum duration (30 days)
        let max_duration = 30 * 24 * 3600_000_000; // 30 days in microseconds
        if end_time.micros() - start_time.micros() > max_duration {
            return Err(ValidationError::TimeConstraintViolation(
                "Competition cannot last more than 30 days".to_string()
            ));
        }

        Ok(())
    }

    /// Validate competition format configuration
    pub fn validate_competition_format(format: &CompetitionFormat) -> Result<(), ValidationError> {
        match format {
            CompetitionFormat::SingleElimination { bracket_size, .. } => {
                if *bracket_size < 4 {
                    return Err(ValidationError::InvalidInput(
                        "Single elimination tournament must have at least 4 participants".to_string()
                    ));
                }
                if *bracket_size > 1024 {
                    return Err(ValidationError::InvalidInput(
                        "Single elimination tournament cannot exceed 1024 participants".to_string()
                    ));
                }
                // Check if bracket size is a power of 2
                if !bracket_size.is_power_of_two() {
                    return Err(ValidationError::InvalidInput(
                        "Single elimination bracket size must be a power of 2".to_string()
                    ));
                }
            }
            CompetitionFormat::RoundRobin { rounds_count, .. } => {
                if *rounds_count == 0 {
                    return Err(ValidationError::InvalidInput(
                        "Round robin tournament must have at least 1 round".to_string()
                    ));
                }
                if *rounds_count > 20 {
                    return Err(ValidationError::InvalidInput(
                        "Round robin tournament cannot have more than 20 rounds".to_string()
                    ));
                }
            }
            CompetitionFormat::TimeBasedLeaderboard { duration, .. } => {
                let min_duration = 3600_000_000; // 1 hour
                let max_duration = 7 * 24 * 3600_000_000; // 7 days
                if *duration < min_duration {
                    return Err(ValidationError::InvalidInput(
                        "Leaderboard competition must last at least 1 hour".to_string()
                    ));
                }
                if *duration > max_duration {
                    return Err(ValidationError::InvalidInput(
                        "Leaderboard competition cannot last more than 7 days".to_string()
                    ));
                }
            }
            CompetitionFormat::EliminationSurvival { elimination_threshold, elimination_interval } => {
                if *elimination_threshold == 0 {
                    return Err(ValidationError::InvalidInput(
                        "Elimination threshold must be greater than 0".to_string()
                    ));
                }
                let min_interval = 300_000_000; // 5 minutes
                if *elimination_interval < min_interval {
                    return Err(ValidationError::InvalidInput(
                        "Elimination interval must be at least 5 minutes".to_string()
                    ));
                }
            }
            CompetitionFormat::TeamBased { team_size, .. } => {
                if *team_size < 2 {
                    return Err(ValidationError::InvalidInput(
                        "Team-based competition must have at least 2 members per team".to_string()
                    ));
                }
                if *team_size > 8 {
                    return Err(ValidationError::InvalidInput(
                        "Team-based competition cannot have more than 8 members per team".to_string()
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate participant eligibility for competition
    pub fn validate_competition_eligibility(
        competition: &Competition,
        participant: &Participant,
        current_time: Timestamp,
    ) -> Result<(), ValidationError> {
        // Check if registration is open
        if !competition.is_registration_open(current_time) {
            return Err(ValidationError::BusinessRuleViolation(
                "Competition registration is not open".to_string()
            ));
        }

        // Check participant status
        ParticipantValidator::validate_activity_eligibility(participant, current_time)?;

        // Check minimum skill rating
        if let Some(min_rating) = competition.participation_rules.min_skill_rating {
            match participant.skill_metrics.skill_rating {
                Some(rating) if rating >= min_rating => {
                    // Participant meets the requirement
                }
                Some(rating) => {
                    return Err(ValidationError::BusinessRuleViolation(
                        format!("Participant skill rating {} is below minimum required {}", rating, min_rating)
                    ));
                }
                None => {
                    return Err(ValidationError::BusinessRuleViolation(
                        "Participant does not have a skill rating".to_string()
                    ));
                }
            }
        }

        // Check entry requirements
        for requirement in &competition.participation_rules.entry_requirements {
            match requirement {
                EntryRequirement::MinimumGamesPlayed(min_games) => {
                    if participant.participation_history.total_sessions_played < *min_games {
                        return Err(ValidationError::BusinessRuleViolation(
                            format!("Participant has played {} games but {} required", 
                                participant.participation_history.total_sessions_played, min_games)
                        ));
                    }
                }
                EntryRequirement::MinimumScore(min_score) => {
                    if participant.skill_metrics.personal_best_score < *min_score {
                        return Err(ValidationError::BusinessRuleViolation(
                            format!("Participant best score {} is below minimum required {}", 
                                participant.skill_metrics.personal_best_score, min_score)
                        ));
                    }
                }
                EntryRequirement::AchievementRequired(achievement_id) => {
                    if !participant.participation_history.achievements_earned
                        .iter()
                        .any(|a| a.achievement_id == *achievement_id) {
                        return Err(ValidationError::BusinessRuleViolation(
                            format!("Participant does not have required achievement: {}", achievement_id)
                        ));
                    }
                }
                EntryRequirement::InvitationRequired => {
                    // This would need to be checked against an invitation system
                    // For now, we'll assume it's handled elsewhere
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linera_sdk::base::{ChainId, Timestamp};

    #[test]
    fn test_validate_username() {
        // Valid usernames
        assert!(ParticipantValidator::validate_registration("valid_user", Timestamp::from(1000000)).is_ok());
        assert!(ParticipantValidator::validate_registration("user123", Timestamp::from(1000000)).is_ok());
        assert!(ParticipantValidator::validate_registration("test-user", Timestamp::from(1000000)).is_ok());

        // Invalid usernames
        assert!(ParticipantValidator::validate_registration("", Timestamp::from(1000000)).is_err());
        assert!(ParticipantValidator::validate_registration("ab", Timestamp::from(1000000)).is_err());
        assert!(ParticipantValidator::validate_registration(&"a".repeat(31), Timestamp::from(1000000)).is_err());
        assert!(ParticipantValidator::validate_registration("user@name", Timestamp::from(1000000)).is_err());
        assert!(ParticipantValidator::validate_registration("admin", Timestamp::from(1000000)).is_err());
    }

    #[test]
    fn test_validate_game_variant() {
        // Valid variants
        assert!(GameSessionValidator::validate_game_variant(&GameVariant::Classic2048).is_ok());
        assert!(GameSessionValidator::validate_game_variant(&GameVariant::Speed2048 { time_limit_seconds: 300 }).is_ok());
        assert!(GameSessionValidator::validate_game_variant(&GameVariant::Elimination { lives_remaining: 3 }).is_ok());
        assert!(GameSessionValidator::validate_game_variant(&GameVariant::Collaborative { team_size: 4 }).is_ok());

        // Invalid variants
        assert!(GameSessionValidator::validate_game_variant(&GameVariant::Speed2048 { time_limit_seconds: 0 }).is_err());
        assert!(GameSessionValidator::validate_game_variant(&GameVariant::Speed2048 { time_limit_seconds: 10000 }).is_err());
        assert!(GameSessionValidator::validate_game_variant(&GameVariant::Elimination { lives_remaining: 0 }).is_err());
        assert!(GameSessionValidator::validate_game_variant(&GameVariant::Elimination { lives_remaining: 25 }).is_err());
        assert!(GameSessionValidator::validate_game_variant(&GameVariant::Collaborative { team_size: 1 }).is_err());
        assert!(GameSessionValidator::validate_game_variant(&GameVariant::Collaborative { team_size: 15 }).is_err());
    }

    #[test]
    fn test_validate_competition_creation() {
        let current_time = Timestamp::from(1000000);
        let start_time = Timestamp::from(2000000);
        let end_time = Timestamp::from(5000000);

        // Valid competition
        assert!(CompetitionValidator::validate_competition_creation(
            "Test Tournament",
            start_time,
            end_time,
            current_time
        ).is_ok());

        // Invalid title
        assert!(CompetitionValidator::validate_competition_creation(
            "",
            start_time,
            end_time,
            current_time
        ).is_err());

        // Invalid time range
        assert!(CompetitionValidator::validate_competition_creation(
            "Test Tournament",
            current_time,
            end_time,
            current_time
        ).is_err());

        assert!(CompetitionValidator::validate_competition_creation(
            "Test Tournament",
            end_time,
            start_time,
            current_time
        ).is_err());
    }

    #[test]
    fn test_validate_competition_format() {
        // Valid formats
        assert!(CompetitionValidator::validate_competition_format(
            &CompetitionFormat::SingleElimination {
                bracket_size: 8,
                advancement_criteria: AdvancementCriteria::HighestScore,
            }
        ).is_ok());

        assert!(CompetitionValidator::validate_competition_format(
            &CompetitionFormat::TimeBasedLeaderboard {
                duration: 7200_000_000, // 2 hours
                ranking_criteria: RankingCriteria::Score,
            }
        ).is_ok());

        // Invalid formats
        assert!(CompetitionValidator::validate_competition_format(
            &CompetitionFormat::SingleElimination {
                bracket_size: 3, // Not power of 2
                advancement_criteria: AdvancementCriteria::HighestScore,
            }
        ).is_err());

        assert!(CompetitionValidator::validate_competition_format(
            &CompetitionFormat::TimeBasedLeaderboard {
                duration: 1800_000_000, // 30 minutes (too short)
                ranking_criteria: RankingCriteria::Score,
            }
        ).is_err());
    }
}