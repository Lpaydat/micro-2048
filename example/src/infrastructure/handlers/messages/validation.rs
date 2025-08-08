// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Cross-chain message validation utilities
//! 
//! Provides centralized validation logic for all incoming cross-chain messages
//! to ensure data integrity and prevent malicious or malformed messages.

use crate::core::types::*;
use crate::infrastructure::errors::GameHubError;
use crate::Message;
use super::{game_registration, batch_updates};

/// Centralized message validation handler
/// 
/// This validator performs comprehensive structural and content validation
/// of cross-chain messages before they are processed by specific handlers.
pub struct MessageValidator;

impl MessageValidator {
    /// Validate message structure and basic data integrity
    /// 
    /// This is the entry point for all cross-chain message validation.
    /// It performs structural validation and delegates to specific validators
    /// based on the message type.
    /// 
    /// # Arguments
    /// * `message` - The cross-chain message to validate
    /// 
    /// # Returns
    /// * `Ok(())` - Message structure is valid
    /// * `Err(GameHubError)` - Message structure or content is invalid
    pub fn validate_message_structure(message: &Message) -> Result<(), GameHubError> {
        match message {
            Message::RegisterGame { game_info } => {
                Self::validate_register_game_message(game_info)
            }
            Message::BatchEventUpdate { event_id, game_id, player_updates, final_leaderboard, .. } => {
                Self::validate_batch_update_message(event_id, game_id, player_updates, final_leaderboard)
            }
        }
    }
    
    /// Validate RegisterGame message structure
    /// 
    /// Performs comprehensive validation of game registration messages
    /// including required fields, data formats, and business rules.
    fn validate_register_game_message(game_info: &PendingGame) -> Result<(), GameHubError> {
        // Use the game registration handler's validation
        game_registration::validate_register_game_structure(game_info)?;
        
        // Additional cross-chain specific validation
        Self::validate_game_id_format(&game_info.id)?;
        Self::validate_contract_address_format(&game_info.contract_address)?;
        Self::validate_developer_contact_format(&game_info.developer_info.contact)?;
        
        Ok(())
    }
    
    /// Validate BatchEventUpdate message structure
    /// 
    /// Performs comprehensive validation of batch event update messages
    /// including data consistency, reasonable limits, and required fields.
    fn validate_batch_update_message(
        event_id: &str, 
        game_id: &str, 
        player_updates: &[PlayerEventUpdate], 
        final_leaderboard: &[LeaderboardEntry]
    ) -> Result<(), GameHubError> {
        // Use the batch updates handler's validation
        batch_updates::validate_batch_update_structure(event_id, game_id, player_updates, final_leaderboard)?;
        
        // Additional cross-chain specific validation
        Self::validate_event_id_format(event_id)?;
        Self::validate_game_id_format(game_id)?;
        Self::validate_player_updates_consistency(player_updates, final_leaderboard)?;
        
        Ok(())
    }
    
    /// Validate game ID format for cross-chain messages
    /// 
    /// Ensures game IDs follow expected patterns and don't contain
    /// problematic characters that could cause issues.
    fn validate_game_id_format(game_id: &str) -> Result<(), GameHubError> {
        // Check length constraints
        if game_id.len() < 3 || game_id.len() > 64 {
            return Err(GameHubError::InvalidInput {
                field: "game_id".to_string(),
                reason: "Game ID must be between 3 and 64 characters".to_string(),
            });
        }
        
        // Check for valid characters (alphanumeric, hyphens, underscores)
        if !game_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(GameHubError::InvalidInput {
                field: "game_id".to_string(),
                reason: "Game ID can only contain letters, numbers, hyphens, and underscores".to_string(),
            });
        }
        
        // Check that it doesn't start or end with special characters
        if game_id.starts_with('-') || game_id.starts_with('_') || 
           game_id.ends_with('-') || game_id.ends_with('_') {
            return Err(GameHubError::InvalidInput {
                field: "game_id".to_string(),
                reason: "Game ID cannot start or end with hyphens or underscores".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Validate event ID format for cross-chain messages
    fn validate_event_id_format(event_id: &str) -> Result<(), GameHubError> {
        // Similar validation to game ID but with different constraints
        if event_id.len() < 3 || event_id.len() > 128 {
            return Err(GameHubError::InvalidInput {
                field: "event_id".to_string(),
                reason: "Event ID must be between 3 and 128 characters".to_string(),
            });
        }
        
        // Allow more flexible characters for event IDs (including dots and colons for timestamps)
        if !event_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':') {
            return Err(GameHubError::InvalidInput {
                field: "event_id".to_string(),
                reason: "Event ID can only contain letters, numbers, hyphens, underscores, dots, and colons".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Validate contract address format
    /// 
    /// Ensures contract addresses are properly formatted blockchain addresses
    fn validate_contract_address_format(address: &str) -> Result<(), GameHubError> {
        // Basic hex string validation (addresses are typically 40-64 hex characters)
        if address.len() < 40 || address.len() > 66 {
            return Err(GameHubError::InvalidInput {
                field: "contract_address".to_string(),
                reason: "Contract address must be between 40 and 66 characters".to_string(),
            });
        }
        
        // Handle addresses with 0x prefix
        let address_without_prefix = if address.starts_with("0x") || address.starts_with("0X") {
            &address[2..]
        } else {
            address
        };
        
        // Validate hex characters
        if !address_without_prefix.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(GameHubError::InvalidInput {
                field: "contract_address".to_string(),
                reason: "Contract address must be a valid hexadecimal string".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Validate developer contact format
    /// 
    /// Ensures developer contact information is properly formatted
    fn validate_developer_contact_format(contact: &str) -> Result<(), GameHubError> {
        // Check for reasonable length
        if contact.len() < 5 || contact.len() > 100 {
            return Err(GameHubError::InvalidInput {
                field: "developer_contact".to_string(),
                reason: "Developer contact must be between 5 and 100 characters".to_string(),
            });
        }
        
        // Check if it looks like an email or other valid contact format
        if contact.contains("@") {
            // Basic email format validation
            let parts: Vec<&str> = contact.split("@").collect();
            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                return Err(GameHubError::InvalidInput {
                    field: "developer_contact".to_string(),
                    reason: "Developer contact email format is invalid".to_string(),
                });
            }
        }
        // Allow other formats like Discord usernames, URLs, etc.
        
        Ok(())
    }
    
    /// Validate consistency between player updates and leaderboard
    /// 
    /// Ensures that the leaderboard entries correspond to the player updates
    /// and that the data is internally consistent.
    fn validate_player_updates_consistency(
        player_updates: &[PlayerEventUpdate], 
        final_leaderboard: &[LeaderboardEntry]
    ) -> Result<(), GameHubError> {
        // Check that leaderboard entries correspond to player updates
        for leaderboard_entry in final_leaderboard {
            // Find corresponding player update
            let matching_update = player_updates.iter()
                .find(|update| update.discord_id == leaderboard_entry.player_discord_id);
            
            if matching_update.is_none() {
                return Err(GameHubError::InvalidInput {
                    field: "leaderboard_consistency".to_string(),
                    reason: format!(
                        "Leaderboard entry for player {} has no corresponding player update", 
                        leaderboard_entry.player_discord_id
                    ),
                });
            }
            
            // Validate that the scores are reasonable
            if let Some(update) = matching_update {
                if leaderboard_entry.score > update.score * 2 {
                    // Allow some flexibility for bonus calculations but flag extreme discrepancies
                    return Err(GameHubError::InvalidInput {
                        field: "score_consistency".to_string(),
                        reason: format!(
                            "Leaderboard score {} for player {} is significantly higher than update score {}",
                            leaderboard_entry.score, leaderboard_entry.player_discord_id, update.score
                        ),
                    });
                }
            }
        }
        
        // Validate leaderboard ranking consistency
        let mut sorted_leaderboard = final_leaderboard.to_vec();
        sorted_leaderboard.sort_by(|a, b| a.rank.cmp(&b.rank));
        
        for (i, entry) in sorted_leaderboard.iter().enumerate() {
            let expected_rank = (i + 1) as u32;
            if entry.rank != expected_rank {
                // Allow some flexibility for tied scores but check for major inconsistencies
                if entry.rank.abs_diff(expected_rank) > 5 {
                    return Err(GameHubError::InvalidInput {
                        field: "leaderboard_ranking".to_string(),
                        reason: format!(
                            "Leaderboard ranking is inconsistent: player {} has rank {} but should be around {}",
                            entry.player_discord_id, entry.rank, expected_rank
                        ),
                    });
                }
            }
        }
        
        Ok(())
    }
}