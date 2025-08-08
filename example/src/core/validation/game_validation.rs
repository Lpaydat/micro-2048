// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Game validation utilities

use crate::infrastructure::errors::GameHubError;
use regex::Regex;

/// Maximum lengths for game-related fields
pub const MAX_GAME_NAME_LENGTH: usize = 100;
pub const MIN_GAME_NAME_LENGTH: usize = 1;
pub const MAX_GAME_DESCRIPTION_LENGTH: usize = 1000;
pub const MAX_DEVELOPER_NAME_LENGTH: usize = 100;
pub const MIN_DEVELOPER_NAME_LENGTH: usize = 1;
pub const MAX_DEVELOPER_CONTACT_LENGTH: usize = 200;
pub const MIN_DEVELOPER_CONTACT_LENGTH: usize = 1;
pub const MAX_CONTRACT_ADDRESS_LENGTH: usize = 200;
pub const MIN_CONTRACT_ADDRESS_LENGTH: usize = 10;
pub const MAX_GAME_ID_LENGTH: usize = 50;
pub const MIN_GAME_ID_LENGTH: usize = 1;
pub const MAX_EVENT_ID_LENGTH: usize = 50;
pub const MIN_EVENT_ID_LENGTH: usize = 1;

/// Game validation utilities
pub struct GameValidator;

impl GameValidator {
    /// Validate game name format and constraints
    pub fn validate_game_name(name: &str) -> Result<(), GameHubError> {
        if name.is_empty() {
            return Err(GameHubError::MissingRequiredField {
                field: "game_name".to_string(),
            });
        }

        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(GameHubError::InvalidGameName {
                reason: "Game name cannot be only whitespace".to_string(),
            });
        }

        if trimmed.len() < MIN_GAME_NAME_LENGTH {
            return Err(GameHubError::InputTooShort {
                field: "game_name".to_string(),
                min_length: MIN_GAME_NAME_LENGTH,
            });
        }

        if trimmed.len() > MAX_GAME_NAME_LENGTH {
            return Err(GameHubError::InputTooLong {
                field: "game_name".to_string(),
                max_length: MAX_GAME_NAME_LENGTH,
            });
        }

        Ok(())
    }

    /// Validate game description format and constraints
    pub fn validate_game_description(description: &str) -> Result<(), GameHubError> {
        if description.len() > MAX_GAME_DESCRIPTION_LENGTH {
            return Err(GameHubError::InputTooLong {
                field: "game_description".to_string(),
                max_length: MAX_GAME_DESCRIPTION_LENGTH,
            });
        }

        // Check for null bytes
        if description.contains('\0') {
            return Err(GameHubError::InvalidCharacter {
                field: "game_description".to_string(),
                reason: "Game description cannot contain null characters".to_string(),
            });
        }

        Ok(())
    }

    /// Validate game ID format and constraints
    pub fn validate_game_id(game_id: &str) -> Result<(), GameHubError> {
        if game_id.is_empty() {
            return Err(GameHubError::MissingRequiredField {
                field: "game_id".to_string(),
            });
        }

        if game_id.len() < MIN_GAME_ID_LENGTH {
            return Err(GameHubError::InputTooShort {
                field: "game_id".to_string(),
                min_length: MIN_GAME_ID_LENGTH,
            });
        }

        if game_id.len() > MAX_GAME_ID_LENGTH {
            return Err(GameHubError::InputTooLong {
                field: "game_id".to_string(),
                max_length: MAX_GAME_ID_LENGTH,
            });
        }

        // Game ID should only contain alphanumeric characters, hyphens, and underscores
        if !game_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(GameHubError::InvalidGameId {
                reason: "Game ID can only contain alphanumeric characters, hyphens, and underscores".to_string(),
            });
        }

        Ok(())
    }

    /// Validate event ID format and constraints
    pub fn validate_event_id(event_id: &str) -> Result<(), GameHubError> {
        if event_id.is_empty() {
            return Err(GameHubError::MissingRequiredField {
                field: "event_id".to_string(),
            });
        }

        if event_id.len() < MIN_EVENT_ID_LENGTH {
            return Err(GameHubError::InputTooShort {
                field: "event_id".to_string(),
                min_length: MIN_EVENT_ID_LENGTH,
            });
        }

        if event_id.len() > MAX_EVENT_ID_LENGTH {
            return Err(GameHubError::InputTooLong {
                field: "event_id".to_string(),
                max_length: MAX_EVENT_ID_LENGTH,
            });
        }

        // Event ID should only contain alphanumeric characters, hyphens, and underscores
        if !event_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(GameHubError::InvalidEventId {
                reason: "Event ID can only contain alphanumeric characters, hyphens, and underscores".to_string(),
            });
        }

        Ok(())
    }

    /// Validate developer name format and constraints
    pub fn validate_developer_name(name: &str) -> Result<(), GameHubError> {
        if name.is_empty() {
            return Err(GameHubError::MissingRequiredField {
                field: "developer_name".to_string(),
            });
        }

        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(GameHubError::InvalidDeveloperName {
                reason: "Developer name cannot be only whitespace".to_string(),
            });
        }

        if trimmed.len() < MIN_DEVELOPER_NAME_LENGTH {
            return Err(GameHubError::InputTooShort {
                field: "developer_name".to_string(),
                min_length: MIN_DEVELOPER_NAME_LENGTH,
            });
        }

        if trimmed.len() > MAX_DEVELOPER_NAME_LENGTH {
            return Err(GameHubError::InputTooLong {
                field: "developer_name".to_string(),
                max_length: MAX_DEVELOPER_NAME_LENGTH,
            });
        }

        Ok(())
    }

    /// Validate developer contact format and constraints
    pub fn validate_developer_contact(contact: &str) -> Result<(), GameHubError> {
        if contact.is_empty() {
            return Err(GameHubError::MissingRequiredField {
                field: "developer_contact".to_string(),
            });
        }

        let trimmed = contact.trim();
        if trimmed.is_empty() {
            return Err(GameHubError::InvalidDeveloperContact {
                reason: "Developer contact cannot be only whitespace".to_string(),
            });
        }

        if trimmed.len() < MIN_DEVELOPER_CONTACT_LENGTH {
            return Err(GameHubError::InputTooShort {
                field: "developer_contact".to_string(),
                min_length: MIN_DEVELOPER_CONTACT_LENGTH,
            });
        }

        if trimmed.len() > MAX_DEVELOPER_CONTACT_LENGTH {
            return Err(GameHubError::InputTooLong {
                field: "developer_contact".to_string(),
                max_length: MAX_DEVELOPER_CONTACT_LENGTH,
            });
        }

        Ok(())
    }

    /// Validate email format if developer contact is an email
    pub fn validate_email_format(email: &str) -> Result<(), GameHubError> {
        let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        if !email_regex.is_match(email) {
            return Err(GameHubError::InvalidEmail {
                reason: "Invalid email format".to_string(),
            });
        }
        Ok(())
    }

    /// Validate contract address format and constraints
    pub fn validate_contract_address(address: &str) -> Result<(), GameHubError> {
        if address.is_empty() {
            return Err(GameHubError::MissingRequiredField {
                field: "contract_address".to_string(),
            });
        }

        if address.len() < MIN_CONTRACT_ADDRESS_LENGTH {
            return Err(GameHubError::InputTooShort {
                field: "contract_address".to_string(),
                min_length: MIN_CONTRACT_ADDRESS_LENGTH,
            });
        }

        if address.len() > MAX_CONTRACT_ADDRESS_LENGTH {
            return Err(GameHubError::InputTooLong {
                field: "contract_address".to_string(),
                max_length: MAX_CONTRACT_ADDRESS_LENGTH,
            });
        }

        // Handle different address formats
        if address.starts_with("0x") {
            // Ethereum-style hex address
            if address.len() != 42 { // 0x + 40 hex chars
                return Err(GameHubError::InvalidContractAddress);
            }
            let hex_part = &address[2..];
            if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(GameHubError::InvalidContractAddress);
            }
        } else if address.starts_with("linera") {
            // Linera-style address - allow alphanumeric and underscores
            if !address.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return Err(GameHubError::InvalidContractAddress);
            }
        } else if address.contains('.') || address.contains('-') || address.contains('_') {
            // Generic address format - allow common separators
            if !address.chars().all(|c| c.is_ascii_alphanumeric() || ".-_".contains(c)) {
                return Err(GameHubError::InvalidContractAddress);
            }
        } else {
            // Pure hexadecimal format
            if !address.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(GameHubError::InvalidContractAddressWithReason {
                    reason: "Contract address must contain only hexadecimal characters".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate complete game registration data
    pub fn validate_complete_game_registration(
        name: &str,
        description: &str,
        contract_address: &str,
        developer_name: &str,
        developer_contact: &str,
    ) -> Result<(), GameHubError> {
        Self::validate_game_name(name)?;
        Self::validate_game_description(description)?;
        Self::validate_contract_address(contract_address)?;
        Self::validate_developer_name(developer_name)?;
        Self::validate_developer_contact(developer_contact)?;
        
        // If developer contact looks like an email, validate it as such
        if developer_contact.contains('@') {
            Self::validate_email_format(developer_contact)?;
        }
        
        Ok(())
    }
}