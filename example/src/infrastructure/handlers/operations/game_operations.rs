// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Game operation handlers

use crate::infrastructure::handlers::traits::{OperationHandler, HandlerUtils};

/// Game-specific operations
#[derive(Debug)]
pub enum GameOperation {
    SuspendGame {
        caller_discord_id: String,
        game_id: String,
        reason: String,
    },
    ReactivateGame {
        caller_discord_id: String,
        game_id: String,
    },
    DeprecateGame {
        caller_discord_id: String,
        game_id: String,
    },
}

/// Handler for game operations
pub struct GameOperationHandler;

impl OperationHandler for GameOperationHandler {
    type Operation = GameOperation;
    type Result = String;
    
    async fn handle<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        operation: Self::Operation
    ) -> Self::Result {
        match operation {
            GameOperation::SuspendGame { caller_discord_id, game_id, reason } => {
                Self::suspend_game(contract, caller_discord_id, game_id, reason).await
            }
            GameOperation::ReactivateGame { caller_discord_id, game_id } => {
                Self::reactivate_game(contract, caller_discord_id, game_id).await
            }
            GameOperation::DeprecateGame { caller_discord_id, game_id } => {
                Self::deprecate_game(contract, caller_discord_id, game_id).await
            }
        }
    }
}

impl GameOperationHandler {
    /// Handle game suspension with permission validation and audit logging
    async fn suspend_game<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        game_id: String,
        reason: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().suspend_game(&caller_discord_id, &game_id, &reason, timestamp).await {
            Ok(()) => {
                format!("Game {} suspended: {}", game_id, reason)
            }
            Err(error) => {
                HandlerUtils::error_response("suspending game", &error.to_string())
            }
        }
    }
    
    /// Handle game reactivation with permission validation and audit logging
    async fn reactivate_game<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        game_id: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().reactivate_game(&caller_discord_id, &game_id, timestamp).await {
            Ok(()) => {
                HandlerUtils::success_response("reactivated", &format!("game {}", game_id), None)
            }
            Err(error) => {
                HandlerUtils::error_response("reactivating game", &error.to_string())
            }
        }
    }
    
    /// Handle game deprecation with permission validation and audit logging
    async fn deprecate_game<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        game_id: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().deprecate_game(&caller_discord_id, &game_id, timestamp).await {
            Ok(()) => {
                HandlerUtils::success_response("deprecated", &format!("game {}", game_id), None)
            }
            Err(error) => {
                HandlerUtils::error_response("deprecating game", &error.to_string())
            }
        }
    }
}

