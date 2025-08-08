// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Moderation-related GraphQL mutations

use std::sync::Arc;
use async_graphql::{Object, Result};
use crate::{infrastructure::operations::Operation, ScheduleOperation};

/// Moderation mutation resolvers
#[derive(Clone)]
pub struct ModerationMutations {
    pub runtime: Arc<dyn ScheduleOperation>,
}

#[Object]
impl ModerationMutations {
    /// Ban a player (admin only)
    async fn ban_player(
        &self,
        admin_discord_id: String,
        player_discord_id: String,
        reason: String
    ) -> Result<String> {
        // Validate Discord ID formats
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid admin Discord ID: {}", e)))?;
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&player_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid player Discord ID: {}", e)))?;
        
        // Validate ban reason
        let trimmed_reason = reason.trim();
        if trimmed_reason.is_empty() {
            return Err(async_graphql::Error::new("Ban reason cannot be empty"));
        }
        if trimmed_reason.len() > 500 {
            return Err(async_graphql::Error::new("Ban reason too long (maximum 500 characters)"));
        }
        
        // Prevent self-ban
        if admin_discord_id == player_discord_id {
            return Err(async_graphql::Error::new("Cannot ban yourself"));
        }
        
        // Schedule ban operation
        self.runtime.schedule_operation(&Operation::BanPlayer {
            caller_discord_id: admin_discord_id,
            player_discord_id: player_discord_id.clone(),
            reason: trimmed_reason.to_string(),
        });
        
        Ok(format!("Player ban scheduled for Discord ID: {}", player_discord_id))
    }

    /// Suspend a player (moderator)
    async fn suspend_player(
        &self,
        admin_discord_id: String,
        player_discord_id: String,
        reason: String,
        duration_hours: Option<i32>
    ) -> Result<String> {
        // Validate Discord ID formats
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid admin Discord ID: {}", e)))?;
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&player_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid player Discord ID: {}", e)))?;
        
        // Validate suspension reason
        let trimmed_reason = reason.trim();
        if trimmed_reason.is_empty() {
            return Err(async_graphql::Error::new("Suspension reason cannot be empty"));
        }
        if trimmed_reason.len() > 500 {
            return Err(async_graphql::Error::new("Suspension reason too long (maximum 500 characters)"));
        }
        
        // Validate duration (if provided)
        if let Some(hours) = duration_hours {
            if hours <= 0 {
                return Err(async_graphql::Error::new("Suspension duration must be positive"));
            }
            if hours > 8760 { // 1 year max
                return Err(async_graphql::Error::new("Suspension duration too long (maximum 1 year)"));
            }
        }
        
        // Prevent self-suspension
        if admin_discord_id == player_discord_id {
            return Err(async_graphql::Error::new("Cannot suspend yourself"));
        }
        
        // Schedule suspension operation
        self.runtime.schedule_operation(&Operation::SuspendPlayer {
            caller_discord_id: admin_discord_id,
            player_discord_id: player_discord_id.clone(),
            reason: trimmed_reason.to_string(),
            duration_hours: duration_hours.map(|h| h as u32),
        });
        
        Ok(format!("Player suspension scheduled for Discord ID: {} ({} hours)", 
            player_discord_id, 
            duration_hours.map(|h| h.to_string()).unwrap_or_else(|| "indefinite".to_string())
        ))
    }

    /// Unban a player (admin only)
    async fn unban_player(
        &self,
        admin_discord_id: String,
        player_discord_id: String
    ) -> Result<String> {
        // Validate Discord ID formats
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid admin Discord ID: {}", e)))?;
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&player_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid player Discord ID: {}", e)))?;
        
        // Prevent self-unban (though this shouldn't be possible if you're banned)
        if admin_discord_id == player_discord_id {
            return Err(async_graphql::Error::new("Cannot unban yourself"));
        }
        
        // Schedule unban operation
        self.runtime.schedule_operation(&Operation::UnbanPlayer {
            caller_discord_id: admin_discord_id,
            player_discord_id: player_discord_id.clone(),
        });
        
        Ok(format!("Player unban scheduled for Discord ID: {}", player_discord_id))
    }

    /// Unsuspend a player (admin/moderator)
    async fn unsuspend_player(
        &self,
        admin_discord_id: String,
        player_discord_id: String
    ) -> Result<String> {
        // Validate Discord ID formats
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid admin Discord ID: {}", e)))?;
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&player_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid player Discord ID: {}", e)))?;
        
        // Prevent self-unsuspend (though this shouldn't be possible if you're suspended)
        if admin_discord_id == player_discord_id {
            return Err(async_graphql::Error::new("Cannot unsuspend yourself"));
        }
        
        // Schedule unsuspend operation
        self.runtime.schedule_operation(&Operation::UnsuspendPlayer {
            caller_discord_id: admin_discord_id,
            player_discord_id: player_discord_id.clone(),
        });
        
        Ok(format!("Player unsuspension scheduled for Discord ID: {}", player_discord_id))
    }
}