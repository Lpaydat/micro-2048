// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Player validation utilities

use crate::infrastructure::errors::GameHubError;
use linera_sdk::linera_base_types::Timestamp;
use std::collections::HashSet;

/// Maximum lengths for player-related fields
pub const MAX_DISCORD_ID_LENGTH: usize = 20;
pub const MIN_DISCORD_ID_LENGTH: usize = 10;
pub const MAX_USERNAME_LENGTH: usize = 100;
pub const MIN_USERNAME_LENGTH: usize = 1;
pub const MAX_AVATAR_URL_LENGTH: usize = 500;
pub const MAX_SCORE_VALUE: u64 = 1_000_000_000;
pub const MAX_STREAK_VALUE: u32 = 10_000;

/// Player validation utilities
pub struct PlayerValidator;

impl PlayerValidator {
    /// Validate Discord ID format and constraints
    pub fn validate_discord_id(discord_id: &str) -> Result<(), GameHubError> {
        if discord_id.is_empty() {
            return Err(GameHubError::MissingRequiredField {
                field: "discord_id".to_string(),
            });
        }

        if discord_id.len() < MIN_DISCORD_ID_LENGTH {
            return Err(GameHubError::InputTooShort {
                field: "discord_id".to_string(),
                min_length: MIN_DISCORD_ID_LENGTH,
            });
        }

        if discord_id.len() > MAX_DISCORD_ID_LENGTH {
            return Err(GameHubError::InputTooLong {
                field: "discord_id".to_string(),
                max_length: MAX_DISCORD_ID_LENGTH,
            });
        }

        // Discord IDs should be numeric
        if !discord_id.chars().all(|c| c.is_ascii_digit()) {
            return Err(GameHubError::InvalidDiscordId {
                reason: "Discord ID must contain only numeric characters".to_string(),
            });
        }

        Ok(())
    }

    /// Validate username format and constraints
    pub fn validate_username(username: &str) -> Result<(), GameHubError> {
        if username.is_empty() {
            return Err(GameHubError::MissingRequiredField {
                field: "username".to_string(),
            });
        }

        // Check for prohibited characters BEFORE trimming
        if username.contains('\0') {
            return Err(GameHubError::InvalidCharacter {
                field: "username".to_string(),
                reason: "Username cannot contain null characters".to_string(),
            });
        }

        if username.contains('\t') || username.contains('\n') || username.contains('\r') {
            return Err(GameHubError::InvalidUsername {
                reason: "Username cannot contain tab or newline characters".to_string(),
            });
        }

        let trimmed = username.trim();
        if trimmed.is_empty() {
            return Err(GameHubError::InvalidUsername {
                reason: "Username cannot be only whitespace".to_string(),
            });
        }

        if trimmed.len() < MIN_USERNAME_LENGTH {
            return Err(GameHubError::InputTooShort {
                field: "username".to_string(),
                min_length: MIN_USERNAME_LENGTH,
            });
        }

        if trimmed.len() > MAX_USERNAME_LENGTH {
            return Err(GameHubError::InputTooLong {
                field: "username".to_string(),
                max_length: MAX_USERNAME_LENGTH,
            });
        }

        Ok(())
    }

    /// Validate avatar URL format and constraints
    pub fn validate_avatar_url(url: &str) -> Result<(), GameHubError> {
        if url.is_empty() {
            return Ok(()); // Avatar URL is optional
        }

        if url.len() > MAX_AVATAR_URL_LENGTH {
            return Err(GameHubError::InputTooLong {
                field: "avatar_url".to_string(),
                max_length: MAX_AVATAR_URL_LENGTH,
            });
        }

        // Check for invalid characters (null bytes, control characters)
        if url.contains('\0') {
            return Err(GameHubError::InvalidCharacter {
                field: "avatar_url".to_string(),
                reason: "Avatar URL cannot contain null characters".to_string(),
            });
        }

        // Basic URL format validation
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(GameHubError::InvalidUrl {
                reason: "Avatar URL must use http:// or https:// protocol".to_string(),
            });
        }

        Ok(())
    }

    /// Validate score value
    pub fn validate_score(score: u64) -> Result<(), GameHubError> {
        if score > MAX_SCORE_VALUE {
            return Err(GameHubError::InvalidScore {
                reason: format!("Score cannot exceed {}", MAX_SCORE_VALUE),
            });
        }
        Ok(())
    }

    /// Validate streak value
    pub fn validate_streak(streak: u32) -> Result<(), GameHubError> {
        if streak > MAX_STREAK_VALUE {
            return Err(GameHubError::InvalidStreak {
                reason: format!("Streak cannot exceed {}", MAX_STREAK_VALUE),
            });
        }
        Ok(())
    }

    /// Validate no duplicate Discord IDs in a collection
    pub fn validate_no_duplicate_discord_ids(discord_ids: &[String]) -> Result<(), GameHubError> {
        let mut seen = HashSet::new();
        let mut duplicates = Vec::new();

        for id in discord_ids {
            if !seen.insert(id) {
                duplicates.push(id.clone());
            }
        }

        if !duplicates.is_empty() {
            return Err(GameHubError::DuplicateDiscordIds { ids: duplicates });
        }

        Ok(())
    }

    /// Validate complete player registration data
    pub fn validate_complete_player_registration(
        discord_id: &str,
        username: &str,
        avatar_url: Option<&str>,
    ) -> Result<(), GameHubError> {
        Self::validate_discord_id(discord_id)?;
        Self::validate_username(username)?;
        
        if let Some(url) = avatar_url {
            Self::validate_avatar_url(url)?;
        }
        
        Ok(())
    }

    /// Validate timestamp is not in the future
    pub fn validate_timestamp_not_future(timestamp: Timestamp, current_time: Timestamp) -> Result<(), GameHubError> {
        if timestamp.micros() > current_time.micros() {
            return Err(GameHubError::InvalidTimestamp {
                reason: "Timestamp cannot be in the future".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_discord_id_valid_cases() {
        // Test the exact ID from our failing integration test
        let test_id = "10000000000000000010";
        assert!(PlayerValidator::validate_discord_id(test_id).is_ok());
        
        // Test other valid IDs
        assert!(PlayerValidator::validate_discord_id("123456789012345678").is_ok());
        assert!(PlayerValidator::validate_discord_id("1234567890").is_ok()); // Minimum length
        assert!(PlayerValidator::validate_discord_id("12345678901234567890").is_ok()); // Maximum length
    }

    #[test]
    fn test_validate_discord_id_invalid_cases() {
        // Empty string
        assert!(PlayerValidator::validate_discord_id("").is_err());
        
        // Too short
        assert!(PlayerValidator::validate_discord_id("123456789").is_err());
        
        // Too long
        assert!(PlayerValidator::validate_discord_id("123456789012345678901").is_err());
        
        // Non-numeric
        assert!(PlayerValidator::validate_discord_id("12345678901234567a").is_err());
        assert!(PlayerValidator::validate_discord_id("player_123456789").is_err());
    }

    #[test]
    fn test_validate_discord_id_edge_cases() {
        // All zeros
        assert!(PlayerValidator::validate_discord_id("00000000000000000000").is_ok());
        
        // All nines
        assert!(PlayerValidator::validate_discord_id("99999999999999999999").is_ok());
        
        // Mixed numbers
        assert!(PlayerValidator::validate_discord_id("13579024681357902468").is_ok());
    }

    #[test]
    fn test_validate_discord_id_character_validation() {
        // Test individual problematic characters
        assert!(PlayerValidator::validate_discord_id("1234567890123456789a").is_err());
        assert!(PlayerValidator::validate_discord_id("1234567890123456789_").is_err());
        assert!(PlayerValidator::validate_discord_id("1234567890123456789-").is_err());
        assert!(PlayerValidator::validate_discord_id("1234567890123456789 ").is_err());
    }

    #[test]
    fn test_validate_discord_id_constants() {
        // Verify our constants are sane
        assert_eq!(MIN_DISCORD_ID_LENGTH, 10);
        assert_eq!(MAX_DISCORD_ID_LENGTH, 20);
        
        // Test boundary conditions
        let min_valid = "1".repeat(MIN_DISCORD_ID_LENGTH);
        let max_valid = "9".repeat(MAX_DISCORD_ID_LENGTH);
        let too_short = "1".repeat(MIN_DISCORD_ID_LENGTH - 1);
        let too_long = "9".repeat(MAX_DISCORD_ID_LENGTH + 1);
        
        assert!(PlayerValidator::validate_discord_id(&min_valid).is_ok());
        assert!(PlayerValidator::validate_discord_id(&max_valid).is_ok());
        assert!(PlayerValidator::validate_discord_id(&too_short).is_err());
        assert!(PlayerValidator::validate_discord_id(&too_long).is_err());
    }

    #[test]
    fn test_validate_username() {
        // Test valid usernames
        assert!(PlayerValidator::validate_username("TestPlayer#1234").is_ok());
        assert!(PlayerValidator::validate_username("DeploymentTest#0001").is_ok());
        assert!(PlayerValidator::validate_username("A").is_ok()); // Minimum length
        
        // Test invalid usernames
        assert!(PlayerValidator::validate_username("").is_err()); // Empty
        assert!(PlayerValidator::validate_username("   ").is_err()); // Only whitespace
        assert!(PlayerValidator::validate_username("A\0").is_err()); // Null byte
        assert!(PlayerValidator::validate_username("A\t").is_err()); // Tab
        assert!(PlayerValidator::validate_username("A\n").is_err()); // Newline
        assert!(PlayerValidator::validate_username("A\r").is_err()); // Carriage return
    }

    #[test]
    fn test_validate_avatar_url() {
        // Test valid URLs
        assert!(PlayerValidator::validate_avatar_url("").is_ok()); // Empty is OK
        assert!(PlayerValidator::validate_avatar_url("https://example.com/avatar.png").is_ok());
        assert!(PlayerValidator::validate_avatar_url("http://example.com/avatar.png").is_ok());
        
        // Test invalid URLs
        assert!(PlayerValidator::validate_avatar_url("ftp://example.com/avatar.png").is_err()); // Wrong protocol
        assert!(PlayerValidator::validate_avatar_url("https://example.com\0/avatar.png").is_err()); // Null byte
    }

    #[test]
    fn test_validate_complete_player_registration() {
        // Test the exact data from our failing integration test
        let result = PlayerValidator::validate_complete_player_registration(
            "10000000000000000010",
            "DeploymentTest#0001",
            None
        );
        assert!(result.is_ok(), "Complete registration validation should pass: {:?}", result);
        
        // Test with avatar URL
        let result = PlayerValidator::validate_complete_player_registration(
            "123456789012345678",
            "TestPlayer#1234",
            Some("https://example.com/avatar.png")
        );
        assert!(result.is_ok(), "Complete registration with avatar should pass: {:?}", result);
    }
}