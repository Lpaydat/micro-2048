//! Participant domain model with better naming and extensibility.

use serde::{Deserialize, Serialize};
use linera_sdk::linera_base_types::{ChainId, Timestamp};
use crate::core::value_objects::*;

/// Represents a participant in the gaming platform with improved naming conventions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub participant_id: ParticipantId,
    pub display_identity: DisplayIdentity,
    pub blockchain_identity: BlockchainIdentity,
    pub participation_history: ParticipationHistory,
    pub skill_metrics: SkillMetrics,
    pub account_status: AccountStatus,
}

impl Participant {
    /// Create a new participant with the given username and home chain
    pub fn new(
        participant_id: ParticipantId,
        username: String,
        home_chain_id: ChainId,
        current_time: Timestamp,
    ) -> Result<Self, ParticipantError> {
        // Validate username
        Self::validate_username(&username)?;

        Ok(Self {
            participant_id,
            display_identity: DisplayIdentity {
                username,
                display_name: None,
                avatar_hash: None,
            },
            blockchain_identity: BlockchainIdentity {
                home_chain_id,
                wallet_address: None,
                verification_status: VerificationStatus::Unverified,
            },
            participation_history: ParticipationHistory {
                account_created_at: current_time,
                last_activity_at: current_time,
                total_sessions_played: 0,
                competitions_entered: 0,
                achievements_earned: Vec::new(),
            },
            skill_metrics: SkillMetrics::default(),
            account_status: AccountStatus::Active,
        })
    }

    /// Validate username according to platform rules
    fn validate_username(username: &str) -> Result<(), ParticipantError> {
        if username.is_empty() {
            return Err(ParticipantError::InvalidUsername("Username cannot be empty".to_string()));
        }
        
        if username.len() < 3 {
            return Err(ParticipantError::InvalidUsername("Username must be at least 3 characters".to_string()));
        }
        
        if username.len() > 20 {
            return Err(ParticipantError::InvalidUsername("Username cannot exceed 20 characters".to_string()));
        }
        
        // Check for valid characters (alphanumeric and underscore only)
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ParticipantError::InvalidUsername("Username can only contain letters, numbers, and underscores".to_string()));
        }
        
        Ok(())
    }

    /// Check if the participant is active and can participate in games
    pub fn is_active(&self) -> bool {
        matches!(self.account_status, AccountStatus::Active)
    }

    /// Check if the participant is banned
    pub fn is_banned(&self) -> bool {
        matches!(self.account_status, AccountStatus::Banned { .. })
    }

    /// Check if the participant is suspended and if the suspension has expired
    pub fn is_suspended(&self, current_time: Timestamp) -> bool {
        match self.account_status {
            AccountStatus::Suspended { until } => current_time < until,
            _ => false,
        }
    }

    /// Update the participant's last activity timestamp
    pub fn update_activity(&mut self, current_time: Timestamp) {
        self.participation_history.last_activity_at = current_time;
    }

    /// Record a completed game session
    pub fn record_game_session(&mut self, final_score: u64, current_time: Timestamp) {
        self.participation_history.total_sessions_played += 1;
        self.update_activity(current_time);
        
        // Update skill metrics
        if final_score > self.skill_metrics.personal_best_score {
            self.skill_metrics.personal_best_score = final_score;
        }
        
        // Update average score (simple moving average)
        let total_sessions = self.participation_history.total_sessions_played as f64;
        self.skill_metrics.average_score = 
            (self.skill_metrics.average_score * (total_sessions - 1.0) + final_score as f64) / total_sessions;
    }

    /// Record participation in a competition
    pub fn record_competition_entry(&mut self, competition_id: CompetitionId, current_time: Timestamp) {
        self.participation_history.competitions_entered += 1;
        self.update_activity(current_time);
    }

    /// Add an achievement to the participant's record
    pub fn unlock_achievement(&mut self, achievement: Achievement) {
        // Check if achievement is already unlocked
        if !self.participation_history.achievements_earned
            .iter()
            .any(|a| a.achievement_id == achievement.achievement_id) {
            self.participation_history.achievements_earned.push(achievement);
        }
    }

    /// Ban the participant with the given reason
    pub fn ban(&mut self, reason: BanReason) {
        self.account_status = AccountStatus::Banned { reason };
    }

    /// Suspend the participant until the given timestamp
    pub fn suspend(&mut self, until: Timestamp) {
        self.account_status = AccountStatus::Suspended { until };
    }

    /// Reactivate a banned or suspended participant
    pub fn reactivate(&mut self) {
        self.account_status = AccountStatus::Active;
    }

    /// Update the participant's display name
    pub fn update_display_name(&mut self, display_name: Option<String>) -> Result<(), ParticipantError> {
        if let Some(ref name) = display_name {
            if name.len() > 50 {
                return Err(ParticipantError::InvalidDisplayName("Display name cannot exceed 50 characters".to_string()));
            }
        }
        self.display_identity.display_name = display_name;
        Ok(())
    }

    /// Set the participant's avatar hash
    pub fn set_avatar_hash(&mut self, avatar_hash: Option<String>) {
        self.display_identity.avatar_hash = avatar_hash;
    }

    /// Get the participant's effective display name (display_name or username)
    pub fn get_effective_display_name(&self) -> &str {
        self.display_identity.display_name
            .as_ref()
            .unwrap_or(&self.display_identity.username)
    }
}

/// Errors that can occur when working with participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantError {
    InvalidUsername(String),
    InvalidDisplayName(String),
    ParticipantNotFound,
    ParticipantAlreadyExists,
    ParticipantBanned(BanReason),
    ParticipantSuspended(Timestamp),
    InsufficientPermissions,
}

impl std::fmt::Display for ParticipantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParticipantError::InvalidUsername(msg) => write!(f, "Invalid username: {}", msg),
            ParticipantError::InvalidDisplayName(msg) => write!(f, "Invalid display name: {}", msg),
            ParticipantError::ParticipantNotFound => write!(f, "Participant not found"),
            ParticipantError::ParticipantAlreadyExists => write!(f, "Participant already exists"),
            ParticipantError::ParticipantBanned(reason) => write!(f, "Participant is banned: {:?}", reason),
            ParticipantError::ParticipantSuspended(until) => write!(f, "Participant is suspended until: {:?}", until),
            ParticipantError::InsufficientPermissions => write!(f, "Insufficient permissions"),
        }
    }
}

impl std::error::Error for ParticipantError {}

#[cfg(test)]
mod tests {
    use super::*;
    use linera_sdk::linera_base_types::{ChainId, Timestamp};

    #[test]
    fn test_participant_creation() {
        let participant_id = ParticipantId(1);
        let username = "test_user".to_string();
        let home_chain = ChainId::root(0);
        let current_time = Timestamp::from(1000000);

        let participant = Participant::new(participant_id, username.clone(), home_chain, current_time)
            .expect("Participant creation should succeed");

        assert_eq!(participant.participant_id, participant_id);
        assert_eq!(participant.display_identity.username, username);
        assert_eq!(participant.blockchain_identity.home_chain_id, home_chain);
        assert_eq!(participant.participation_history.account_created_at, current_time);
        assert!(participant.is_active());
        assert!(!participant.is_banned());
        assert!(!participant.is_suspended(current_time));
    }

    #[test]
    fn test_username_validation() {
        let participant_id = ParticipantId(1);
        let home_chain = ChainId::root(0);
        let current_time = Timestamp::from(1000000);

        // Test empty username
        let result = Participant::new(participant_id, "".to_string(), home_chain, current_time);
        assert!(matches!(result, Err(ParticipantError::InvalidUsername(_))));

        // Test short username
        let result = Participant::new(participant_id, "ab".to_string(), home_chain, current_time);
        assert!(matches!(result, Err(ParticipantError::InvalidUsername(_))));

        // Test long username
        let long_username = "a".repeat(21);
        let result = Participant::new(participant_id, long_username, home_chain, current_time);
        assert!(matches!(result, Err(ParticipantError::InvalidUsername(_))));

        // Test invalid characters
        let result = Participant::new(participant_id, "user@name".to_string(), home_chain, current_time);
        assert!(matches!(result, Err(ParticipantError::InvalidUsername(_))));

        // Test valid username
        let result = Participant::new(participant_id, "valid_user123".to_string(), home_chain, current_time);
        assert!(result.is_ok());
    }

    #[test]
    fn test_game_session_recording() {
        let participant_id = ParticipantId(1);
        let username = "test_user".to_string();
        let home_chain = ChainId::root(0);
        let current_time = Timestamp::from(1000000);

        let mut participant = Participant::new(participant_id, username, home_chain, current_time)
            .expect("Participant creation should succeed");

        assert_eq!(participant.participation_history.total_sessions_played, 0);
        assert_eq!(participant.skill_metrics.personal_best_score, 0);
        assert_eq!(participant.skill_metrics.average_score, 0.0);

        // Record first game
        participant.record_game_session(1000, current_time);
        assert_eq!(participant.participation_history.total_sessions_played, 1);
        assert_eq!(participant.skill_metrics.personal_best_score, 1000);
        assert_eq!(participant.skill_metrics.average_score, 1000.0);

        // Record second game with higher score
        participant.record_game_session(1500, current_time);
        assert_eq!(participant.participation_history.total_sessions_played, 2);
        assert_eq!(participant.skill_metrics.personal_best_score, 1500);
        assert_eq!(participant.skill_metrics.average_score, 1250.0);

        // Record third game with lower score
        participant.record_game_session(800, current_time);
        assert_eq!(participant.participation_history.total_sessions_played, 3);
        assert_eq!(participant.skill_metrics.personal_best_score, 1500); // Should not change
        assert!((participant.skill_metrics.average_score - 1100.0).abs() < 0.01); // (1000 + 1500 + 800) / 3
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
        assert!(!participant.is_suspended(current_time));

        // Suspend participant
        let suspension_end = Timestamp::from(2000000);
        participant.suspend(suspension_end);
        
        assert!(!participant.is_active());
        assert!(participant.is_suspended(current_time));
        assert!(participant.is_suspended(Timestamp::from(1500000))); // Still suspended
        assert!(!participant.is_suspended(Timestamp::from(2500000))); // Suspension expired

        // Reactivate participant
        participant.reactivate();
        assert!(participant.is_active());
        assert!(!participant.is_suspended(current_time));
    }

    #[test]
    fn test_participant_ban() {
        let participant_id = ParticipantId(1);
        let username = "test_user".to_string();
        let home_chain = ChainId::root(0);
        let current_time = Timestamp::from(1000000);

        let mut participant = Participant::new(participant_id, username, home_chain, current_time)
            .expect("Participant creation should succeed");

        assert!(participant.is_active());
        assert!(!participant.is_banned());

        // Ban participant
        participant.ban(BanReason::Cheating);
        
        assert!(!participant.is_active());
        assert!(participant.is_banned());

        // Reactivate participant
        participant.reactivate();
        assert!(participant.is_active());
        assert!(!participant.is_banned());
    }

    #[test]
    fn test_achievement_unlocking() {
        let participant_id = ParticipantId(1);
        let username = "test_user".to_string();
        let home_chain = ChainId::root(0);
        let current_time = Timestamp::from(1000000);

        let mut participant = Participant::new(participant_id, username, home_chain, current_time)
            .expect("Participant creation should succeed");

        assert_eq!(participant.participation_history.achievements_earned.len(), 0);

        let achievement = Achievement {
            achievement_id: "first_game".to_string(),
            name: "First Game".to_string(),
            description: "Played your first game".to_string(),
            unlocked_at: current_time,
        };

        participant.unlock_achievement(achievement.clone());
        assert_eq!(participant.participation_history.achievements_earned.len(), 1);

        // Try to unlock the same achievement again
        participant.unlock_achievement(achievement);
        assert_eq!(participant.participation_history.achievements_earned.len(), 1); // Should not duplicate
    }
}