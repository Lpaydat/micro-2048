// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Player operation handlers

use crate::{
    core::types::EventType,
    infrastructure::handlers::traits::{OperationHandler, HandlerUtils}
};

/// Player-specific operations
#[derive(Debug)]
pub enum PlayerOperation {
    RegisterPlayer {
        discord_id: String,
        username: String,
        avatar_url: Option<String>,
    },
    UpdatePlayerProfile {
        discord_id: String,
        username: Option<String>,
        avatar_url: Option<String>,
    },
}

/// Handler for player operations
pub struct PlayerOperationHandler;

impl OperationHandler for PlayerOperationHandler {
    type Operation = PlayerOperation;
    type Result = String;
    
    async fn handle<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        operation: Self::Operation
    ) -> Self::Result {
        match operation {
            PlayerOperation::RegisterPlayer { discord_id, username, avatar_url } => {
                Self::register_player(contract, discord_id, username, avatar_url).await
            }
            PlayerOperation::UpdatePlayerProfile { discord_id, username, avatar_url } => {
                Self::update_player_profile(contract, discord_id, username, avatar_url).await
            }
        }
    }
}

impl PlayerOperationHandler {
    /// Handle player registration with pending data merging
    async fn register_player<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        discord_id: String,
        username: String,
        avatar_url: Option<String>
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().register_or_update_player(&discord_id, &username, avatar_url, timestamp).await {
            Ok(player) => {
                // Log the registration event
                HandlerUtils::log_event(
                    contract,
                    EventType::PlayerRegistered,
                    format!("Player {} registered with {} total points", username, player.total_points),
                    Some(discord_id.clone()),
                    Some(discord_id.clone()),
                );
                
                // Return appropriate success message
                if player.total_points > 0 {
                    format!(
                        "Player {} registered successfully! Merged {} pending points from previous activity.",
                        username, 
                        player.total_points
                    )
                } else {
                    format!("Player {} registered successfully!", username)
                }
            }
            Err(error) => {
                HandlerUtils::error_response("registering player", &error.to_string())
            }
        }
    }
    
    /// Handle player profile updates
    async fn update_player_profile<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        discord_id: String,
        username: Option<String>,
        avatar_url: Option<String>
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().update_player_profile(&discord_id, username, avatar_url, timestamp).await {
            Ok(()) => {
                HandlerUtils::success_response("updated profile for player", &discord_id, None)
            }
            Err(error) => {
                HandlerUtils::error_response("updating player profile", &error.to_string())
            }
        }
    }
}

