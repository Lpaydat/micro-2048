// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Calculation helper functions for GraphQL resolvers

use crate::{
    api::graphql_types::PlayerEngagementObject,
    core::types::{ScoringConfig, Player},
};

/// Calculate streak booster level and multiplier
pub fn calculate_streak_booster(streak: u32, config: &ScoringConfig) -> (String, u32) {
    // Find the highest booster level that applies
    let mut best_booster = ("None".to_string(), 100u32); // Base 100% (no boost)
    
    for booster in &config.booster_levels {
        if streak >= booster.required_streak {
            if booster.multiplier > best_booster.1 as u64 {
                best_booster = (booster.name.clone(), booster.multiplier as u32);
            }
        }
    }
    
    best_booster
}

/// Parse date string to timestamp (microseconds)
pub fn parse_date_to_timestamp(date_str: &str) -> Option<u64> {
    // Try to parse as direct timestamp first
    if let Ok(timestamp) = date_str.parse::<u64>() {
        return Some(timestamp);
    }
    
    // Try to parse basic ISO date format: "YYYY-MM-DD"
    if let Some(date_parts) = parse_iso_date_basic(date_str) {
        let (year, month, day) = date_parts;
        
        // Convert to approximate timestamp (simplified calculation)
        // This is a basic approximation for blockchain use
        let days_since_epoch = approximate_days_since_epoch(year, month, day);
        let timestamp_micros = days_since_epoch * 24 * 60 * 60 * 1_000_000;
        
        return Some(timestamp_micros);
    }
    
    // Try to parse with time: "YYYY-MM-DD HH:MM:SS" 
    if let Some((date_parts, time_parts)) = parse_iso_datetime_basic(date_str) {
        let (year, month, day) = date_parts;
        let (hour, minute, second) = time_parts;
        
        let days_since_epoch = approximate_days_since_epoch(year, month, day);
        let seconds_in_day = hour * 3600 + minute * 60 + second;
        let timestamp_micros = (days_since_epoch * 24 * 60 * 60 + seconds_in_day) * 1_000_000;
        
        return Some(timestamp_micros);
    }
    
    None
}

/// Parse basic ISO date format: "YYYY-MM-DD"
fn parse_iso_date_basic(date_str: &str) -> Option<(u64, u32, u32)> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() == 3 {
        if let (Ok(year), Ok(month), Ok(day)) = (
            parts[0].parse::<u64>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>()
        ) {
            if year >= 1970 && month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                return Some((year, month, day));
            }
        }
    }
    None
}

/// Parse basic ISO datetime format: "YYYY-MM-DD HH:MM:SS"
fn parse_iso_datetime_basic(datetime_str: &str) -> Option<((u64, u32, u32), (u64, u64, u64))> {
    let parts: Vec<&str> = datetime_str.split_whitespace().collect();
    if parts.len() == 2 {
        if let Some(date_parts) = parse_iso_date_basic(parts[0]) {
            let time_parts: Vec<&str> = parts[1].split(':').collect();
            if time_parts.len() == 3 {
                if let (Ok(hour), Ok(minute), Ok(second)) = (
                    time_parts[0].parse::<u64>(),
                    time_parts[1].parse::<u64>(),
                    time_parts[2].parse::<u64>()
                ) {
                    if hour < 24 && minute < 60 && second < 60 {
                        return Some((date_parts, (hour, minute, second)));
                    }
                }
            }
        }
    }
    None
}

/// Approximate days since Unix epoch (simplified calculation)
fn approximate_days_since_epoch(year: u64, month: u32, day: u32) -> u64 {
    // Simplified calculation - doesn't account for leap years perfectly
    let years_since_1970 = year - 1970;
    let days_from_years = years_since_1970 * 365 + years_since_1970 / 4; // Rough leap year approximation
    let days_from_months = match month {
        1 => 0,
        2 => 31,
        3 => 59,
        4 => 90,
        5 => 120,
        6 => 151,
        7 => 181,
        8 => 212,
        9 => 243,
        10 => 273,
        11 => 304,
        12 => 334,
        _ => 0,
    };
    
    days_from_years + days_from_months + (day - 1) as u64
}

/// Calculate daily engagement metrics
pub fn calculate_daily_engagement(players: &[Player], start_ts: u64, end_ts: u64) -> Vec<PlayerEngagementObject> {
    let mut engagement_data = Vec::new();
    
    // Ensure valid time range
    if end_ts <= start_ts || players.is_empty() {
        return engagement_data;
    }
    
    // Calculate number of days in range
    let days_in_range = ((end_ts - start_ts) / (24 * 60 * 60 * 1_000_000)).max(1).min(365); // Cap at 1 year
    
    // Group players by activity day
    for day in 0..days_in_range {
        let day_start = start_ts + (day * 24 * 60 * 60 * 1_000_000);
        let day_end = day_start + (24 * 60 * 60 * 1_000_000);
        
        // Count players active during this day
        let active_count = players.iter()
            .filter(|player| {
                player.last_active.micros() >= day_start && 
                player.last_active.micros() < day_end
            })
            .count() as u32;
        
        // Only include days with activity
        if active_count > 0 {
            // Calculate engagement rate (percentage of total players active)
            let _engagement_rate = if players.len() > 0 {
                (active_count as f64 / players.len() as f64 * 100.0).round() as u32
            } else {
                0
            };
            
            // Format date as simple day identifier
            let day_timestamp = day_start + (12 * 60 * 60 * 1_000_000); // Use noon as representative time
            let date = format_timestamp_to_date(day_timestamp);
            
            engagement_data.push(PlayerEngagementObject {
                date,
                active_users: active_count,
                new_registrations: 0, // Would need registration tracking for this day
                total_events: 0, // Would need event tracking for this day  
                total_participation: active_count, // Use active_count as participation approximation
            });
        }
    }
    
    engagement_data
}

/// Format timestamp to simple date string
fn format_timestamp_to_date(timestamp_micros: u64) -> String {
    let seconds = timestamp_micros / 1_000_000;
    let days_since_epoch = seconds / (24 * 60 * 60);
    
    // Approximate year/month/day calculation (simplified)
    let years_since_1970 = days_since_epoch / 365;
    let remaining_days = days_since_epoch % 365;
    let month = (remaining_days / 30) + 1; // Rough month approximation
    let day = (remaining_days % 30) + 1;
    
    format!("{:04}-{:02}-{:02}", 1970 + years_since_1970, month.min(12), day.min(31))
}