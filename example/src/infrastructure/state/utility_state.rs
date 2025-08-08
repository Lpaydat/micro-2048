// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Utility state operations
//!
//! This module provides shared utility functions for state operations including
//! validation helpers, calculation utilities, and common state processing functions.

use crate::core::types::*;
use super::GameHubState;

/// Validate contract address format
pub async fn validate_contract_address(_state: &GameHubState, contract_address: &str) -> bool {
    // Comprehensive contract address validation
    // Check length (64 characters for standard blockchain addresses)
    if contract_address.len() != 64 {
        return false;
    }
    
    // Check if it's a valid hex string
    if !contract_address.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }
    
    // Check for common invalid patterns
    if contract_address == "0".repeat(64) || // All zeros
       contract_address == "f".repeat(64) || // All f's  
       contract_address == "F".repeat(64) {  // All F's
        return false;
    }
    
    true
}

/// Calculate streak from pending event scores
pub async fn calculate_streak_from_pending(state: &GameHubState, pending_data: &PendingPlayerData) -> u32 {
    // Sort events by participation timestamp
    let mut sorted_events = pending_data.event_scores.clone();
    sorted_events.sort_by_key(|score| score.participation_timestamp);
    
    let mut current_streak = 0;
    let mut last_eligible_timestamp = None;
    
    for event_score in sorted_events {
        if !event_score.streak_eligible {
            continue;
        }
        
        if let Some(last_timestamp) = last_eligible_timestamp {
            // Check if there's a gap that would break the streak
            // Use grace period from scoring configuration
            let scoring_config = state.scoring_config.get();
            let time_diff = event_score.participation_timestamp.micros() - last_timestamp;
            let grace_period_micros = (scoring_config.grace_period_hours as u64) * 60 * 60 * 1_000_000u64;
            
            if time_diff > grace_period_micros {
                current_streak = 1; // Reset streak
            } else {
                current_streak += 1;
            }
        } else {
            current_streak = 1; // First eligible event
        }
        
        last_eligible_timestamp = Some(event_score.participation_timestamp.micros());
    }
    
    current_streak
}

/// Calculate leaderboard rankings for a set of players
pub async fn calculate_leaderboard_rankings(
    state: &GameHubState,
    mut players: Vec<Player>,
) -> Vec<LeaderboardEntry> {
    // Sort players by total points (highest first), then by best streak as tiebreaker
    players.sort_by(|a, b| {
        match b.total_points.cmp(&a.total_points) {
            std::cmp::Ordering::Equal => b.best_streak.cmp(&a.best_streak),
            other => other,
        }
    });
    
    let mut leaderboard = Vec::new();
    for (index, player) in players.iter().enumerate() {
        let rank = (index + 1) as u32;
        
        // Create participation data
        // Get actual streak multiplier from configuration
        let streak_multiplier = get_scoring_multiplier_for_streak(state, player.participation_streak).await;
        
        let participation_data = ParticipationData {
            streak_level: player.participation_streak,
            streak_multiplier: streak_multiplier as u64,
            total_points_earned: player.total_points,
            participation_timestamp: player.last_active,
        };
        
        leaderboard.push(LeaderboardEntry {
            rank,
            player_discord_id: player.discord_id.clone(),
            score: player.total_points,
            participation_data,
        });
    }
    
    leaderboard
}

/// Format timestamp for display  
pub fn format_timestamp_for_display(timestamp: linera_sdk::linera_base_types::Timestamp) -> String {
    // Convert microseconds to seconds since epoch
    let micros = timestamp.micros();
    let seconds = micros / 1_000_000;
    
    // Format as ISO-8601 basic format (YYYY-MM-DD_HH:MM:SS)
    // This is a simplified approach for blockchain context
    // Using Unix epoch conversion for consistent formatting
    let days_since_epoch = seconds / (24 * 60 * 60);
    let remaining_seconds = seconds % (24 * 60 * 60);
    let hours = remaining_seconds / 3600;
    let minutes = (remaining_seconds % 3600) / 60;
    let secs = remaining_seconds % 60;
    
    // Simplified date calculation (approximate)
    let years_since_1970 = days_since_epoch / 365;
    let year = 1970 + years_since_1970;
    let day_of_year = days_since_epoch % 365;
    let month = (day_of_year / 30) + 1; // Rough month approximation
    let day = (day_of_year % 30) + 1;
    
    format!("{:04}-{:02}-{:02}_{:02}:{:02}:{:02}", year, month, day, hours, minutes, secs)
}

/// Calculate time difference in hours
pub fn calculate_time_diff_hours(
    earlier: linera_sdk::linera_base_types::Timestamp,
    later: linera_sdk::linera_base_types::Timestamp,
) -> u64 {
    let diff_micros = later.micros().saturating_sub(earlier.micros());
    diff_micros / (60 * 60 * 1_000_000) // Convert to hours
}

/// Check if timestamp is within grace period
pub async fn is_within_grace_period(
    state: &GameHubState,
    event_timestamp: linera_sdk::linera_base_types::Timestamp,
    current_timestamp: linera_sdk::linera_base_types::Timestamp,
) -> bool {
    // Get grace period from scoring configuration (should be initialized in contract setup)
    let scoring_config = state.scoring_config.get();
    let grace_period_hours = scoring_config.grace_period_hours as u64;
    
    let time_diff_hours = calculate_time_diff_hours(event_timestamp, current_timestamp);
    time_diff_hours <= grace_period_hours
}

/// Generate unique ID with timestamp
pub fn generate_unique_id(prefix: &str, timestamp: linera_sdk::linera_base_types::Timestamp) -> String {
    format!("{}_{}", prefix, timestamp.micros())
}

/// Sanitize string for storage (remove invalid characters)
pub fn sanitize_string_for_storage(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.,!?".contains(*c))
        .collect::<String>()
        .trim()
        .to_string()
}

/// Calculate percentage change
pub fn calculate_percentage_change(old_value: u64, new_value: u64) -> f64 {
    if old_value == 0 {
        if new_value > 0 { 100.0 } else { 0.0 }
    } else {
        ((new_value as f64 - old_value as f64) / old_value as f64) * 100.0
    }
}

/// Get scoring multiplier for streak level
pub async fn get_scoring_multiplier_for_streak(
    state: &GameHubState,
    participation_streak: u32,
) -> u32 {
    // Get scoring configuration (should be initialized in contract setup)
    let scoring_config = state.scoring_config.get();
    
    // Find the best applicable booster level
    let mut best_multiplier = 100u32; // Default 1.0x multiplier
    for booster in &scoring_config.booster_levels {
        if participation_streak >= booster.required_streak {
            best_multiplier = booster.multiplier as u32;
        }
    }
    
    best_multiplier
}

/// Validate and parse CSV line
pub fn parse_csv_line(line: &str, expected_fields: usize) -> Result<Vec<String>, String> {
    let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
    
    if fields.len() < expected_fields {
        return Err(format!("Expected {} fields, got {}", expected_fields, fields.len()));
    }
    
    // Validate that fields are not empty
    for (index, field) in fields.iter().enumerate() {
        if field.is_empty() && index < expected_fields {
            return Err(format!("Field {} is empty", index + 1));
        }
    }
    
    Ok(fields)
}