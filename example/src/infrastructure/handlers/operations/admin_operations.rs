// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Admin operation handlers

use crate::{
    core::types::EventType,
    infrastructure::handlers::traits::{OperationHandler, HandlerUtils}
};

/// Admin-specific operations
#[derive(Debug)]
pub enum AdminOperation {
    ApproveGame {
        caller_discord_id: String,
        game_id: String,
    },
    RejectGame {
        caller_discord_id: String,
        game_id: String,
        reason: String,
    },
    AddAdmin {
        caller_discord_id: String,
        discord_id: String,
    },
    RemoveAdmin {
        caller_discord_id: String,
        discord_id: String,
    },
}

/// Handler for admin operations
pub struct AdminOperationHandler;

impl OperationHandler for AdminOperationHandler {
    type Operation = AdminOperation;
    type Result = String;
    
    async fn handle<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        operation: Self::Operation
    ) -> Self::Result {
        match operation {
            AdminOperation::ApproveGame { caller_discord_id, game_id } => {
                Self::approve_game(contract, caller_discord_id, game_id).await
            }
            AdminOperation::RejectGame { caller_discord_id, game_id, reason } => {
                Self::reject_game(contract, caller_discord_id, game_id, reason).await
            }
            AdminOperation::AddAdmin { caller_discord_id, discord_id } => {
                Self::add_admin(contract, caller_discord_id, discord_id).await
            }
            AdminOperation::RemoveAdmin { caller_discord_id, discord_id } => {
                Self::remove_admin(contract, caller_discord_id, discord_id).await
            }
        }
    }
}

impl AdminOperationHandler {
    /// Handle game approval with permission validation and audit logging
    async fn approve_game<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        game_id: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().approve_game(&caller_discord_id, &game_id, timestamp).await {
            Ok(approved_game) => {
                // Log the approval event
                HandlerUtils::log_event(
                    contract,
                    EventType::GameApproved,
                    format!("Game '{}' approved by admin", approved_game.name),
                    Some(caller_discord_id),
                    Some(game_id),
                );
                
                format!("Game '{}' approved and added to active games.", approved_game.name)
            }
            Err(error) => {
                HandlerUtils::error_response("approving game", &error.to_string())
            }
        }
    }
    
    /// Handle game rejection with permission validation and audit logging  
    async fn reject_game<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        game_id: String,
        reason: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().reject_game(&caller_discord_id, &game_id, &reason, timestamp).await {
            Ok(()) => {
                format!("Game '{}' rejected and removed from pending games. Reason: {}", game_id, reason)
            }
            Err(error) => {
                HandlerUtils::error_response("rejecting game", &error.to_string())
            }
        }
    }
    
    /// Handle admin addition with permission validation and audit logging
    async fn add_admin<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        discord_id: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().add_admin(&caller_discord_id, &discord_id, timestamp).await {
            Ok(()) => {
                HandlerUtils::success_response("added", &format!("user {} as admin", discord_id), None)
            }
            Err(error) => {
                HandlerUtils::error_response("adding admin", &error.to_string())
            }
        }
    }
    
    /// Handle admin removal with permission validation and audit logging
    async fn remove_admin<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        discord_id: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().remove_admin(&caller_discord_id, &discord_id, timestamp).await {
            Ok(()) => {
                HandlerUtils::success_response("removed", &format!("user {} from admin", discord_id), None)
            }
            Err(error) => {
                HandlerUtils::error_response("removing admin", &error.to_string())
            }
        }
    }
}

