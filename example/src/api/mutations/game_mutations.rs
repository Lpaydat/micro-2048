// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Game-related GraphQL mutations

use std::sync::Arc;
use async_graphql::{Object, Result};
use crate::{infrastructure::operations::Operation, ScheduleOperation};

/// Game mutation resolvers
#[derive(Clone)]
pub struct GameMutations {
    pub runtime: Arc<dyn ScheduleOperation>,
}

#[Object]
impl GameMutations {
    /// Approve a pending game (admin only)
    async fn approve_game(&self, admin_discord_id: String, game_id: String) -> Result<String> {
        // Validate admin Discord ID format
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid admin Discord ID: {}", e)))?;
        
        // Validate game ID format
        let trimmed_game_id = game_id.trim();
        if trimmed_game_id.is_empty() {
            return Err(async_graphql::Error::new("Game ID cannot be empty"));
        }
        if trimmed_game_id.len() > 100 {
            return Err(async_graphql::Error::new("Game ID too long (maximum 100 characters)"));
        }
        
        // Schedule game approval operation
        self.runtime.schedule_operation(&Operation::ApproveGame {
            caller_discord_id: admin_discord_id,
            game_id: trimmed_game_id.to_string(),
        });
        
        Ok(format!("Game approval scheduled for game ID: {}", trimmed_game_id))
    }

    /// Reject a pending game (admin only)
    async fn reject_game(&self, admin_discord_id: String, game_id: String, reason: String) -> Result<String> {
        // Validate admin Discord ID format
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid admin Discord ID: {}", e)))?;
        
        // Validate game ID format
        let trimmed_game_id = game_id.trim();
        if trimmed_game_id.is_empty() {
            return Err(async_graphql::Error::new("Game ID cannot be empty"));
        }
        if trimmed_game_id.len() > 100 {
            return Err(async_graphql::Error::new("Game ID too long (maximum 100 characters)"));
        }
        
        // Validate rejection reason
        let trimmed_reason = reason.trim();
        if trimmed_reason.is_empty() {
            return Err(async_graphql::Error::new("Rejection reason cannot be empty"));
        }
        if trimmed_reason.len() > 1000 {
            return Err(async_graphql::Error::new("Rejection reason too long (maximum 1000 characters)"));
        }
        
        // Schedule game rejection operation
        self.runtime.schedule_operation(&Operation::RejectGame {
            caller_discord_id: admin_discord_id,
            game_id: trimmed_game_id.to_string(),
            reason: trimmed_reason.to_string(),
        });
        
        Ok(format!("Game rejection scheduled for game ID: {}", trimmed_game_id))
    }

    /// Suspend a game (admin only)
    async fn suspend_game(&self, admin_discord_id: String, game_id: String, reason: String) -> Result<String> {
        // Validate admin Discord ID format
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid admin Discord ID: {}", e)))?;
        
        // Validate game ID format
        let trimmed_game_id = game_id.trim();
        if trimmed_game_id.is_empty() {
            return Err(async_graphql::Error::new("Game ID cannot be empty"));
        }
        if trimmed_game_id.len() > 100 {
            return Err(async_graphql::Error::new("Game ID too long (maximum 100 characters)"));
        }
        
        // Validate suspension reason
        let trimmed_reason = reason.trim();
        if trimmed_reason.is_empty() {
            return Err(async_graphql::Error::new("Suspension reason cannot be empty"));
        }
        if trimmed_reason.len() > 1000 {
            return Err(async_graphql::Error::new("Suspension reason too long (maximum 1000 characters)"));
        }
        
        // Schedule game suspension operation
        self.runtime.schedule_operation(&Operation::SuspendGame {
            caller_discord_id: admin_discord_id,
            game_id: trimmed_game_id.to_string(),
            reason: trimmed_reason.to_string(),
        });
        
        Ok(format!("Game suspension scheduled for game ID: {}", trimmed_game_id))
    }

    /// Reactivate a suspended game (admin only)
    async fn reactivate_game(&self, admin_discord_id: String, game_id: String) -> Result<String> {
        // Validate admin Discord ID format
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid admin Discord ID: {}", e)))?;
        
        // Validate game ID format
        let trimmed_game_id = game_id.trim();
        if trimmed_game_id.is_empty() {
            return Err(async_graphql::Error::new("Game ID cannot be empty"));
        }
        if trimmed_game_id.len() > 100 {
            return Err(async_graphql::Error::new("Game ID too long (maximum 100 characters)"));
        }
        
        // Schedule game reactivation operation
        self.runtime.schedule_operation(&Operation::ReactivateGame {
            caller_discord_id: admin_discord_id,
            game_id: trimmed_game_id.to_string(),
        });
        
        Ok(format!("Game reactivation scheduled for game ID: {}", trimmed_game_id))
    }

    /// Deprecate a game (admin only)
    async fn deprecate_game(&self, admin_discord_id: String, game_id: String) -> Result<String> {
        // Validate admin Discord ID format
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid admin Discord ID: {}", e)))?;
        
        // Validate game ID format
        let trimmed_game_id = game_id.trim();
        if trimmed_game_id.is_empty() {
            return Err(async_graphql::Error::new("Game ID cannot be empty"));
        }
        if trimmed_game_id.len() > 100 {
            return Err(async_graphql::Error::new("Game ID too long (maximum 100 characters)"));
        }
        
        // Schedule game deprecation operation
        self.runtime.schedule_operation(&Operation::DeprecateGame {
            caller_discord_id: admin_discord_id,
            game_id: trimmed_game_id.to_string(),
        });
        
        Ok(format!("Game deprecation scheduled for game ID: {}", trimmed_game_id))
    }
}