// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Contract initialization integration tests
//! 
//! Integration tests for contract initialization validation workflows.

#![cfg(test)]

use crate::core::validation::player_validation::PlayerValidator;
use crate::infrastructure::errors::GameHubError;

#[test]
fn test_initialize_contract_validation_valid_admin_id() {
    // Test validation logic for valid admin Discord ID
    let valid_admin_id = "123456789012345678";
    let result = PlayerValidator::validate_discord_id(valid_admin_id);
    assert!(result.is_ok());
}

#[test]
fn test_initialize_contract_validation_invalid_admin_id() {
    // Test validation logic for invalid Discord ID format
    let invalid_admin_id = "invalid_discord_id";
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
        _ => panic!("Expected MissingRequiredField error"),
    }
}

#[test]
fn test_initialize_contract_validation_short_admin_id() {
    // Test validation logic for too short Discord ID
    let short_admin_id = "123";
    let result = PlayerValidator::validate_discord_id(short_admin_id);
    assert!(result.is_err());
    
    match result {
        Err(GameHubError::InputTooShort { field, min_length: _ }) => {
            assert_eq!(field, "discord_id");
        },
        _ => panic!("Expected InputTooShort error"),
    }
}

#[test]
fn test_contract_initialization_validation_flow() {
    // Test complete contract initialization validation workflow
    let test_cases = vec![
        ("123456789012345678", true),   // Valid Discord ID
        ("987654321098765432", true),   // Another valid Discord ID
        ("invalid_id", false),          // Invalid format
        ("", false),                    // Empty
        ("123", false),                 // Too short
    ];

    for (admin_id, should_pass) in test_cases {
        let result = PlayerValidator::validate_discord_id(admin_id);
        
        if should_pass {
            assert!(result.is_ok(), "Admin ID '{}' should be valid", admin_id);
        } else {
            assert!(result.is_err(), "Admin ID '{}' should be invalid", admin_id);
        }
    }
}