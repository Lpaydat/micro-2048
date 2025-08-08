//! Domain value objects and enums for type safety.
//! These provide strong typing and encapsulation of domain concepts.

use serde::{Deserialize, Serialize};
use linera_sdk::linera_base_types::{ChainId, Timestamp};

// Unique identifiers for domain entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameSessionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParticipantId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CompetitionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub u64);

// Game-related value objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoardState {
    /// Bit-packed 4x4 grid representation for efficient storage
    pub tiles: u64,
    /// Number of moves made in this game session
    pub move_count: u32,
    /// Highest tile value achieved (e.g., 2048, 4096)
    pub highest_tile_achieved: u32,
}

impl BoardState {
    pub fn new() -> Self {
        Self {
            tiles: 0,
            move_count: 0,
            highest_tile_achieved: 2,
        }
    }
}

impl Default for BoardState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    InProgress,
    CompletedSuccessfully,
    CompletedWithFailure,
    AbandonedByPlayer,
    TerminatedBySystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLifecycle {
    pub initiated_at: Timestamp,
    pub last_activity_at: Timestamp,
    pub concluded_at: Option<Timestamp>,
    pub duration_limit: Option<u64>, // Duration in microseconds
}

// Extensible game variants for future game modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameVariant {
    Classic2048,
    Speed2048 { time_limit_seconds: u32 },
    Elimination { lives_remaining: u8 },
    Collaborative { team_size: u8 },
}

impl Default for GameVariant {
    fn default() -> Self {
        GameVariant::Classic2048
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringMetrics {
    pub primary_score: u64,
    pub bonus_multipliers: Vec<BonusMultiplier>,
    pub achievement_unlocks: Vec<Achievement>,
    pub performance_stats: PerformanceStats,
}

impl Default for ScoringMetrics {
    fn default() -> Self {
        Self {
            primary_score: 0,
            bonus_multipliers: Vec::new(),
            achievement_unlocks: Vec::new(),
            performance_stats: PerformanceStats::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusMultiplier {
    pub multiplier_type: MultiplierType,
    pub value: f64,
    pub applied_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MultiplierType {
    SpeedBonus,
    ComboBonus,
    TournamentBonus,
    AchievementBonus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub achievement_id: String,
    pub name: String,
    pub description: String,
    pub unlocked_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub average_move_time: f64, // in seconds
    pub total_play_time: u64,   // in microseconds
    pub efficiency_rating: f64,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            average_move_time: 0.0,
            total_play_time: 0,
            efficiency_rating: 0.0,
        }
    }
}

// Participant-related value objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayIdentity {
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainIdentity {
    pub home_chain_id: ChainId,
    pub wallet_address: Option<String>, // For future wallet integration
    pub verification_status: VerificationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Unverified,
    Pending,
    Verified,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationHistory {
    pub account_created_at: Timestamp,
    pub last_activity_at: Timestamp,
    pub total_sessions_played: u32,
    pub competitions_entered: u32,
    pub achievements_earned: Vec<Achievement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetrics {
    pub personal_best_score: u64,
    pub average_score: f64,
    pub skill_rating: Option<u32>, // ELO-style rating for future matchmaking
    pub preferred_game_variants: Vec<GameVariant>,
}

impl Default for SkillMetrics {
    fn default() -> Self {
        Self {
            personal_best_score: 0,
            average_score: 0.0,
            skill_rating: None,
            preferred_game_variants: vec![GameVariant::Classic2048],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AccountStatus {
    Active,
    Suspended { until: Timestamp },
    Banned { reason: BanReason },
    Inactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BanReason {
    Cheating,
    Harassment,
    Spam,
    TermsViolation,
    Other,
}

// Competition-related value objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionMetadata {
    pub title: String,
    pub description: Option<String>,
    pub organizer_id: ParticipantId,
    pub category: CompetitionCategory,
    pub visibility: CompetitionVisibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompetitionCategory {
    Casual,
    Competitive,
    Professional,
    Educational,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CompetitionVisibility {
    Public,
    Private,
    InviteOnly,
}

// Extensible competition formats for different tournament types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompetitionFormat {
    SingleElimination {
        bracket_size: u32,
        advancement_criteria: AdvancementCriteria,
    },
    RoundRobin {
        rounds_count: u32,
        scoring_system: ScoringSystem,
    },
    TimeBasedLeaderboard {
        duration: u64, // Duration in microseconds
        ranking_criteria: RankingCriteria,
    },
    EliminationSurvival {
        elimination_threshold: u64,
        elimination_interval: u64, // Duration in microseconds
    },
    TeamBased {
        team_size: u8,
        team_formation: TeamFormation,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AdvancementCriteria {
    HighestScore,
    FastestTime,
    MostEfficient,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ScoringSystem {
    Standard,
    Weighted,
    Cumulative,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RankingCriteria {
    Score,
    Time,
    Efficiency,
    Combined,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TeamFormation {
    Random,
    SelfSelected,
    SkillBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationRules {
    pub max_participants: Option<u32>,
    pub min_skill_rating: Option<u32>,
    pub entry_requirements: Vec<EntryRequirement>,
    pub game_variant_restrictions: Vec<GameVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryRequirement {
    MinimumGamesPlayed(u32),
    MinimumScore(u64),
    AchievementRequired(String),
    InvitationRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionLifecycle {
    pub registration_opens_at: Timestamp,
    pub registration_closes_at: Timestamp,
    pub competition_starts_at: Timestamp,
    pub competition_ends_at: Timestamp,
    pub results_finalized_at: Option<Timestamp>,
    pub current_phase: CompetitionPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CompetitionPhase {
    RegistrationOpen,
    RegistrationClosed,
    InProgress,
    Concluded,
    ResultsFinalized,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardStructure {
    pub prize_pool: u64,
    pub distribution: Vec<RewardTier>,
    pub reward_type: RewardType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardTier {
    pub position_range: (u32, u32), // (start, end) positions
    pub reward_amount: u64,
    pub reward_percentage: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RewardType {
    Points,
    Tokens,
    Achievements,
    Titles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainCoordination {
    pub coordinator_chain: ChainId,
    pub participant_chains: Vec<ChainId>,
    pub leaderboard_chain: ChainId,
    pub sync_frequency: u64, // Duration in microseconds
}

// Cross-chain message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossChainMessage {
    ParticipantRegistered {
        participant_id: ParticipantId,
        home_chain: ChainId,
    },
    GameSessionStarted {
        session_id: GameSessionId,
        participant_id: ParticipantId,
        competition_id: Option<CompetitionId>,
    },
    GameSessionCompleted {
        session_id: GameSessionId,
        participant_id: ParticipantId,
        final_score: u64,
        competition_id: Option<CompetitionId>,
    },
    CompetitionCreated {
        competition_id: CompetitionId,
        metadata: CompetitionMetadata,
    },
    LeaderboardUpdate {
        competition_id: CompetitionId,
        participant_scores: Vec<(ParticipantId, u64)>,
    },
}

// Administrative types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AdministrativeRole {
    SuperAdmin,
    CompetitionOrganizer,
    Moderator,
    Observer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub entry_id: u64,
    pub timestamp: Timestamp,
    pub actor_id: ParticipantId,
    pub action: AdministrativeAction,
    pub target: Option<String>,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdministrativeAction {
    ParticipantBanned,
    ParticipantUnbanned,
    CompetitionCreated,
    CompetitionCancelled,
    SystemConfigurationChanged,
    DataMigrationPerformed,
}

// System configuration and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub max_concurrent_sessions: u32,
    pub session_timeout: u64, // Duration in microseconds
    pub max_competition_participants: u32,
    pub cross_chain_sync_interval: u64, // Duration in microseconds
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            max_concurrent_sessions: 10000,
            session_timeout: 3600_000_000, // 1 hour in microseconds
            max_competition_participants: 1000,
            cross_chain_sync_interval: 60_000_000, // 1 minute in microseconds
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub active_sessions_count: u32,
    pub total_participants: u32,
    pub active_competitions: u32,
    pub cross_chain_messages_pending: u32,
    pub last_updated: Timestamp,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            active_sessions_count: 0,
            total_participants: 0,
            active_competitions: 0,
            cross_chain_messages_pending: 0,
            last_updated: Timestamp::from(0),
        }
    }
}