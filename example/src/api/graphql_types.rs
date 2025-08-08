// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use async_graphql::{Enum, SimpleObject, InputObject};
use serde::{Deserialize, Serialize};

use crate::core::types::{
    GameStatus as InternalGameStatus, 
    PlayerStatus as InternalPlayerStatus,
    EventStatus as InternalEventStatus,
};

/// GraphQL-compatible player status
#[derive(Debug, Serialize, Deserialize, Clone, Enum, Copy, PartialEq, Eq)]
pub enum PlayerStatusType {
    Active,
    Suspended,
    Banned,
}

impl From<&InternalPlayerStatus> for PlayerStatusType {
    fn from(status: &InternalPlayerStatus) -> Self {
        match status {
            InternalPlayerStatus::Active => PlayerStatusType::Active,
            InternalPlayerStatus::Suspended { .. } => PlayerStatusType::Suspended,
            InternalPlayerStatus::Banned { .. } => PlayerStatusType::Banned,
        }
    }
}

/// GraphQL-compatible game status
#[derive(Debug, Serialize, Deserialize, Clone, Enum, Copy, PartialEq, Eq)]
pub enum GameStatusType {
    Pending,
    Active,
    Suspended,
    Deprecated,
}

impl From<&InternalGameStatus> for GameStatusType {
    fn from(status: &InternalGameStatus) -> Self {
        match status {
            InternalGameStatus::Pending => GameStatusType::Pending,
            InternalGameStatus::Active => GameStatusType::Active,
            InternalGameStatus::Suspended { .. } => GameStatusType::Suspended,
            InternalGameStatus::Deprecated => GameStatusType::Deprecated,
        }
    }
}

/// Enhanced player for GraphQL queries
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct PlayerObject {
    pub discord_id: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub total_points: u64,
    pub participation_streak: u32,
    pub current_rank: Option<u32>,
    pub status: PlayerStatusType,
    pub created_at: String, // Timestamp as string for GraphQL
    pub last_active: String, // Timestamp as string for GraphQL
}

/// Input for player registration from PRD specification
#[derive(InputObject, Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPlayerInput {
    pub discord_id: String,
    pub username: String,
    pub avatar_url: Option<String>,
}

/// Enhanced game for GraphQL queries
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct GameObject {
    pub id: String,  
    pub name: String,
    pub description: String,
    pub contract_address: String,
    pub developer_name: String,
    pub developer_contact: String,
    pub status: GameStatusType,
    pub approved_by: Option<String>,
    pub created_at: String, // Timestamp as string for GraphQL
    pub approved_at: Option<String>, // Timestamp as string for GraphQL
}

/// Pending game for GraphQL queries
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct PendingGameObject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub contract_address: String,
    pub developer_name: String,
    pub developer_contact: String,
    pub created_at: String, // Timestamp as string for GraphQL
}

/// Mutation response for GraphQL
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct MutationResponse {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

/// GraphQL-compatible event status
#[derive(Debug, Serialize, Deserialize, Clone, Enum, Copy, PartialEq, Eq)]
pub enum EventStatusType {
    Upcoming,
    Active,
    Ended,
    Cancelled,
}

impl From<&InternalEventStatus> for EventStatusType {
    fn from(status: &InternalEventStatus) -> Self {
        match status {
            InternalEventStatus::Upcoming => EventStatusType::Upcoming,
            InternalEventStatus::Active => EventStatusType::Active,
            InternalEventStatus::Ended => EventStatusType::Ended,
            InternalEventStatus::Cancelled => EventStatusType::Cancelled,
        }
    }
}

/// Event for GraphQL queries
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct EventObject {
    pub id: String,
    pub game_id: String,
    pub name: String,
    pub description: String,
    pub start_time: String, // Timestamp as string
    pub end_time: Option<String>, // Timestamp as string
    pub status: EventStatusType,
    pub max_participants: Option<u32>,
    pub prize_pool: Option<u64>,
    pub is_mandatory_for_streak: bool, // Required for streak system
}

/// Leaderboard entry for GraphQL queries
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntryObject {
    pub player_discord_id: String,
    pub player_username: String,
    pub score: u64,
    pub rank: u32,
    pub points_earned: u64,
    pub completion_time: Option<String>, // Timestamp as string
}

/// Scoring configuration for GraphQL queries
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfigObject {
    pub base_points_per_event: u64,
    pub streak_bonus_multiplier: u64, // Scaled by 100 (150 = 1.5x)
    pub bronze_booster_threshold: u32,
    pub silver_booster_threshold: u32,
    pub gold_booster_threshold: u32,
    pub bronze_multiplier: u64, // Scaled by 100 (120 = 1.2x)
    pub silver_multiplier: u64, // Scaled by 100 (150 = 1.5x)
    pub gold_multiplier: u64, // Scaled by 100 (200 = 2.0x)
    pub streak_grace_period_hours: u32,
    pub max_streak_multiplier: u64, // Scaled by 100 (300 = 3.0x)
}

/// Audit log entry for GraphQL queries
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntryObject {
    pub id: String,
    pub action_type: String, // Stringified AdminAction for GraphQL compatibility
    pub performed_by: String,
    pub target: Option<String>,
    pub timestamp: String, // Timestamp as string
    pub details: Option<String>,
}

/// GameHub event for GraphQL queries
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct GameHubEventObject {
    pub id: String,
    pub event_type: String, // Stringified EventType
    pub description: String,
    pub actor_id: Option<String>,
    pub target_id: Option<String>,
    pub timestamp: String, // Timestamp as string
    pub metadata: Option<String>, // JSON string
}

/// Player statistics for GraphQL queries
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatsObject {
    pub discord_id: String,
    pub username: String,
    pub total_points: u64,
    pub participation_streak: u32,
    pub events_participated: u32,
    pub events_won: u32,
    pub current_rank: Option<u32>,
    pub average_score: u64,
    pub best_streak: u32,
    pub total_boosters_earned: u32,
}

/// Pending player data for GraphQL queries
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct PendingPlayerDataObject {
    pub discord_id: String,
    pub username: String,
    pub total_pending_points: u64,
    pub events_participated: u32,
    pub last_participation: String, // Timestamp as string
    pub current_streak: u32,
}

/// Participation data for GraphQL queries
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationDataObject {
    pub event_id: String,
    pub score: u64,
    pub rank: u32,
    pub points_earned: u64,
    pub timestamp: String, // Timestamp as string
    pub booster_applied: Option<String>, // Booster type as string
}

/// Enhanced participation data for main leaderboard
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedParticipationDataObject {
    pub streak_level: String,
    pub streak_multiplier: u32,
    pub total_points_earned: u64,
    pub events_participated: u32,
    pub last_event_timestamp: Option<String>,
    pub booster_history: Vec<String>,
}

/// Main leaderboard entry with enhanced data
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct MainLeaderboardEntryObject {
    pub player: PlayerObject,
    pub participation_data: EnhancedParticipationDataObject,
}

/// Player game history entry
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct GameHistoryEntryObject {
    pub game_id: String,
    pub game_name: String,
    pub events_participated: u32,
    pub total_score: u64,
    pub best_score: u64,
    pub first_participation: String, // Timestamp as string
    pub last_participation: String,  // Timestamp as string
    pub current_streak: u32,
}

/// Player event participation entry
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct EventParticipationObject {
    pub event_id: String,
    pub event_name: String,
    pub game_id: String,
    pub game_name: String,
    pub score: u64,
    pub rank: Option<u32>,
    pub points_earned: u64,
    pub booster_applied: Option<String>,
    pub participation_timestamp: String, // Timestamp as string
    pub streak_eligible: bool,
}

/// Player streak history entry
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct StreakHistoryEntryObject {
    pub streak_count: u32,
    pub start_date: String,       // Timestamp as string
    pub end_date: Option<String>, // Timestamp as string (None if current streak)
    pub events_included: Vec<String>, // Event IDs
    pub booster_level: Option<String>,
    pub total_bonus_points: u64,
}

/// System health metrics for admin dashboard
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthObject {
    pub total_players: u32,
    pub total_games: u32,
    pub pending_games: u32,
    pub total_events: u32,
    pub total_audit_entries: u32,
    pub active_players: u32,
    pub recent_registrations: u32,
}

/// Date range input for analytics queries
#[derive(InputObject, Debug, Clone, Serialize, Deserialize)]
pub struct DateRangeInput {
    pub start_date: String, // ISO 8601 date string
    pub end_date: String,   // ISO 8601 date string
}

/// Player engagement metrics for analytics
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct PlayerEngagementObject {
    pub date: String,           // ISO 8601 date string
    pub active_users: u32,      // Users who participated in events this day
    pub new_registrations: u32, // New player registrations this day
    pub total_events: u32,      // Events held this day
    pub total_participation: u32, // Total participations across all events
}

/// Analytics data for admin dashboard with date range filtering
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsObject {
    pub total_players: u32,
    pub active_games: u32,
    pub total_events: u32,
    pub active_players_in_period: u32, // Players active within the date range
    pub new_registrations_in_period: u32, // New registrations within date range
    pub player_engagement: Vec<PlayerEngagementObject>, // Daily breakdown
}

/// Game popularity and statistics for admin management
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct GameStatsObject {
    pub game_id: String,
    pub game_name: String,
    pub total_events: u32,
    pub total_participants: u32,
    pub unique_players: u32,
    pub average_participants_per_event: f64,
    pub last_event_date: Option<String>, // ISO 8601 date string
    pub popularity_score: f64, // Calculated popularity metric
}

/// Import result for CSV operations
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct ImportResultObject {
    pub success: bool,
    pub total_records: u32,
    pub successful_imports: u32,
    pub failed_imports: u32,
    pub errors: Vec<String>,
    pub summary_message: String,
}
