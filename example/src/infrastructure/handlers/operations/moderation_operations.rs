// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Moderation operation handlers

use crate::infrastructure::handlers::traits::{OperationHandler, HandlerUtils};

/// Moderation-specific operations
#[derive(Debug)]
pub enum ModerationOperation {
    BanPlayer {
        caller_discord_id: String,
        player_discord_id: String,
        reason: String,
    },
    SuspendPlayer {
        caller_discord_id: String,
        player_discord_id: String,
        reason: String,
        duration_hours: Option<u32>,
    },
    UnbanPlayer {
        caller_discord_id: String,
        player_discord_id: String,
    },
    UnsuspendPlayer {
        caller_discord_id: String,
        player_discord_id: String,
    },
    AssignModerator {
        caller_discord_id: String,
        discord_id: String,
    },
    RemoveModerator {
        caller_discord_id: String,
        discord_id: String,
    },
}

/// Handler for moderation operations
pub struct ModerationOperationHandler;

impl OperationHandler for ModerationOperationHandler {
    type Operation = ModerationOperation;
    type Result = String;
    
    async fn handle<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        operation: Self::Operation
    ) -> Self::Result {
        match operation {
            ModerationOperation::BanPlayer { caller_discord_id, player_discord_id, reason } => {
                Self::ban_player(contract, caller_discord_id, player_discord_id, reason).await
            }
            ModerationOperation::SuspendPlayer { caller_discord_id, player_discord_id, reason, duration_hours } => {
                Self::suspend_player(contract, caller_discord_id, player_discord_id, reason, duration_hours).await
            }
            ModerationOperation::UnbanPlayer { caller_discord_id, player_discord_id } => {
                Self::unban_player(contract, caller_discord_id, player_discord_id).await
            }
            ModerationOperation::UnsuspendPlayer { caller_discord_id, player_discord_id } => {
                Self::unsuspend_player(contract, caller_discord_id, player_discord_id).await
            }
            ModerationOperation::AssignModerator { caller_discord_id, discord_id } => {
                Self::assign_moderator(contract, caller_discord_id, discord_id).await
            }
            ModerationOperation::RemoveModerator { caller_discord_id, discord_id } => {
                Self::remove_moderator(contract, caller_discord_id, discord_id).await
            }
        }
    }
}

impl ModerationOperationHandler {
    /// Handle player banning with permission validation and audit logging
    async fn ban_player<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        player_discord_id: String,
        reason: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().ban_player(&caller_discord_id, &player_discord_id, &reason, timestamp).await {
            Ok(()) => {
                format!("Player {} has been banned: {}", player_discord_id, reason)
            }
            Err(error) => {
                HandlerUtils::error_response("banning player", &error.to_string())
            }
        }
    }
    
    /// Handle player suspension with permission validation and audit logging
    async fn suspend_player<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        player_discord_id: String,
        reason: String,
        duration_hours: Option<u32>
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().suspend_player(&caller_discord_id, &player_discord_id, &reason, duration_hours, timestamp).await {
            Ok(()) => {
                match duration_hours {
                    Some(hours) => format!("Player {} suspended for {} hours: {}", player_discord_id, hours, reason),
                    None => format!("Player {} suspended indefinitely: {}", player_discord_id, reason),
                }
            }
            Err(error) => {
                HandlerUtils::error_response("suspending player", &error.to_string())
            }
        }
    }
    
    /// Handle player unbanning with permission validation and audit logging
    async fn unban_player<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        player_discord_id: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().unban_player(&caller_discord_id, &player_discord_id, timestamp).await {
            Ok(()) => {
                format!("Player {} has been unbanned and is now active", player_discord_id)
            }
            Err(error) => {
                HandlerUtils::error_response("unbanning player", &error.to_string())
            }
        }
    }
    
    /// Handle player unsuspension with permission validation and audit logging
    async fn unsuspend_player<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        player_discord_id: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().unsuspend_player(&caller_discord_id, &player_discord_id, timestamp).await {
            Ok(()) => {
                format!("Player {} has been unsuspended and is now active", player_discord_id)
            }
            Err(error) => {
                HandlerUtils::error_response("unsuspending player", &error.to_string())
            }
        }
    }
    
    /// Handle moderator assignment with permission validation and audit logging
    async fn assign_moderator<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        discord_id: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().assign_moderator(&caller_discord_id, &discord_id, timestamp).await {
            Ok(()) => {
                HandlerUtils::success_response("assigned", &format!("user {} as moderator", discord_id), None)
            }
            Err(error) => {
                HandlerUtils::error_response("assigning moderator", &error.to_string())
            }
        }
    }
    
    /// Handle moderator removal with permission validation and audit logging
    async fn remove_moderator<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        discord_id: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().remove_moderator(&caller_discord_id, &discord_id, timestamp).await {
            Ok(()) => {
                HandlerUtils::success_response("removed", &format!("user {} from moderator", discord_id), None)
            }
            Err(error) => {
                HandlerUtils::error_response("removing moderator", &error.to_string())
            }
        }
    }
}

