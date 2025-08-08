// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Admin-related GraphQL queries

use std::sync::Arc;
use async_graphql::{Object, Result};
use crate::{
    api::graphql_types::{AuditLogEntryObject, PlayerObject, PlayerStatusType},
    infrastructure::state::GameHubState,
};

/// Admin query resolvers
#[derive(Clone)]
pub struct AdminQueries {
    pub state: Arc<GameHubState>,
}

#[Object]
impl AdminQueries {
    /// Get audit log entries (admin only)
    async fn audit_log(&self, limit: Option<i32>) -> Result<Vec<AuditLogEntryObject>> {
        let limit = limit.unwrap_or(20).max(1).min(100) as usize; // Reasonable limits
        
        let entries = self.state.get_audit_log_entries().await;
        let limited_entries: Vec<_> = entries.into_iter().take(limit).collect();
        let mut log_objects = Vec::new();
        for entry in limited_entries {
            log_objects.push(AuditLogEntryObject {
                id: entry.id.clone(),
                action_type: format!("{:?}", entry.action),
                performed_by: entry.performed_by.clone(),
                target: entry.target.clone(),
                timestamp: entry.timestamp.micros().to_string(),
                details: entry.details.clone(),
            });
        }
        Ok(log_objects)
    }

    /// Get all admin users (admin only)
    async fn all_admins(&self) -> Result<Vec<String>> {
        // Get all admin IDs from the state
        let admin_ids = self.state.get_all_admins().await;
        Ok(admin_ids)
    }

    /// Get all moderator users (admin only)
    async fn all_moderators(&self) -> Result<Vec<String>> {
        // Get all moderator IDs from the state
        let moderator_ids = self.state.get_all_moderators().await;
        Ok(moderator_ids)
    }

    /// Get all banned players (admin/moderator only)
    async fn banned_players(&self) -> Result<Vec<PlayerObject>> {
        let players = self.state.get_all_players().await;
        let mut banned_players = Vec::new();
        
        for player in players {
            if let crate::core::types::player::PlayerStatus::Banned { .. } = player.status {
                banned_players.push(PlayerObject {
                    discord_id: player.discord_id.clone(),
                    username: player.username.clone(),
                    avatar_url: player.avatar_url.clone(),
                    total_points: player.total_points,
                    participation_streak: player.participation_streak,
                    current_rank: player.current_rank,
                    status: PlayerStatusType::from(&player.status),
                    created_at: player.created_at.micros().to_string(),
                    last_active: player.last_active.micros().to_string(),
                });
            }
        }
        Ok(banned_players)
    }

    /// Get all suspended players (admin/moderator only)
    async fn suspended_players(&self) -> Result<Vec<PlayerObject>> {
        let players = self.state.get_all_players().await;
        let mut suspended_players = Vec::new();
        
        for player in players {
            if let crate::core::types::player::PlayerStatus::Suspended { .. } = player.status {
                suspended_players.push(PlayerObject {
                    discord_id: player.discord_id.clone(),
                    username: player.username.clone(),
                    avatar_url: player.avatar_url.clone(),
                    total_points: player.total_points,
                    participation_streak: player.participation_streak,
                    current_rank: player.current_rank,
                    status: PlayerStatusType::from(&player.status),
                    created_at: player.created_at.micros().to_string(),
                    last_active: player.last_active.micros().to_string(),
                });
            }
        }
        Ok(suspended_players)
    }
}