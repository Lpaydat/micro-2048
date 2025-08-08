// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Admin-related GraphQL mutations

use std::sync::Arc;
use async_graphql::{Object, Result};
use crate::{infrastructure::operations::Operation, ScheduleOperation};

/// Admin mutation resolvers  
#[derive(Clone)]
pub struct AdminMutations {
    pub runtime: Arc<dyn ScheduleOperation>,
}

#[Object]
impl AdminMutations {
    /// Add a new admin (admin only)
    async fn add_admin(&self, caller_discord_id: String, new_admin_discord_id: String) -> Result<String> {
        // Validate Discord ID formats
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&caller_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid caller Discord ID: {}", e)))?;
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&new_admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid new admin Discord ID: {}", e)))?;
        
        // Prevent self-promotion (basic validation)
        if caller_discord_id == new_admin_discord_id {
            return Err(async_graphql::Error::new("Cannot add yourself as admin"));
        }
        
        // Schedule admin addition operation
        self.runtime.schedule_operation(&Operation::AddAdmin {
            caller_discord_id,
            discord_id: new_admin_discord_id.clone(),
        });
        
        Ok(format!("Admin addition scheduled for Discord ID: {}", new_admin_discord_id))
    }

    /// Remove an admin (admin only)
    async fn remove_admin(&self, caller_discord_id: String, admin_discord_id: String) -> Result<String> {
        // Validate Discord ID formats
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&caller_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid caller Discord ID: {}", e)))?;
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&admin_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid admin Discord ID: {}", e)))?;
        
        // Basic validation - cannot remove yourself (prevents accidental lockout)
        if caller_discord_id == admin_discord_id {
            return Err(async_graphql::Error::new("Cannot remove yourself as admin. Use another admin account."));
        }
        
        // Schedule admin removal operation
        self.runtime.schedule_operation(&Operation::RemoveAdmin {
            caller_discord_id,
            discord_id: admin_discord_id.clone(),
        });
        
        Ok(format!("Admin removal scheduled for Discord ID: {}", admin_discord_id))
    }

    /// Assign moderator role (admin only)
    async fn assign_moderator(&self, caller_discord_id: String, moderator_discord_id: String) -> Result<String> {
        // Validate Discord ID formats
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&caller_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid caller Discord ID: {}", e)))?;
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&moderator_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid moderator Discord ID: {}", e)))?;
        
        // Prevent assigning moderator to yourself (admins don't need moderator role)
        if caller_discord_id == moderator_discord_id {
            return Err(async_graphql::Error::new("Admins cannot assign moderator role to themselves"));
        }
        
        // Schedule moderator assignment operation
        self.runtime.schedule_operation(&Operation::AssignModerator {
            caller_discord_id,
            discord_id: moderator_discord_id.clone(),
        });
        
        Ok(format!("Moderator assignment scheduled for Discord ID: {}", moderator_discord_id))
    }

    /// Remove moderator role (admin only)
    async fn remove_moderator(&self, caller_discord_id: String, moderator_discord_id: String) -> Result<String> {
        // Validate Discord ID formats
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&caller_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid caller Discord ID: {}", e)))?;
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&moderator_discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid moderator Discord ID: {}", e)))?;
        
        // Prevent removing moderator from yourself (admins shouldn't be moderators)
        if caller_discord_id == moderator_discord_id {
            return Err(async_graphql::Error::new("Cannot remove moderator role from yourself"));
        }
        
        // Schedule moderator removal operation
        self.runtime.schedule_operation(&Operation::RemoveModerator {
            caller_discord_id,
            discord_id: moderator_discord_id.clone(),
        });
        
        Ok(format!("Moderator removal scheduled for Discord ID: {}", moderator_discord_id))
    }
}