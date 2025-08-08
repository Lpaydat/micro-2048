// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Player validation tests
//! 
//! Tests for player data validation including Discord IDs, usernames, and avatar URLs.

#[cfg(test)]
mod tests {
    use crate::{
        core::validation::player_validation::PlayerValidator,
        infrastructure::errors::GameHubError,
        tests::helpers::*,
    };

    // Player Registration Validation Tests

    #[test]
    fn test_player_registration_validation_complete_valid() {
        // Test complete player validation with all valid data
        let discord_id = VALID_DISCORD_ID;
        let username = VALID_USERNAME;
        let avatar_url = Some(VALID_AVATAR_URL.to_string());
        
        assert!(PlayerValidator::validate_discord_id(discord_id).is_ok());
        assert!(PlayerValidator::validate_username(username).is_ok());
        if let Some(url) = &avatar_url {
            assert!(PlayerValidator::validate_avatar_url(url).is_ok());
        }
    }

    #[test]
    fn test_player_registration_validation_invalid_discord_id() {
        // Test player validation with invalid Discord ID
        let invalid_discord_id = INVALID_DISCORD_ID;
        let result = PlayerValidator::validate_discord_id(invalid_discord_id);
        assert!(result.is_err());
        
        match result {
            Err(GameHubError::InvalidDiscordId { reason: _ }) => {
                // Expected error
            },
            _ => panic!("Expected InvalidDiscordId error"),
        }
    }

    #[test]
    fn test_player_registration_validation_invalid_username() {
        // Test player validation with invalid username
        let invalid_username = ""; // Empty username
        let result = PlayerValidator::validate_username(invalid_username);
        assert!(result.is_err());
        
        match result {
            Err(GameHubError::MissingRequiredField { field }) => {
                assert_eq!(field, "username");
            },
            _ => panic!("Expected MissingRequiredField error for empty username"),
        }
    }

    #[test]
    fn test_player_registration_validation_invalid_avatar_url() {
        // Test player validation with invalid avatar URL
        let invalid_avatar_url = "not-a-url";
        let result = PlayerValidator::validate_avatar_url(invalid_avatar_url);
        assert!(result.is_err());
        
        match result {
            Err(GameHubError::InvalidUrl { reason: _ }) => {
                // Expected error for invalid URL format
            },
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    // Username Validation Tests

    #[test]
    fn test_username_validation_valid() {
        // Test various valid username formats
        let valid_usernames = vec![
            "TestPlayer",
            "User123", 
            "Player_Name",
            "Test-Player",
            "ValidUser#1234",
        ];
        
        for username in valid_usernames {
            let result = PlayerValidator::validate_username(username);
            assert!(result.is_ok(), "Username '{}' should be valid", username);
        }
    }

    #[test]
    fn test_username_validation_invalid() {
        // Test invalid username formats
        let long_username = "x".repeat(101); // MAX_USERNAME_LENGTH = 100
        let invalid_usernames = vec![
            "", // Empty
            "   ", // Only whitespace
            &long_username, // Too long
            "user\0name", // Contains null character
            "user\tname", // Contains tab
            "user\nname", // Contains newline
        ];
        
        for username in invalid_usernames {
            let result = PlayerValidator::validate_username(username);
            assert!(result.is_err(), "Username '{}' should be invalid", username);
        }
    }

    // Avatar URL Validation Tests

    #[test]
    fn test_avatar_url_validation_valid() {
        // Test valid avatar URL formats
        let valid_urls = vec![
            "https://cdn.discordapp.com/avatars/123/avatar.png",
            "https://example.com/avatar.jpg",
            "https://secure.gravatar.com/avatar/hash.png",
            "http://example.com/avatar.png", // HTTP is also valid
        ];
        
        for url in valid_urls {
            let result = PlayerValidator::validate_avatar_url(url);
            assert!(result.is_ok(), "URL '{}' should be valid", url);
        }
    }

    #[test]
    fn test_avatar_url_validation_invalid() {
        // Test invalid avatar URL formats
        let invalid_urls = vec![
            "not-a-url", // No protocol
            "ftp://file.com/avatar.png", // Wrong protocol
            "avatar\0url.png", // Contains null character
        ];
        
        for url in invalid_urls {
            let result = PlayerValidator::validate_avatar_url(url);
            assert!(result.is_err(), "URL '{}' should be invalid", url);
        }
    }

    #[test]
    fn test_avatar_url_validation_empty() {
        // Test that empty avatar URL is valid (optional field)
        let empty_url = "";
        let result = PlayerValidator::validate_avatar_url(empty_url);
        assert!(result.is_ok(), "Empty avatar URL should be valid as it's optional");
    }

    // Profile Update Validation Tests

    #[test]
    fn test_profile_update_validation_valid_username() {
        // Test profile update with valid username change
        let new_username = "UpdatedPlayer";
        let result = PlayerValidator::validate_username(new_username);
        assert!(result.is_ok());
    }

    #[test]
    fn test_profile_update_validation_invalid_username() {
        // Test profile update with invalid username change
        let invalid_username = "";
        let result = PlayerValidator::validate_username(invalid_username);
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_update_validation_valid_avatar_url() {
        // Test profile update with valid avatar URL change
        let new_avatar_url = "https://example.com/new-avatar.png";
        let result = PlayerValidator::validate_avatar_url(new_avatar_url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_profile_update_validation_invalid_avatar_url() {
        // Test profile update with invalid avatar URL change
        let invalid_avatar_url = "not-a-valid-url";
        let result = PlayerValidator::validate_avatar_url(invalid_avatar_url);
        assert!(result.is_err());
    }
}