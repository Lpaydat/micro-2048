//! Blockchain state management using Linera's view system with improved organization.

use linera_sdk::linera_base_types::{ChainId, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::models::*;
use crate::core::value_objects::*;

/// Main application state with improved naming and organization following DDD principles
/// Simplified for compilation - in a real implementation this would use Linera's view system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlatformState {
    // Participant management (better naming than "players")
    pub participants: HashMap<ParticipantId, Participant>,
    pub username_to_participant: HashMap<String, ParticipantId>,
    pub participant_activity_index: HashMap<ParticipantId, Timestamp>,
    
    // Game session management (more descriptive than "games")
    pub active_game_sessions: HashMap<GameSessionId, GameSession>,
    pub completed_game_sessions: HashMap<GameSessionId, GameSession>,
    
    // Competition management (extensible tournament system)
    pub active_competitions: HashMap<CompetitionId, Competition>,
    pub competition_participants: HashMap<CompetitionId, Vec<ParticipantId>>,
    pub competition_history: HashMap<CompetitionId, CompetitionResults>,
    
    // Cross-chain coordination (improved naming and structure)
    pub outbound_message_queue: Vec<CrossChainMessage>,
    pub inbound_message_log: HashMap<MessageId, ProcessedMessage>,
    pub chain_coordination_registry: HashMap<ChainId, ChainCoordinationInfo>,
    pub leaderboard_synchronization_state: HashMap<CompetitionId, SyncState>,
    
    // System administration (better organization)
    pub platform_configuration: PlatformConfig,
    pub administrative_roles: HashMap<ParticipantId, AdministrativeRole>,
    pub audit_log: Vec<AuditLogEntry>,
    pub system_metrics: SystemMetrics,
    
    // Scalability and performance indexes
    pub competition_by_category: HashMap<CompetitionCategory, Vec<CompetitionId>>,
    pub participants_by_skill_level: HashMap<SkillLevel, Vec<ParticipantId>>,
    pub game_variant_statistics: HashMap<GameVariant, VariantStatistics>,
}

// Supporting types for better state organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionLeaderboard {
    pub last_updated_at: Timestamp,
    pub update_frequency: u64, // Duration in microseconds
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LeaderboardEntry {
    pub participant_id: ParticipantId,
    pub current_rank: u32,
    pub best_score: u64,
    pub total_games: u32,
    pub last_game_time: Timestamp,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChainCoordinationInfo {
    pub chain_role: ChainRole,
    pub last_sync_timestamp: Timestamp,
    pub pending_operations_count: u32,
    pub reliability_score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ChainRole {
    ParticipantHome,      // Hosts participant data
    GameSession,          // Hosts game sessions
    CompetitionHub,       // Coordinates competitions
    LeaderboardAggregator, // Aggregates cross-chain scores
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessedMessage {
    pub message_id: MessageId,
    pub message: CrossChainMessage,
    pub processed_at: Timestamp,
    pub processing_result: MessageProcessingResult,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MessageProcessingResult {
    Success,
    Failed(String),
    Deferred(Timestamp), // Retry at this time
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncState {
    pub last_sync_timestamp: Timestamp,
    pub pending_updates: u32,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SyncStatus {
    InSync,
    Syncing,
    OutOfSync,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SkillLevel {
    Beginner,    // 0-999 rating
    Intermediate, // 1000-1999 rating
    Advanced,    // 2000-2999 rating
    Expert,      // 3000+ rating
}

impl From<Option<u32>> for SkillLevel {
    fn from(rating: Option<u32>) -> Self {
        match rating {
            Some(r) if r >= 3000 => SkillLevel::Expert,
            Some(r) if r >= 2000 => SkillLevel::Advanced,
            Some(r) if r >= 1000 => SkillLevel::Intermediate,
            _ => SkillLevel::Beginner,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariantStatistics {
    pub total_sessions: u32,
    pub average_score: f64,
    pub highest_score: u64,
    pub average_duration: u64, // in microseconds
    pub completion_rate: f64,
}

impl Default for VariantStatistics {
    fn default() -> Self {
        Self {
            total_sessions: 0,
            average_score: 0.0,
            highest_score: 0,
            average_duration: 0,
            completion_rate: 0.0,
        }
    }
}

impl GamePlatformState {
    /// Initialize the platform with default configuration
    pub async fn initialize_platform(&mut self) -> Result<(), StateError> {
        // Set default platform configuration
        self.platform_configuration = PlatformConfig::default();

        // Initialize system metrics
        self.system_metrics = SystemMetrics::default();

        Ok(())
    }

    /// Get participant by ID
    pub async fn get_participant(&self, participant_id: &ParticipantId) -> Result<Option<Participant>, StateError> {
        Ok(self.participants.get(participant_id).cloned())
    }

    /// Get participant by username
    pub async fn get_participant_by_username(&self, username: &str) -> Result<Option<Participant>, StateError> {
        if let Some(participant_id) = self.username_to_participant.get(username) {
            self.get_participant(participant_id).await
        } else {
            Ok(None)
        }
    }

    /// Register a new participant
    pub async fn register_participant(&mut self, participant: Participant) -> Result<(), StateError> {
        let participant_id = participant.participant_id;
        let username = participant.display_identity.username.clone();

        // Check if username is already taken
        if self.username_to_participant.contains_key(&username) {
            return Err(StateError::UsernameAlreadyExists);
        }

        // Store participant
        self.participants.insert(participant_id, participant);

        // Index by username
        self.username_to_participant.insert(username, participant_id);

        // Update activity index
        let current_time = Timestamp::from(0); // This will be set by the contract
        self.participant_activity_index.insert(participant_id, current_time);

        Ok(())
    }

    /// Create a new game session
    pub async fn create_game_session(&mut self, session: GameSession) -> Result<(), StateError> {
        let session_id = session.session_id;

        // Store the session
        self.active_game_sessions.insert(session_id, session);

        Ok(())
    }

    /// Complete a game session
    pub async fn complete_game_session(&mut self, session_id: &GameSessionId) -> Result<(), StateError> {
        // Get session first
        if let Some(session) = self.active_game_sessions.get(session_id).cloned() {
            // Store in completed sessions
            self.completed_game_sessions.insert(*session_id, session);

            // Remove from active sessions
            self.active_game_sessions.remove(session_id);

            Ok(())
        } else {
            Err(StateError::SessionNotFound)
        }
    }

    /// Create a new competition
    pub async fn create_competition(&mut self, competition: Competition) -> Result<(), StateError> {
        let competition_id = competition.competition_id;

        // Store competition
        self.active_competitions.insert(competition_id, competition);

        Ok(())
    }

    /// Add participant to competition
    pub async fn add_participant_to_competition(
        &mut self,
        competition_id: &CompetitionId,
        participant_id: &ParticipantId,
    ) -> Result<(), StateError> {
        self.competition_participants
            .entry(*competition_id)
            .or_insert_with(Vec::new)
            .push(*participant_id);
        Ok(())
    }

    /// Get all participants in a competition
    pub async fn get_competition_participants(&self, competition_id: &CompetitionId) -> Result<Vec<ParticipantId>, StateError> {
        Ok(self.competition_participants
            .get(competition_id)
            .cloned()
            .unwrap_or_default())
    }

    /// Update system metrics
    pub async fn update_system_metrics(&mut self, current_time: Timestamp) -> Result<(), StateError> {
        let metrics = SystemMetrics {
            active_sessions_count: self.active_game_sessions.len() as u32,
            total_participants: self.participants.len() as u32,
            active_competitions: self.active_competitions.len() as u32,
            cross_chain_messages_pending: self.outbound_message_queue.len() as u32,
            last_updated: current_time,
        };

        self.system_metrics = metrics;
        Ok(())
    }
}

/// Errors that can occur during state operations
#[derive(Debug, Clone)]
pub enum StateError {
    ViewError(String),
    ParticipantNotFound,
    SessionNotFound,
    CompetitionNotFound,
    UsernameAlreadyExists,
    InvalidOperation(String),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::ViewError(msg) => write!(f, "View error: {}", msg),
            StateError::ParticipantNotFound => write!(f, "Participant not found"),
            StateError::SessionNotFound => write!(f, "Game session not found"),
            StateError::CompetitionNotFound => write!(f, "Competition not found"),
            StateError::UsernameAlreadyExists => write!(f, "Username already exists"),
            StateError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

impl std::error::Error for StateError {}

impl Default for GamePlatformState {
    fn default() -> Self {
        Self {
            participants: HashMap::new(),
            username_to_participant: HashMap::new(),
            participant_activity_index: HashMap::new(),
            active_game_sessions: HashMap::new(),
            completed_game_sessions: HashMap::new(),
            active_competitions: HashMap::new(),
            competition_participants: HashMap::new(),
            competition_history: HashMap::new(),
            outbound_message_queue: Vec::new(),
            inbound_message_log: HashMap::new(),
            chain_coordination_registry: HashMap::new(),
            leaderboard_synchronization_state: HashMap::new(),
            platform_configuration: PlatformConfig::default(),
            administrative_roles: HashMap::new(),
            audit_log: Vec::new(),
            system_metrics: SystemMetrics::default(),
            competition_by_category: HashMap::new(),
            participants_by_skill_level: HashMap::new(),
            game_variant_statistics: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linera_sdk::linera_base_types::{ChainId, Timestamp};

    // Note: These tests would require a proper test setup with ViewStorageContext
    // For now, we'll include basic structure tests

    #[test]
    fn test_skill_level_conversion() {
        assert_eq!(SkillLevel::from(None), SkillLevel::Beginner);
        assert_eq!(SkillLevel::from(Some(500)), SkillLevel::Beginner);
        assert_eq!(SkillLevel::from(Some(1500)), SkillLevel::Intermediate);
        assert_eq!(SkillLevel::from(Some(2500)), SkillLevel::Advanced);
        assert_eq!(SkillLevel::from(Some(3500)), SkillLevel::Expert);
    }

    #[test]
    fn test_variant_statistics_default() {
        let stats = VariantStatistics::default();
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.average_score, 0.0);
        assert_eq!(stats.highest_score, 0);
        assert_eq!(stats.average_duration, 0);
        assert_eq!(stats.completion_rate, 0.0);
    }

    #[test]
    fn test_chain_role_enum() {
        let role = ChainRole::ParticipantHome;
        assert_eq!(role, ChainRole::ParticipantHome);
        assert_ne!(role, ChainRole::GameSession);
    }

    #[test]
    fn test_sync_status_enum() {
        let status = SyncStatus::InSync;
        assert_eq!(status, SyncStatus::InSync);
        assert_ne!(status, SyncStatus::OutOfSync);
    }
}