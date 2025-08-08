// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Configuration-related GraphQL mutations

use std::sync::Arc;
use async_graphql::{Object, Result};
use crate::{
    infrastructure::operations::Operation,
    core::types::{BoosterLevel, ScoringConfig, StreakResetRules},
    ScheduleOperation,
};

/// Configuration mutation resolvers
#[derive(Clone)]
pub struct ConfigMutations {
    pub runtime: Arc<dyn ScheduleOperation>,
}

#[Object]
impl ConfigMutations {
    /// Update scoring configuration (admin only)
    async fn update_scoring_config(
        &self,
        admin_discord_id: String,
        grace_period_hours: Option<i32>,
        bronze_threshold: Option<i32>,
        silver_threshold: Option<i32>, 
        gold_threshold: Option<i32>,
        bronze_multiplier: Option<i32>,
        silver_multiplier: Option<i32>,
        gold_multiplier: Option<i32>
    ) -> Result<String> {
        // Validate admin Discord ID format
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid admin Discord ID: {}", e)))?;
        
        // Validate grace period
        if let Some(hours) = grace_period_hours {
            if hours < 1 || hours > 168 { // 1 hour to 1 week
                return Err(async_graphql::Error::new("Grace period must be between 1 and 168 hours (1 week)"));
            }
        }
        
        // Validate thresholds and multipliers
        let validate_threshold_multiplier = |threshold: Option<i32>, multiplier: Option<i32>, name: &str| -> Result<(), async_graphql::Error> {
            match (threshold, multiplier) {
                (Some(t), Some(m)) => {
                    if t < 1 || t > 100 {
                        return Err(async_graphql::Error::new(format!("{} threshold must be between 1 and 100", name)));
                    }
                    if m < 100 || m > 1000 { // 100% (no boost) to 1000% (10x boost)
                        return Err(async_graphql::Error::new(format!("{} multiplier must be between 100 and 1000", name)));
                    }
                },
                (Some(_), None) => {
                    return Err(async_graphql::Error::new(format!("{} threshold provided but no multiplier", name)));
                },
                (None, Some(_)) => {
                    return Err(async_graphql::Error::new(format!("{} multiplier provided but no threshold", name)));
                },
                (None, None) => {} // Valid - no booster level configured
            }
            Ok(())
        };
        
        validate_threshold_multiplier(bronze_threshold, bronze_multiplier, "Bronze")?;
        validate_threshold_multiplier(silver_threshold, silver_multiplier, "Silver")?;
        validate_threshold_multiplier(gold_threshold, gold_multiplier, "Gold")?;
        
        // Build booster levels from parameters
        let mut booster_levels = Vec::new();
        
        if let (Some(threshold), Some(multiplier)) = (bronze_threshold, bronze_multiplier) {
            booster_levels.push(BoosterLevel {
                name: "Bronze".to_string(),
                required_streak: threshold as u32,
                multiplier: multiplier as u64,
            });
        }
        
        if let (Some(threshold), Some(multiplier)) = (silver_threshold, silver_multiplier) {
            booster_levels.push(BoosterLevel {
                name: "Silver".to_string(),
                required_streak: threshold as u32,
                multiplier: multiplier as u64,
            });
        }
        
        if let (Some(threshold), Some(multiplier)) = (gold_threshold, gold_multiplier) {
            booster_levels.push(BoosterLevel {
                name: "Gold".to_string(),
                required_streak: threshold as u32,
                multiplier: multiplier as u64,
            });
        }

        // Create the complete scoring config
        let config = ScoringConfig {
            grace_period_hours: grace_period_hours.map(|h| h as u32).unwrap_or(48),
            booster_levels,
            streak_reset_rules: StreakResetRules::default(),
        };

        // Schedule scoring config update operation
        self.runtime.schedule_operation(&Operation::UpdateScoringConfig {
            caller_discord_id: admin_discord_id,
            config,
        });
        
        Ok("Scoring configuration update scheduled".to_string())
    }

    /// Import leaderboard data (admin only)
    async fn import_leaderboard_data(&self, admin_discord_id: String, csv_data: String) -> Result<String> {
        // Validate admin Discord ID format
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid admin Discord ID: {}", e)))?;
        
        // Basic CSV validation
        let trimmed_csv = csv_data.trim();
        if trimmed_csv.is_empty() {
            return Err(async_graphql::Error::new("CSV data cannot be empty"));
        }
        
        let lines = trimmed_csv.lines().count();
        if lines > 10000 {
            return Err(async_graphql::Error::new("CSV data too large - maximum 10,000 lines allowed"));
        }
        
        if lines < 2 {
            return Err(async_graphql::Error::new("CSV data must contain at least a header and one data row"));
        }
        
        // Validate CSV data size
        if trimmed_csv.len() > 5_000_000 { // 5MB limit
            return Err(async_graphql::Error::new("CSV data too large - maximum 5MB allowed"));
        }
        
        // Basic CSV format validation (check for header structure)
        let sample_lines: Vec<&str> = trimmed_csv.lines().take(3).collect();
        if let Some(header) = sample_lines.first() {
            let header_parts: Vec<&str> = header.split(',').map(|s| s.trim()).collect();
            if header_parts.len() < 2 {
                return Err(async_graphql::Error::new("CSV header must contain at least 2 columns"));
            }
        }
        
        // Schedule import operation
        self.runtime.schedule_operation(&Operation::ImportLeaderboardData {
            caller_discord_id: admin_discord_id.clone(),
            csv_data: trimmed_csv.to_string(),
        });
        
        Ok(format!("CSV import scheduled by {} ({} lines to process)", admin_discord_id, lines - 1)) // -1 for header
    }
}