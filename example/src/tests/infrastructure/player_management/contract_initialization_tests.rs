// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Contract initialization validation tests
//! 
//! Tests for contract setup and admin initialization validation logic.

#[cfg(test)]
mod tests {
    use crate::{
        core::validation::player_validation::PlayerValidator,
        infrastructure::errors::GameHubError,
        tests::helpers::*,
    };

    #[test]
    fn test_initialize_contract_validation_valid_admin_id() {
        // Test validation logic for valid admin Discord ID
        let valid_admin_id = VALID_DISCORD_ID;
        let result = PlayerValidator::validate_discord_id(valid_admin_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_contract_validation_invalid_admin_id() {
        // Test validation logic for invalid Discord ID format
        let invalid_admin_id = INVALID_DISCORD_ID;
        let result = PlayerValidator::validate_discord_id(invalid_admin_id);
        assert!(result.is_err());
        
        match result {
            Err(GameHubError::InvalidDiscordId { reason: _ }) => {
                // Expected error type
            },
            _ => panic!("Expected InvalidDiscordId error"),
        }
    }

    #[test] 
    fn test_initialize_contract_validation_empty_admin_id() {
        // Test validation logic for empty admin Discord ID
        let empty_admin_id = "";
        let result = PlayerValidator::validate_discord_id(empty_admin_id);
        assert!(result.is_err());
        
        match result {
            Err(GameHubError::MissingRequiredField { field }) => {
                assert_eq!(field, "discord_id");
            },
            _ => panic!("Expected MissingRequiredField error for empty discord_id"),
        }
    }

    #[test]
    fn test_initialize_contract_validation_short_admin_id() {
        // Test validation logic for short Discord ID
        let short_admin_id = "123";
        let result = PlayerValidator::validate_discord_id(short_admin_id);
        assert!(result.is_err());
        
        match result {
            Err(GameHubError::InputTooShort { field, min_length }) => {
                assert_eq!(field, "discord_id");
                assert_eq!(min_length, 10);
            },
            _ => panic!("Expected InputTooShort error for short discord_id"),
        }
    }
}