// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Player-related GraphQL mutations

use std::sync::Arc;
use async_graphql::{Object, Result};
use crate::{infrastructure::operations::Operation, ScheduleOperation};

/// Player mutation resolvers
#[derive(Clone)]
pub struct PlayerMutations {
    pub runtime: Arc<dyn ScheduleOperation>,
}

#[Object]
impl PlayerMutations {
    /// Register a new player
    async fn register_player(
        &self,
        discord_id: String,
        username: String,
        avatar_url: Option<String>
    ) -> Result<String> {
        // Validate input before scheduling operation
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid Discord ID: {}", e)))?;
        
        crate::core::validation::player_validation::PlayerValidator::validate_username(&username)
            .map_err(|e| async_graphql::Error::new(format!("Invalid username: {}", e)))?;
            
        if let Some(ref url) = avatar_url {
            crate::core::validation::player_validation::PlayerValidator::validate_avatar_url(url)
                .map_err(|e| async_graphql::Error::new(format!("Invalid avatar URL: {}", e)))?;
        }
        
        // Schedule registration operation
        self.runtime.schedule_operation(&Operation::RegisterPlayer {
            discord_id: discord_id.clone(),
            username,
            avatar_url,
        });
        
        Ok(format!("Player registration scheduled for Discord ID: {}", discord_id))
    }

    /// Update player profile
    async fn update_player_profile(
        &self,
        discord_id: String,
        username: Option<String>,
        avatar_url: Option<String>
    ) -> Result<String> {
        // Validate Discord ID
        crate::core::validation::player_validation::PlayerValidator::validate_discord_id(&discord_id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid Discord ID: {}", e)))?;
        
        // Validate input
        if let Some(ref new_username) = username {
            crate::core::validation::player_validation::PlayerValidator::validate_username(new_username)
                .map_err(|e| async_graphql::Error::new(format!("Invalid username: {}", e)))?;
        }
        
        if let Some(ref url) = avatar_url {
            crate::core::validation::player_validation::PlayerValidator::validate_avatar_url(url)
                .map_err(|e| async_graphql::Error::new(format!("Invalid avatar URL: {}", e)))?;
        }
        
        // Ensure at least one field is being updated
        if username.is_none() && avatar_url.is_none() {
            return Err(async_graphql::Error::new("At least one field (username or avatar_url) must be provided for update"));
        }
        
        // Schedule profile update operation
        self.runtime.schedule_operation(&Operation::UpdatePlayerProfile {
            discord_id: discord_id.clone(),
            username,
            avatar_url,
        });
        
        Ok(format!("Profile update scheduled for player {}", discord_id))
    }
}