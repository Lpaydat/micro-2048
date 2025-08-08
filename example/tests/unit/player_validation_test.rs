//! Unit tests for player validation
#![cfg(test)]

use super::PlayerValidator;
use crate::infrastructure::errors::GameHubError;

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
    use super::{MIN_DISCORD_ID_LENGTH, MAX_DISCORD_ID_LENGTH};
    
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