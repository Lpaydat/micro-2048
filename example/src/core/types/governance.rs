use serde::{Deserialize, Serialize};
use linera_sdk::linera_base_types::Timestamp;
use std::collections::HashMap;
use crate::core::types::points::{ParticipationPointsConfig, StreakBonusType, WinnerBonusConfig};
use crate::infrastructure::errors::GameHubError;

/// DAO governance proposal for community voting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    /// Unique proposal identifier
    pub id: String,
    /// Type of proposal and its specific data
    pub proposal_type: ProposalType,
    /// Human-readable proposal title
    pub title: String,
    /// Detailed description of the proposal
    pub description: String,
    /// Discord ID of the proposer
    pub proposed_by: String,
    /// When the proposal was created
    pub created_at: Timestamp,
    /// When voting ends
    pub voting_ends_at: Timestamp,
    /// Minimum total votes required for proposal to be valid
    pub minimum_votes_required: u32,
    /// Minimum percentage of eligible voters that must participate (quorum)
    pub quorum_threshold: u32,
    /// Total voting power cast for the proposal
    pub votes_for: u32,
    /// Total voting power cast against the proposal
    pub votes_against: u32,
    /// All votes cast (Discord ID -> Vote)
    pub voters: HashMap<String, Vote>,
    /// Current status of the proposal
    pub status: ProposalStatus,
    /// Impact analysis for the proposal (if applicable)
    pub impact_analysis: Option<ProposalImpactAnalysis>,
    /// Safety review results
    pub safety_review: ProposalSafetyReview,
}

/// Types of proposals that can be submitted to DAO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    /// Change the entire points configuration
    PointsConfigChange { 
        proposed_config: ParticipationPointsConfig 
    },
    /// Modify just the streak bonus calculation
    StreakBonusModification { 
        new_bonus_type: StreakBonusType 
    },
    /// Introduce or modify winner bonuses
    WinnerBonusIntroduction { 
        winner_config: WinnerBonusConfig 
    },
    /// Set special event rules and multipliers
    SpecialEventRules { 
        event_type: String, 
        multiplier: f64 
    },
    /// Change system parameters
    SystemParameterChange {
        parameter: String,
        new_value: String,
        rationale: String,
    },
}

/// Current status of a governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Proposal is in draft state
    Draft,
    /// Currently accepting votes
    Active,
    /// Passed and successfully executed
    PassedAndExecuted,
    /// Passed but execution failed
    PassedExecutionFailed,
    /// Rejected by community vote
    RejectedByVote,
    /// Expired due to low turnout (didn't meet quorum)
    ExpiredLowTurnout,
    /// Expired due to insufficient votes
    ExpiredInsufficientVotes,
    /// Finalized early due to supermajority
    EarlyFinalized,
    /// Overridden by admin for emergency reasons
    AdminOverridden,
}

/// A single vote cast by a community member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Discord ID of the voter
    pub voter_discord_id: String,
    /// How they voted
    pub vote_type: VoteType,
    /// When the vote was cast
    pub cast_at: Timestamp,
    /// Total voting power of this vote
    pub voting_power: u32,
    /// Detailed breakdown of voting power calculation
    pub voting_power_breakdown: VotingPowerBreakdown,
}

/// Types of votes that can be cast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteType {
    /// Vote in favor of the proposal
    For,
    /// Vote against the proposal
    Against,
    /// Abstain from voting (counts for quorum but not outcome)
    Abstain,
}

/// Detailed breakdown of how voting power was calculated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingPowerBreakdown {
    /// Total voting power
    pub total: u32,
    /// Base power (every registered player gets 1)
    pub base: u32,
    /// Bonus from participation history
    pub participation: u32,
    /// Bonus from current streak
    pub streak: u32,
    /// Bonus from account consistency/age
    pub consistency: u32,
    /// Bonus from recent activity
    pub activity: u32,
}

/// Impact analysis for proposals that change scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalImpactAnalysis {
    /// Number of players estimated to be affected
    pub estimated_players_affected: u32,
    /// Average percentage change in points players can expect
    pub average_points_change_percent: f64,
    /// Maximum points increase any single player could see
    pub maximum_points_increase: u64,
    /// Risk level of this proposal
    pub risk_level: RiskLevel,
    /// Expected impact on player engagement
    pub estimated_engagement_impact: EngagementImpact,
}

/// Risk level classification for proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Less than 10% change in typical rewards
    Low,
    /// 10-25% change in typical rewards
    Medium,
    /// 25-50% change in typical rewards  
    High,
    /// More than 50% change or fundamental system changes
    Critical,
}

/// Expected impact on player engagement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngagementImpact {
    /// Likely to significantly increase engagement
    PositiveHigh,
    /// Likely to moderately increase engagement
    PositiveModerate,
    /// Likely to slightly increase engagement
    PositiveLow,
    /// No significant impact expected
    Neutral,
    /// May slightly decrease engagement
    NegativeLow,
    /// May moderately decrease engagement
    NegativeModerate,
    /// May significantly decrease engagement
    NegativeHigh,
}

/// Safety review results for proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalSafetyReview {
    /// Whether the proposal passed basic safety checks
    pub passed_safety_checks: bool,
    /// Non-blocking warnings about the proposal
    pub warnings: Vec<String>,
    /// Blocking reasons why the proposal cannot proceed
    pub blocked_reasons: Vec<String>,
}

/// Vote eligibility status for a player
#[derive(Debug, Clone)]
pub enum VoteEligibility {
    /// Player is eligible to vote with full power
    Eligible,
    /// Player can vote but with limited voting power
    Limited { voting_power: u32, reason: String },
    /// Player is not eligible to vote
    Ineligible { reason: String },
}

/// Result of casting a vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResult {
    /// Whether the vote was successfully cast
    pub success: bool,
    /// Result message
    pub message: String,
    /// Voting power that was applied
    pub voting_power: u32,
    /// Current vote tally after this vote
    pub current_tally: VoteTally,
    /// Updated proposal status (if changed)
    pub updated_status: Option<ProposalStatus>,
}

/// Current vote tally for a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteTally {
    /// Total voting power for the proposal
    pub votes_for: u32,
    /// Total voting power against the proposal
    pub votes_against: u32,
    /// Total number of voters (including abstains)
    pub total_votes: u32,
    /// Percentage of eligible voters who participated
    pub participation_rate: f64,
}

/// DAO governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Minimum voting power required to create proposals
    pub minimum_proposal_power: u32,
    /// Minimum total votes needed for a proposal to pass
    pub minimum_votes_for_passage: u32,
    /// Minimum percentage of eligible voters that must participate
    pub quorum_percentage: u32,
    /// Default voting period in hours
    pub proposal_duration_hours: u32,
    /// Maximum number of active proposals at once
    pub max_active_proposals: u32,
    /// Hours between proposals from the same user
    pub proposal_rate_limit_hours: u32,
    /// Percentage majority needed for early finalization
    pub early_finalization_threshold: f64,
    /// Whether safety review is required for all proposals
    pub safety_review_required: bool,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            minimum_proposal_power: 5,      // Require some participation history
            minimum_votes_for_passage: 10,  // Absolute minimum votes
            quorum_percentage: 20,          // 20% participation required
            proposal_duration_hours: 168,   // 1 week voting period
            max_active_proposals: 5,        // Max 5 active proposals at once
            proposal_rate_limit_hours: 168, // 1 proposal per week per user
            early_finalization_threshold: 75.0, // 75% majority
            safety_review_required: true,   // Always require safety analysis
        }
    }
}

/// DAO governance management service
#[derive(Debug, Clone)]
pub struct DaoGovernance;

impl DaoGovernance {
    /// Calculate voting power for a player based on their participation and activity
    pub async fn calculate_voting_power(
        &self,
        state: &crate::infrastructure::state::GameHubState,
        discord_id: &str,
    ) -> Result<u32, GameHubError> {
        let player = state.players.get(discord_id).await?
            .ok_or(GameHubError::PlayerNotFound)?;
        
        // Base power - every registered player gets 1 vote
        let base_power = 1u32;
        
        // Participation power - based on event participation (max 5 bonus)
        let participation_count = state.get_player_participation_count(discord_id).await?;
        let participation_power = (participation_count / 10).min(5);
        
        // Streak power - based on current streak (max 3 bonus)
        let current_streak = state.calculate_player_streak(discord_id).await?;
        let streak_power = (current_streak / 7).min(3);
        
        // Consistency power - based on account age (max 2 bonus)
        let days_since_registration = self.calculate_days_since_registration(
            player.created_at, 
            state.runtime.system_time()
        );
        let consistency_power = (days_since_registration / 30).min(2);
        
        // Activity power - bonus for recent participation
        let recent_participation = state.get_recent_participation_count(discord_id, 30).await?;
        let activity_power = if recent_participation > 0 { 1 } else { 0 };
        
        Ok(base_power + participation_power + streak_power + consistency_power + activity_power)
    }
    
    /// Calculate detailed voting power breakdown for transparency
    pub async fn calculate_detailed_voting_power(
        &self,
        state: &crate::infrastructure::state::GameHubState,
        discord_id: &str,
    ) -> Result<VotingPowerBreakdown, GameHubError> {
        let player = state.players.get(discord_id).await?
            .ok_or(GameHubError::PlayerNotFound)?;
            
        // Calculate each component
        let base_power = 1u32;
        let participation_count = state.get_player_participation_count(discord_id).await?;
        let participation_power = (participation_count / 10).min(5);
        let current_streak = state.calculate_player_streak(discord_id).await?;
        let streak_power = (current_streak / 7).min(3);
        let days_since_registration = self.calculate_days_since_registration(
            player.created_at, 
            state.runtime.system_time()
        );
        let consistency_power = (days_since_registration / 30).min(2);
        let recent_participation = state.get_recent_participation_count(discord_id, 30).await?;
        let activity_power = if recent_participation > 0 { 1 } else { 0 };
        
        let total_power = base_power + participation_power + streak_power + consistency_power + activity_power;
        
        Ok(VotingPowerBreakdown {
            total: total_power,
            base: base_power,
            participation: participation_power,
            streak: streak_power,
            consistency: consistency_power,
            activity: activity_power,
        })
    }
    
    /// Calculate days between registration and current time
    fn calculate_days_since_registration(&self, created_at: Timestamp, current_time: Timestamp) -> u32 {
        let seconds_diff = (current_time.micros() - created_at.micros()) / 1_000_000;
        let days_diff = seconds_diff / (24 * 60 * 60);
        days_diff as u32
    }
}

/// Voting anomaly detection for security
#[derive(Debug, Clone)]
pub enum VotingAnomaly {
    /// Suspicious coordinated voting pattern
    CoordinatedVoting { voting_power: u32, suspicious_count: u32 },
    /// Mass voting in short time period
    MassVoting { votes_in_hour: usize, total_power: u32, suspicious: bool },
    /// High concentration of voting power
    PowerConcentration { top_voter_percentage: f64 },
    /// Potential Sybil attack pattern
    SybilSuspicion { similar_accounts: Vec<String> },
}

/// Retry context for governance operations
#[derive(Debug, Clone)]
pub struct RetryContext {
    pub attempt_count: u32,
    pub first_attempt_time: Timestamp,
    pub last_error: Option<GameHubError>,
}

/// Recovery result for failed operations
#[derive(Debug)]
pub enum RecoveryResult {
    ShouldRetry { delay_ms: u64 },
    MaxRetriesExceeded,
    PermanentFailure,
    CriticalFailure,
}

/// Error severity classification
#[derive(Debug, Clone)]
pub enum ErrorSeverity {
    /// Network issues, temporary DB problems
    Transient,
    /// Invalid data, missing resources
    Permanent,
    /// System integrity issues, security problems
    Critical,
}

/// Statistics about governance activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStatistics {
    /// Total number of proposals ever created
    pub total_proposals: u32,
    /// Number of currently active proposals
    pub active_proposals: u32,
    /// Number of proposals that have passed
    pub passed_proposals: u32,
    /// Average voting participation rate across all proposals
    pub average_voting_participation: f64,
    /// Total number of eligible voters
    pub total_eligible_voters: u32,
    /// Average voting power across all eligible voters
    pub average_voting_power: f64,
    /// Number of proposals created this month
    pub proposals_this_month: u32,
}

/// Result of proposal finalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalFinalizationResult {
    /// Proposal that was finalized
    pub proposal_id: String,
    /// Proposal title
    pub title: String,
    /// Final status after finalization
    pub final_status: ProposalStatus,
    /// Result of execution (if applicable)
    pub execution_result: Option<String>,
}