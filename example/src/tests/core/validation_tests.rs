// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive validation tests for GameHub error handling and validation

#[cfg(test)]
mod tests {
    use crate::{
        core::validation::*,
        infrastructure::errors::GameHubError,
    };

    #[test]
    fn test_comprehensive_discord_id_validation() {
        // Valid Discord IDs
        assert!(PlayerValidator::validate_discord_id("1234567890123456789").is_ok());
        assert!(PlayerValidator::validate_discord_id("1000000000000000000").is_ok());
        
        // Invalid Discord IDs - empty
        match PlayerValidator::validate_discord_id("") {
            Err(GameHubError::MissingRequiredField { field }) => {
                assert_eq!(field, "discord_id");
            }
            _ => panic!("Expected MissingRequiredField error"),
        }
        
        // Invalid Discord IDs - too short
        match PlayerValidator::validate_discord_id("123456789") {
            Err(GameHubError::InputTooShort { field, min_length }) => {
                assert_eq!(field, "discord_id");
                assert_eq!(min_length, 10);
            }
            _ => panic!("Expected InputTooShort error"),
        }
        
        // Invalid Discord IDs - too long
        match PlayerValidator::validate_discord_id("123456789012345678901") {
            Err(GameHubError::InputTooLong { field, max_length }) => {
                assert_eq!(field, "discord_id");
                assert_eq!(max_length, 20);
            }
            _ => panic!("Expected InputTooLong error"),
        }
        
        // Invalid Discord IDs - non-numeric
        match PlayerValidator::validate_discord_id("12345abc67890") {
            Err(GameHubError::InvalidDiscordId { reason }) => {
                assert!(reason.contains("numeric"));
            }
            _ => panic!("Expected InvalidDiscordId error"),
        }
    }

    #[test]
    fn test_comprehensive_username_validation() {
        // Valid usernames
        assert!(PlayerValidator::validate_username("TestUser").is_ok());
        assert!(PlayerValidator::validate_username("User123").is_ok());
        assert!(PlayerValidator::validate_username("A").is_ok()); // minimum length
        
        // Invalid usernames - empty
        match PlayerValidator::validate_username("") {
            Err(GameHubError::MissingRequiredField { field }) => {
                assert_eq!(field, "username");
            }
            _ => panic!("Expected MissingRequiredField error"),
        }
        
        // Invalid usernames - only whitespace
        match PlayerValidator::validate_username("   ") {
            Err(GameHubError::InvalidUsername { reason }) => {
                assert!(reason.contains("whitespace"));
            }
            _ => panic!("Expected InvalidUsername error"),
        }
        
        // Invalid usernames - too long
        let long_username = "x".repeat(101);
        match PlayerValidator::validate_username(&long_username) {
            Err(GameHubError::InputTooLong { field, max_length }) => {
                assert_eq!(field, "username");
                assert_eq!(max_length, 100);
            }
            _ => panic!("Expected InputTooLong error"),
        }
        
        // Invalid usernames - null bytes
        match PlayerValidator::validate_username("User\0Name") {
            Err(GameHubError::InvalidCharacter { field, reason }) => {
                assert_eq!(field, "username");
                assert!(reason.contains("null"));
            }
            _ => panic!("Expected InvalidCharacter error"),
        }
    }

    #[test]
    fn test_comprehensive_contract_address_validation() {
        // Valid Ethereum addresses
        assert!(GameValidator::validate_contract_address("0x1234567890abcdef1234567890abcdef12345678").is_ok());
        assert!(GameValidator::validate_contract_address("0xABCDEF1234567890abcdef1234567890ABCDEF12").is_ok());
        
        // Valid Linera addresses
        assert!(GameValidator::validate_contract_address("linera1abcdef1234567890").is_ok());
        assert!(GameValidator::validate_contract_address("linera_contract_12345678901234567890").is_ok());
        
        // Valid generic addresses
        assert!(GameValidator::validate_contract_address("contract_12345").is_ok());
        assert!(GameValidator::validate_contract_address("my-contract.address").is_ok());
        
        // Invalid addresses - empty
        match GameValidator::validate_contract_address("") {
            Err(GameHubError::MissingRequiredField { field }) => {
                assert_eq!(field, "contract_address");
            }
            _ => panic!("Expected MissingRequiredField error"),
        }
        
        // Invalid addresses - too short
        match GameValidator::validate_contract_address("short") {
            Err(GameHubError::InputTooShort { field, min_length }) => {
                assert_eq!(field, "contract_address");
                assert_eq!(min_length, 10);
            }
            _ => panic!("Expected InputTooShort error"),
        }
        
        // Invalid addresses - wrong length for Ethereum (too long for the minimum check)
        match GameValidator::validate_contract_address("0x1234567890abcdef") {
            Err(GameHubError::InvalidContractAddress) => {}
            result => panic!("Expected InvalidContractAddress error, got: {:?}", result),
        }
        
        // Invalid addresses - non-hex for Ethereum
        match GameValidator::validate_contract_address("0x1234567890abcdef1234567890abcdef1234567G") {
            Err(GameHubError::InvalidContractAddress) => {}
            _ => panic!("Expected InvalidContractAddress error"),
        }
    }

    #[test]
    fn test_comprehensive_game_validation() {
        // Valid game names
        assert!(GameValidator::validate_game_name("Test Game").is_ok());
        assert!(GameValidator::validate_game_name("A").is_ok()); // minimum length
        assert!(GameValidator::validate_game_name(&"x".repeat(100)).is_ok()); // maximum length
        
        // Invalid game names - empty
        match GameValidator::validate_game_name("") {
            Err(GameHubError::MissingRequiredField { field }) => {
                assert_eq!(field, "game_name");
            }
            _ => panic!("Expected MissingRequiredField error"),
        }
        
        // Invalid game names - only whitespace
        match GameValidator::validate_game_name("   ") {
            Err(GameHubError::InvalidGameName { reason }) => {
                assert!(reason.contains("whitespace"));
            }
            _ => panic!("Expected InvalidGameName error"),
        }
        
        // Invalid game names - too long
        let long_name = "x".repeat(101);
        match GameValidator::validate_game_name(&long_name) {
            Err(GameHubError::InputTooLong { field, max_length }) => {
                assert_eq!(field, "game_name");
                assert_eq!(max_length, 100);
            }
            _ => panic!("Expected InputTooLong error"),
        }
        
        // Valid game descriptions
        assert!(GameValidator::validate_game_description("").is_ok()); // empty is ok
        assert!(GameValidator::validate_game_description("A great game!").is_ok());
        assert!(GameValidator::validate_game_description(&"x".repeat(1000)).is_ok()); // maximum length
        
        // Invalid game descriptions - too long
        let long_description = "x".repeat(1001);
        match GameValidator::validate_game_description(&long_description) {
            Err(GameHubError::InputTooLong { field, max_length }) => {
                assert_eq!(field, "game_description");
                assert_eq!(max_length, 1000);
            }
            _ => panic!("Expected InputTooLong error"),
        }
        
        // Invalid game descriptions - null bytes
        match GameValidator::validate_game_description("Game with\0null byte") {
            Err(GameHubError::InvalidCharacter { field, reason }) => {
                assert_eq!(field, "game_description");
                assert!(reason.contains("null"));
            }
            _ => panic!("Expected InvalidCharacter error"),
        }
    }

    #[test]
    fn test_email_validation() {
        // Valid emails
        assert!(GameValidator::validate_email_format("test@example.com").is_ok());
        assert!(GameValidator::validate_email_format("user.name+tag@domain.co.uk").is_ok());
        assert!(GameValidator::validate_email_format("user123@test-domain.org").is_ok());
        
        // Invalid emails
        match GameValidator::validate_email_format("invalid-email") {
            Err(GameHubError::InvalidEmail { reason }) => {
                assert!(reason.contains("Invalid email"));
            }
            _ => panic!("Expected InvalidEmail error"),
        }
        
        match GameValidator::validate_email_format("@domain.com") {
            Err(GameHubError::InvalidEmail { .. }) => {}
            _ => panic!("Expected InvalidEmail error"),
        }
        
        match GameValidator::validate_email_format("user@") {
            Err(GameHubError::InvalidEmail { .. }) => {}
            _ => panic!("Expected InvalidEmail error"),
        }
    }

    #[test]
    fn test_score_validation() {
        // Valid scores
        assert!(PlayerValidator::validate_score(0).is_ok());
        assert!(PlayerValidator::validate_score(1000).is_ok());
        assert!(PlayerValidator::validate_score(1_000_000_000).is_ok()); // max value
        
        // Invalid scores - exceeds maximum
        match PlayerValidator::validate_score(1_000_000_001) {
            Err(GameHubError::InvalidScore { reason }) => {
                assert!(reason.contains("exceed"));
            }
            _ => panic!("Expected InvalidScore error"),
        }
    }

    #[test]
    fn test_avatar_url_validation() {
        // Valid URLs
        assert!(PlayerValidator::validate_avatar_url("").is_ok()); // empty is ok
        assert!(PlayerValidator::validate_avatar_url("https://example.com/avatar.png").is_ok());
        assert!(PlayerValidator::validate_avatar_url("http://test.com/img.jpg").is_ok());
        
        // Invalid URLs - wrong protocol
        match PlayerValidator::validate_avatar_url("ftp://example.com/avatar.png") {
            Err(GameHubError::InvalidUrl { reason }) => {
                assert!(reason.contains("http"));
            }
            _ => panic!("Expected InvalidUrl error"),
        }
        
        // Invalid URLs - too long
        let long_url = format!("https://example.com/{}", "x".repeat(500));
        match PlayerValidator::validate_avatar_url(&long_url) {
            Err(GameHubError::InputTooLong { field, max_length }) => {
                assert_eq!(field, "avatar_url");
                assert_eq!(max_length, 500);
            }
            _ => panic!("Expected InputTooLong error"),
        }
        
        // Invalid URLs - invalid characters
        match PlayerValidator::validate_avatar_url("https://example.com/avatar\0.png") {
            Err(GameHubError::InvalidCharacter { field, reason }) => {
                assert_eq!(field, "avatar_url");
                assert!(reason.contains("null"));
            }
            _ => panic!("Expected InvalidCharacter error"),
        }
    }

    #[test]
    fn test_sanitization() {
        // Text sanitization
        assert_eq!(GeneralValidator::sanitize_text_input("  test\0input\r  "), "testinput");
        assert_eq!(GeneralValidator::sanitize_text_input("normal text"), "normal text");
        assert_eq!(GeneralValidator::sanitize_text_input("\r\n\0"), "");
        
        // HTML sanitization
        assert_eq!(
            GeneralValidator::sanitize_html_content("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
        assert_eq!(
            GeneralValidator::sanitize_html_content("Safe & \"quoted\" text"),
            "Safe &amp; &quot;quoted&quot; text"
        );
    }

    #[test]
    fn test_batch_validation() {
        // Valid batch sizes
        assert!(ScoringValidator::validate_batch_size(1, "test").is_ok());
        assert!(ScoringValidator::validate_batch_size(100, "test").is_ok());
        assert!(ScoringValidator::validate_batch_size(1000, "test").is_ok()); // max size
        
        // Invalid batch sizes - empty
        match ScoringValidator::validate_batch_size(0, "test") {
            Err(GameHubError::InvalidBatchSize { operation, reason }) => {
                assert_eq!(operation, "test");
                assert!(reason.contains("greater than 0"));
            }
            _ => panic!("Expected InvalidBatchSize error"),
        }
        
        // Invalid batch sizes - too large
        match ScoringValidator::validate_batch_size(1001, "test") {
            Err(GameHubError::InvalidBatchSize { operation, reason }) => {
                assert_eq!(operation, "test");
                assert!(reason.contains("exceed"));
            }
            _ => panic!("Expected InvalidBatchSize error"),
        }
    }
}