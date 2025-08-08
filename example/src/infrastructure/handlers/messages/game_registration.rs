// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Cross-chain game registration message handler
//! 
//! Handles RegisterGame messages from external game contracts
//! seeking approval to join the GameHub ecosystem.

use crate::core::types::*;
use crate::infrastructure::{state::GameHubState, errors::GameHubError};
use linera_sdk::linera_base_types::Timestamp;

/// Handle RegisterGame cross-chain message
/// 
/// This function processes game registration requests sent from external
/// game contracts via cross-chain messaging. It validates the game data,
/// adds it to the pending games queue for admin approval, and returns
/// a status message.
/// 
/// # Arguments
/// * `state` - Mutable reference to the GameHub blockchain state
/// * `game_info` - Game information from the registration message
/// * `timestamp` - Blockchain timestamp when message was received
/// 
/// # Returns
/// * `Ok(String)` - Success message with game registration details
/// * `Err(GameHubError)` - Error if validation fails or database operation fails
pub async fn handle_register_game_message(
    state: &mut GameHubState,
    game_info: PendingGame,
    timestamp: Timestamp,
) -> Result<String, GameHubError> {
    // Validate the incoming message data
    state.validate_game_registration_request(&game_info).await?;
    
    // Add to pending games queue (uses existing state method)
    // This method already includes validation and error handling
    state.add_pending_game(game_info.clone()).await?;
    
    // Create success message for logging
    let success_message = format!(
        "Cross-chain game registration received and queued for approval: '{}' (ID: {}) from developer '{}'", 
        game_info.name, 
        game_info.id,
        game_info.developer_info.name
    );
    
    Ok(success_message)
}

/// Validate game registration message structure
/// 
/// Performs basic structural validation of the RegisterGame message
/// before passing it to the state validation methods.
pub fn validate_register_game_structure(game_info: &PendingGame) -> Result<(), GameHubError> {
    // Basic non-empty field validation
    if game_info.id.is_empty() {
        return Err(GameHubError::InvalidInput { 
            field: "game_id".to_string(), 
            reason: "Game ID cannot be empty in cross-chain registration".to_string() 
        });
    }
    
    if game_info.name.is_empty() {
        return Err(GameHubError::InvalidInput { 
            field: "game_name".to_string(), 
            reason: "Game name cannot be empty in cross-chain registration".to_string() 
        });
    }
    
    if game_info.contract_address.is_empty() {
        return Err(GameHubError::InvalidInput { 
            field: "contract_address".to_string(), 
            reason: "Contract address cannot be empty in cross-chain registration".to_string() 
        });
    }
    
    // Validate developer info structure
    if game_info.developer_info.name.is_empty() {
        return Err(GameHubError::InvalidInput { 
            field: "developer_name".to_string(), 
            reason: "Developer name is required for cross-chain registration".to_string() 
        });
    }
    
    if game_info.developer_info.contact.is_empty() {
        return Err(GameHubError::InvalidInput { 
            field: "developer_contact".to_string(), 
            reason: "Developer contact is required for cross-chain registration".to_string() 
        });
    }
    
    Ok(())
}