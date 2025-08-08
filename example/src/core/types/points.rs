use serde::{Deserialize, Serialize};
use linera_sdk::linera_base_types::Timestamp;
use std::collections::HashMap;
use crate::infrastructure::errors::GameHubError;

/// Configuration for participation points calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationPointsConfig {
    /// Configuration version number for tracking changes
    pub version: u32,
    /// When this configuration becomes effective
    pub effective_from: Timestamp,
    /// Base points awarded for each qualifying event participation
    pub base_points_per_event: u64,
    /// Type of streak bonus to apply
    pub streak_bonus_type: StreakBonusType,
    /// Winner bonus configuration (optional)
    pub winner_bonus: Option<WinnerBonusConfig>,
    /// Rank-based bonuses (rank -> bonus points)
    pub rank_bonuses: HashMap<u32, u64>,
    /// Multiplier for special events
    pub special_event_multiplier: f64,
    /// Who created this configuration
    pub created_by: ConfigCreationType,
    /// When this configuration was approved (for DAO governance)
    pub approved_at: Option<Timestamp>,
}

impl Default for ParticipationPointsConfig {
    fn default() -> Self {
        Self {
            version: 1,
            effective_from: Timestamp::from(0),
            base_points_per_event: 100,
            streak_bonus_type: StreakBonusType::Linear { points_per_streak: 10 },
            winner_bonus: None,
            rank_bonuses: HashMap::new(),
            special_event_multiplier: 1.0,
            created_by: ConfigCreationType::SystemDefault,
            approved_at: None,
        }
    }
}

/// Different types of streak bonuses available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreakBonusType {
    /// No streak bonus
    None,
    /// Linear bonus: bonus = streak * points_per_streak
    Linear { points_per_streak: u64 },
    /// Static bonus after threshold: fixed bonus once streak reaches threshold
    Static { bonus_after_threshold: u64, threshold: u32 },
    /// Exponential bonus: bonus = base^streak, capped at maximum
    Exponential { base: f64, cap: u64 },
    /// Progressive tiers with different bonuses at different streak levels
    Progressive { tiers: Vec<StreakTier> },
}

/// Tier in progressive streak bonus system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakTier {
    /// Minimum streak to qualify for this tier
    pub min_streak: u32,
    /// Maximum streak for this tier (None = unlimited)
    pub max_streak: Option<u32>,
    /// Bonus points for this tier
    pub bonus_points: u64,
}

/// Winner bonus configuration based on placement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinnerBonusConfig {
    /// Bonus for 1st place
    pub first_place: u64,
    /// Bonus for 2nd place
    pub second_place: u64,
    /// Bonus for 3rd place
    pub third_place: u64,
    /// Bonus for top 10% of participants
    pub top_10_percent: u64,
    /// Bonus for top 25% of participants
    pub top_25_percent: u64,
}

impl Default for WinnerBonusConfig {
    fn default() -> Self {
        Self {
            first_place: 500,
            second_place: 300,
            third_place: 200,
            top_10_percent: 100,
            top_25_percent: 50,
        }
    }
}

/// How this configuration was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigCreationType {
    /// Set by system admin
    AdminSet { admin_id: String },
    /// Approved by DAO governance
    DaoApproved { 
        proposal_id: String, 
        votes_for: u32, 
        votes_against: u32 
    },
    /// System default configuration
    SystemDefault,
}

/// Complete transaction record for points awarded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointsTransaction {
    /// Event where points were earned
    pub event_id: String,
    /// Player who earned the points
    pub player_discord_id: String,
    /// Base points from participation
    pub base_points: u64,
    /// Bonus points from streak
    pub streak_bonus: u64,
    /// Bonus points from ranking/placement
    pub winner_bonus: u64,
    /// Special bonus points (game-specific, events, etc.)
    pub special_bonus: u64,
    /// Total points awarded
    pub total: u64,
    /// When points were awarded
    pub timestamp: Timestamp,
    /// Configuration version used for calculation
    pub config_version: u32,
    /// Detailed breakdown for transparency
    pub breakdown: PointsBreakdown,
}

/// Detailed breakdown of how points were calculated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointsBreakdown {
    /// Base participation points
    pub base_points: u64,
    /// Streak bonus amount
    pub streak_bonus: u64,
    /// Streak value used for calculation
    pub streak_value: u32,
    /// Winner/ranking bonus amount
    pub winner_bonus: u64,
    /// Player's rank in the event
    pub player_rank: Option<u32>,
    /// Total participants in the event
    pub total_participants: Option<u32>,
    /// Additional special condition bonuses
    pub special_condition_bonuses: Vec<SpecialConditionBonus>,
}

/// Special condition bonuses (game-specific, achievements, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialConditionBonus {
    /// Name of the condition that triggered the bonus
    pub condition_name: String,
    /// Description of what earned this bonus
    pub description: String,
    /// Bonus points awarded
    pub bonus_points: u64,
    /// Whether player qualified for this bonus
    pub qualified: bool,
}

/// Points calculation engine
pub struct PointsCalculator;

impl PointsCalculator {
    /// Calculate base participation points including streak bonus
    pub fn calculate_participation_points(
        streak: u32,
        config: &ParticipationPointsConfig,
    ) -> u64 {
        let base_points = config.base_points_per_event;
        let streak_bonus = Self::calculate_streak_bonus(streak, &config.streak_bonus_type);
        
        let total = base_points + streak_bonus;
        
        // Apply special event multiplier if configured
        if config.special_event_multiplier != 1.0 {
            (total as f64 * config.special_event_multiplier) as u64
        } else {
            total
        }
    }
    
    /// Calculate streak bonus based on configuration type
    pub fn calculate_streak_bonus(streak: u32, bonus_type: &StreakBonusType) -> u64 {
        match bonus_type {
            StreakBonusType::None => 0,
            StreakBonusType::Linear { points_per_streak } => {
                streak as u64 * points_per_streak
            },
            StreakBonusType::Static { bonus_after_threshold, threshold } => {
                if streak >= *threshold {
                    *bonus_after_threshold
                } else {
                    0
                }
            },
            StreakBonusType::Exponential { base, cap } => {
                let bonus = base.powi(streak as i32) as u64;
                bonus.min(*cap)
            },
            StreakBonusType::Progressive { tiers } => {
                for tier in tiers.iter().rev() { // Check highest tiers first
                    if streak >= tier.min_streak {
                        if let Some(max_streak) = tier.max_streak {
                            if streak <= max_streak {
                                return tier.bonus_points;
                            }
                        } else {
                            return tier.bonus_points;
                        }
                    }
                }
                0 // No tier matched
            },
        }
    }
    
    /// Calculate winner bonus based on player ranking
    pub fn calculate_winner_bonus(
        player_rank: u32,
        total_participants: u32,
        winner_config: &WinnerBonusConfig,
    ) -> u64 {
        match player_rank {
            1 => winner_config.first_place,
            2 => winner_config.second_place,
            3 => winner_config.third_place,
            _ => {
                let top_10_threshold = (total_participants as f64 * 0.1).ceil() as u32;
                let top_25_threshold = (total_participants as f64 * 0.25).ceil() as u32;
                
                if player_rank <= top_10_threshold {
                    winner_config.top_10_percent
                } else if player_rank <= top_25_threshold {
                    winner_config.top_25_percent
                } else {
                    0
                }
            }
        }
    }
    
    /// Calculate game-specific points with custom scoring rules
    pub fn calculate_game_specific_points(
        base_points: u64,
        streak: u32,
        player_rank: Option<u32>,
        total_participants: Option<u32>,
        custom_rules: &Option<CustomScoringRules>,
    ) -> u64 {
        let mut total_points = base_points;
        
        if let Some(rules) = custom_rules {
            // Apply game-specific multiplier
            total_points = (total_points as f64 * rules.point_multiplier) as u64;
            
            // Apply custom streak bonus if configured
            if let Some(ref custom_streak_bonus) = rules.streak_bonus_override {
                let custom_bonus = Self::calculate_streak_bonus(streak, custom_streak_bonus);
                // Replace the original streak bonus with custom one
                total_points = total_points - Self::calculate_streak_bonus(streak, &StreakBonusType::None) + custom_bonus;
            }
            
            // Apply custom winner bonus if configured
            if let (Some(rank), Some(total), Some(ref custom_winner_bonus)) = 
                (player_rank, total_participants, &rules.winner_bonus_override) {
                let custom_winner_points = Self::calculate_winner_bonus(rank, total, custom_winner_bonus);
                total_points += custom_winner_points;
            }
        }
        
        total_points
    }
}

/// Custom scoring rules for specific games
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomScoringRules {
    /// Multiplier applied to all points for this game
    pub point_multiplier: f64,
    /// Override streak bonus calculation for this game
    pub streak_bonus_override: Option<StreakBonusType>,
    /// Override winner bonus calculation for this game
    pub winner_bonus_override: Option<WinnerBonusConfig>,
    /// Special conditions that can award bonus points
    pub special_conditions: Vec<SpecialCondition>,
}

/// Special condition that can trigger bonus points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialCondition {
    /// Name of the condition
    pub condition_name: String,
    /// Description of what triggers this condition
    pub description: String,
    /// Bonus points awarded when condition is met
    pub bonus_points: u64,
    /// Who this condition applies to
    pub applies_to: ConditionTarget,
}

/// Target for special condition bonuses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionTarget {
    /// All participants qualify
    AllParticipants,
    /// Top N players qualify
    TopNPlayers { n: u32 },
    /// Players with scores in a range
    PlayersWithScore { min: u64, max: Option<u64> },
    /// First-time participants in this game
    FirstTimeParticipants,
    /// Players with active streaks
    StreakPlayers { minimum_streak: u32 },
}

/// Calculation result for potential points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointsCalculation {
    /// Base participation points
    pub base_points: u64,
    /// Streak bonus points
    pub streak_bonus: u64,
    /// Winner/ranking bonus points
    pub winner_bonus: u64,
    /// Total points that would be awarded
    pub total_points: u64,
    /// Detailed breakdown
    pub breakdown: PointsBreakdown,
    /// Configuration version used
    pub config_version: u32,
}

/// Historical points transaction query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPointsHistory {
    /// Player's Discord ID
    pub discord_id: String,
    /// Total transactions returned
    pub total_transactions: u32,
    /// All point transactions
    pub transactions: Vec<PointsTransaction>,
    /// Summary statistics
    pub summary: PointsHistorySummary,
}

/// Summary of player's points history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointsHistorySummary {
    /// Total points earned from all transactions
    pub total_points_earned: u64,
    /// Average points per event
    pub average_points_per_event: f64,
    /// Number of events participated in
    pub events_participated: u32,
    /// Highest single transaction points
    pub highest_single_transaction: u64,
    /// Most common streak bonus received
    pub most_common_streak: u32,
}