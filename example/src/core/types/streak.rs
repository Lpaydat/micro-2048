use serde::{Deserialize, Serialize};
use linera_sdk::linera_base_types::Timestamp;
use crate::core::types::event::Event;
use crate::infrastructure::errors::GameHubError;

/// Day-based streak calculator that processes events by UTC calendar days
#[derive(Debug, Clone)]
pub struct DayBasedStreakCalculator;

impl DayBasedStreakCalculator {
    /// Calculate a player's current streak based on consecutive UTC days with mandatory event participation
    pub async fn calculate_player_streak(
        &self,
        state: &crate::infrastructure::state::GameHubState,
        player_discord_id: &str,
        current_time: Timestamp,
    ) -> Result<StreakCalculation, GameHubError> {
        // Get all player's event participation records
        let participation_index = self.build_participation_index(state, player_discord_id).await?;
        
        if participation_index.is_empty() {
            return Ok(StreakCalculation {
                current_streak: 0,
                last_participation_date: None,
                consecutive_days: Vec::new(),
                next_mandatory_events: self.get_upcoming_mandatory_events(state, current_time).await?,
                calculation_timestamp: current_time,
            });
        }

        // Group events by UTC calendar days
        let participation_days = self.group_events_by_utc_days(&participation_index);
        
        // Calculate consecutive days from most recent backwards
        let consecutive_days = self.calculate_consecutive_days(&participation_days, current_time);
        
        let current_streak = consecutive_days.len() as u32;
        let last_participation_date = participation_days.keys().max().copied();
        
        Ok(StreakCalculation {
            current_streak,
            last_participation_date,
            consecutive_days,
            next_mandatory_events: self.get_upcoming_mandatory_events(state, current_time).await?,
            calculation_timestamp: current_time,
        })
    }

    /// Build index of all player's event participation with scores that met thresholds
    async fn build_participation_index(
        &self,
        state: &crate::infrastructure::state::GameHubState,
        player_discord_id: &str,
    ) -> Result<Vec<ParticipationRecord>, GameHubError> {
        let mut participation_records = Vec::new();
        
        // Get all event participation keys for this player
        let participation_keys = state.event_participation.indices().await?;
        let player_prefix = format!("{}:", player_discord_id);
        
        for key in participation_keys {
            if key.starts_with(&player_prefix) {
                if let Ok(Some(participation)) = state.event_participation.get(&key).await {
                    // Only include participation that met the game's threshold requirements
                    if participation.meets_threshold {
                        let event_id = participation.event_id.clone();
                        
                        // Get the event details to extract timestamp
                        if let Ok(Some(event)) = state.events.get(&event_id).await {
                            participation_records.push(ParticipationRecord {
                                event_id,
                                event_timestamp: event.end_time, // Use event end time for day grouping
                                final_score: participation.final_score.unwrap_or(participation.latest_score),
                                met_threshold: true,
                            });
                        }
                    }
                }
            }
        }
        
        // Sort by timestamp for processing
        participation_records.sort_by_key(|record| record.event_timestamp);
        
        Ok(participation_records)
    }

    /// Group participation records by UTC calendar days
    fn group_events_by_utc_days(
        &self,
        participation_records: &[ParticipationRecord],
    ) -> std::collections::BTreeMap<u32, Vec<ParticipationRecord>> {
        let mut days_map = std::collections::BTreeMap::new();
        
        for record in participation_records {
            let utc_day = self.timestamp_to_utc_day(record.event_timestamp);
            days_map.entry(utc_day).or_insert_with(Vec::new).push(record.clone());
        }
        
        days_map
    }

    /// Calculate consecutive days working backwards from the most recent participation
    fn calculate_consecutive_days(
        &self,
        participation_days: &std::collections::BTreeMap<u32, Vec<ParticipationRecord>>,
        current_time: Timestamp,
    ) -> Vec<MandatoryEventDay> {
        if participation_days.is_empty() {
            return Vec::new();
        }

        let current_utc_day = self.timestamp_to_utc_day(current_time);
        let mut consecutive_days = Vec::new();
        
        // Get the most recent participation day
        if let Some(&last_participation_day) = participation_days.keys().max() {
            let mut checking_day = last_participation_day;
            
            // Work backwards to find consecutive days
            loop {
                if let Some(day_events) = participation_days.get(&checking_day) {
                    // This day has qualifying participation
                    consecutive_days.push(MandatoryEventDay {
                        utc_day: checking_day,
                        events_participated: day_events.iter().map(|r| r.event_id.clone()).collect(),
                        mandatory_event_available: true, // Assume events were mandatory if player participated
                    });
                    
                    // Check if this is consecutive to the previous day
                    if checking_day == 0 {
                        break; // Can't go before day 0
                    }
                    checking_day -= 1;
                } else {
                    // Gap found - streak ends here
                    break;
                }
            }
        }
        
        // Reverse to get chronological order (oldest first)
        consecutive_days.reverse();
        consecutive_days
    }

    /// Convert timestamp to UTC day number (days since epoch)
    fn timestamp_to_utc_day(&self, timestamp: Timestamp) -> u32 {
        // Convert microseconds to seconds, then to days
        let seconds_since_epoch = timestamp.micros() / 1_000_000;
        let days_since_epoch = seconds_since_epoch / (24 * 60 * 60);
        days_since_epoch as u32
    }

    /// Get upcoming mandatory events that could extend or break the streak
    async fn get_upcoming_mandatory_events(
        &self,
        state: &crate::infrastructure::state::GameHubState,
        current_time: Timestamp,
    ) -> Result<Vec<Event>, GameHubError> {
        let mut upcoming_events = Vec::new();
        let event_ids = state.events.indices().await?;
        
        // Look for events in the next 7 days that are still active
        let seven_days_from_now = current_time + (7 * 24 * 60 * 60 * 1_000_000); // 7 days in microseconds
        
        for event_id in event_ids {
            if let Ok(Some(event)) = state.events.get(&event_id).await {
                // Include events that are currently running or will start soon
                if event.start_time <= seven_days_from_now && event.end_time >= current_time {
                    upcoming_events.push(event);
                }
            }
        }
        
        // Sort by start time
        upcoming_events.sort_by_key(|event| event.start_time);
        
        // Limit to next 10 events to avoid overwhelming responses
        upcoming_events.truncate(10);
        
        Ok(upcoming_events)
    }
}

/// Result of streak calculation with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakCalculation {
    /// Current consecutive streak count
    pub current_streak: u32,
    /// Last UTC day the player participated (as day number since epoch)
    pub last_participation_date: Option<u32>,
    /// Detailed breakdown of consecutive days
    pub consecutive_days: Vec<MandatoryEventDay>,
    /// Upcoming events that could affect the streak
    pub next_mandatory_events: Vec<Event>,
    /// When this calculation was performed
    pub calculation_timestamp: Timestamp,
}

/// Represents a single day in the player's streak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MandatoryEventDay {
    /// UTC day number (days since epoch)
    pub utc_day: u32,
    /// Events the player participated in on this day
    pub events_participated: Vec<String>,
    /// Whether there was a mandatory event available on this day
    pub mandatory_event_available: bool,
}

/// Internal record of player's qualifying event participation
#[derive(Debug, Clone)]
pub struct ParticipationRecord {
    /// Event ID
    pub event_id: String,
    /// When the event ended (used for day grouping)
    pub event_timestamp: Timestamp,
    /// Player's final score in the event
    pub final_score: u64,
    /// Whether the participation met the game's threshold requirements
    pub met_threshold: bool,
}

/// Result of participation validation for streak purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationResult {
    /// Whether this participation counts toward streak maintenance
    pub counts_for_streak: bool,
    /// The UTC day this participation occurred
    pub participation_day: u32,
    /// Reason why participation did/didn't count
    pub validation_reason: String,
    /// Player's score in the event
    pub player_score: u64,
    /// Minimum score required for this game
    pub required_threshold: u64,
}

/// Migration comparison between legacy and new streak calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakComparison {
    pub discord_id: String,
    pub legacy_calculation: u32,
    pub new_calculation: u32,
    pub difference: i32,
    pub migration_status: MigrationStatus,
}

/// Status of streak migration for a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationStatus {
    NotMigrated,
    MigratedSuccessfully,
    MigrationFailed,
    UnderReview,
}

/// Cached streak data for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedStreakData {
    pub streak_value: u32,
    pub calculated_at: Timestamp,
    pub needs_recalculation: bool,
    pub last_participation_day: Option<u32>,
}

/// Report from streak migration validation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    pub total_players: u32,
    pub significant_differences: u32,
    pub average_difference: f64,
    pub acceptable: bool,
}

/// Migration validation service
pub struct MigrationValidator;

impl MigrationValidator {
    pub async fn validate_streak_migration(
        &self,
        state: &crate::infrastructure::state::GameHubState,
    ) -> Result<MigrationReport, GameHubError> {
        let player_ids = state.players.indices().await?;
        let mut total_players = 0;
        let mut significant_differences = 0;
        let mut average_difference = 0.0;
        
        let calculator = DayBasedStreakCalculator;
        let current_time = state.runtime.system_time();
        
        for player_id in player_ids {
            let legacy = state.calculate_player_streak_legacy(&player_id).await?;
            let new_calculation = calculator.calculate_player_streak(state, &player_id, current_time).await?;
            let new = new_calculation.current_streak;
            
            let difference = (new as i32 - legacy as i32).abs();
            if difference > 2 { // Allow small differences due to different calculation methods
                significant_differences += 1;
            }
            
            average_difference += difference as f64;
            total_players += 1;
        }
        
        if total_players > 0 {
            average_difference /= total_players as f64;
        }
        
        Ok(MigrationReport {
            total_players,
            significant_differences,
            average_difference,
            acceptable: significant_differences < (total_players / 10), // <10% significant differences
        })
    }
}