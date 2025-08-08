// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Scoring validation utilities

use crate::infrastructure::errors::GameHubError;

/// Maximum values for scoring-related fields
pub const MAX_DURATION_HOURS: u32 = 8760; // 1 year
pub const MAX_LEADERBOARD_SIZE: usize = 1000;
pub const MAX_BATCH_SIZE: usize = 1000;

/// Scoring validation utilities
pub struct ScoringValidator;

impl ScoringValidator {
    /// Validate duration in hours
    pub fn validate_duration_hours(hours: Option<u32>) -> Result<(), GameHubError> {
        if let Some(h) = hours {
            if h == 0 {
                return Err(GameHubError::InvalidDuration {
                    reason: "Duration must be greater than 0".to_string(),
                });
            }

            if h > MAX_DURATION_HOURS {
                return Err(GameHubError::InvalidDuration {
                    reason: format!("Duration cannot exceed {} hours", MAX_DURATION_HOURS),
                });
            }
        }
        Ok(())
    }

    /// Validate leaderboard size
    pub fn validate_leaderboard_size(size: usize) -> Result<(), GameHubError> {
        if size == 0 {
            return Err(GameHubError::InvalidLeaderboardSize {
                reason: "Leaderboard size must be greater than 0".to_string(),
            });
        }

        if size > MAX_LEADERBOARD_SIZE {
            return Err(GameHubError::InvalidLeaderboardSize {
                reason: format!("Leaderboard size cannot exceed {}", MAX_LEADERBOARD_SIZE),
            });
        }

        Ok(())
    }

    /// Validate multiplier value
    pub fn validate_multiplier(multiplier: f64, field_name: &str) -> Result<(), GameHubError> {
        if multiplier < 0.0 {
            return Err(GameHubError::InvalidMultiplier {
                field: field_name.to_string(),
                reason: "Multiplier cannot be negative".to_string(),
            });
        }

        if multiplier > 10.0 {
            return Err(GameHubError::InvalidMultiplier {
                field: field_name.to_string(),
                reason: "Multiplier cannot exceed 10.0".to_string(),
            });
        }

        if multiplier.is_nan() || multiplier.is_infinite() {
            return Err(GameHubError::InvalidMultiplier {
                field: field_name.to_string(),
                reason: "Multiplier must be a valid number".to_string(),
            });
        }

        Ok(())
    }

    /// Validate grace period hours
    pub fn validate_grace_period_hours(hours: u32) -> Result<(), GameHubError> {
        if hours > 168 { // 1 week
            return Err(GameHubError::InvalidGracePeriod {
                reason: "Grace period cannot exceed 168 hours (1 week)".to_string(),
            });
        }
        Ok(())
    }

    /// Validate booster level name
    pub fn validate_booster_level_name(name: &str) -> Result<(), GameHubError> {
        if name.is_empty() {
            return Err(GameHubError::MissingRequiredField {
                field: "booster_level_name".to_string(),
            });
        }

        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(GameHubError::InvalidBoosterLevelName {
                reason: "Booster level name cannot be only whitespace".to_string(),
            });
        }

        if trimmed.len() > 50 {
            return Err(GameHubError::InputTooLong {
                field: "booster_level_name".to_string(),
                max_length: 50,
            });
        }

        Ok(())
    }

    /// Validate batch size for operations
    pub fn validate_batch_size(size: usize, operation: &str) -> Result<(), GameHubError> {
        if size == 0 {
            return Err(GameHubError::InvalidBatchSize {
                operation: operation.to_string(),
                reason: "Batch size must be greater than 0".to_string(),
            });
        }

        if size > MAX_BATCH_SIZE {
            return Err(GameHubError::InvalidBatchSize {
                operation: operation.to_string(),
                reason: format!("Batch size cannot exceed {}", MAX_BATCH_SIZE),
            });
        }

        Ok(())
    }

    /// Validate numeric bounds for any numeric type
    pub fn validate_numeric_bounds<T>(value: T, min: T, max: T, field_name: &str) -> Result<(), GameHubError>
    where
        T: PartialOrd + std::fmt::Display,
    {
        if value < min {
            return Err(GameHubError::ValueTooSmall {
                field: field_name.to_string(),
                min_value: min.to_string(),
            });
        }

        if value > max {
            return Err(GameHubError::ValueTooLarge {
                field: field_name.to_string(),
                max_value: max.to_string(),
            });
        }

        Ok(())
    }
}