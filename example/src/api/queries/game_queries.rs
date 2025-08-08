// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Game-related GraphQL queries

use std::sync::Arc;
use async_graphql::{Object, Result};
use crate::{
    api::graphql_types::{GameObject, PendingGameObject, GameStatusType, GameStatsObject},
    infrastructure::state::GameHubState,
};

/// Game query resolvers
#[derive(Clone)]
pub struct GameQueries {
    pub state: Arc<GameHubState>,
}

#[Object]
impl GameQueries {
    /// Get all approved games
    async fn approved_games(&self) -> Result<Vec<GameObject>> {
        let games = self.state.get_all_games().await;
        let mut game_objects = Vec::new();
        for game in games {
            game_objects.push(GameObject {
                id: game.id.clone(),
                name: game.name.clone(),
                description: game.description.clone(),
                contract_address: game.contract_address.clone(),
                developer_name: game.developer_info.name.clone(),
                developer_contact: game.developer_info.contact.clone(),
                status: GameStatusType::from(&game.status),
                approved_by: game.approved_by.clone(),
                created_at: game.created_at.micros().to_string(),
                approved_at: game.approved_at.map(|t| t.micros().to_string()),
            });
        }
        Ok(game_objects)
    }

    /// Get pending games (admin only)
    async fn pending_games(&self, admin_discord_id: Option<String>) -> Result<Vec<PendingGameObject>> {
        // Validate admin permissions if admin_discord_id is provided
        if let Some(admin_id) = admin_discord_id {
            if !self.state.is_admin(&admin_id).await {
                return Err(async_graphql::Error::new("Insufficient permissions: Admin access required"));
            }
        } else {
            // If no admin ID provided, return error for security
            return Err(async_graphql::Error::new("Admin Discord ID required for pending games access"));
        }
        
        let games = self.state.get_all_pending_games().await;
        let mut game_objects = Vec::new();
        for game in games {
            game_objects.push(PendingGameObject {
                id: game.id.clone(),
                name: game.name.clone(),
                description: game.description.clone(),
                contract_address: game.contract_address.clone(),
                developer_name: game.developer_info.name.clone(),
                developer_contact: game.developer_info.contact.clone(),
                created_at: game.created_at.micros().to_string(),
            });
        }
        Ok(game_objects)
    }

    /// Check if a game is approved
    async fn game_approved(&self, game_id: String) -> Result<bool> {
        Ok(self.state.is_game_approved(&game_id).await)
    }

    /// Get game statistics and popularity metrics (admin only)
    async fn game_stats(&self, limit: Option<i32>) -> Result<Vec<GameStatsObject>> {
        let limit = limit.unwrap_or(20).max(1).min(100) as usize;
        let games = self.state.get_all_games().await;
        let all_events = self.state.get_all_events().await;
        
        let mut game_stats = Vec::new();
        
        for game in games.into_iter().take(limit) {
            // Calculate stats for this game
            let game_events: Vec<_> = all_events.iter()
                .filter(|event| event.game_id == game.id)
                .collect();
            
            let total_events = game_events.len() as u32;
            let total_participants = game_events.iter()
                .map(|event| event.participant_count)
                .sum::<u32>();
            
            let average_participants_per_event = if total_events > 0 {
                total_participants as f64 / total_events as f64
            } else {
                0.0
            };
            
            let last_event_date = game_events.iter()
                .map(|event| event.end_time)
                .max()
                .map(|timestamp| timestamp.micros().to_string());
                
            // Calculate popularity score based on events and participants
            let popularity_score = (total_events as f64 * 0.3) + (total_participants as f64 * 0.7);
            
            game_stats.push(GameStatsObject {
                game_id: game.id.clone(),
                game_name: game.name.clone(),
                total_events,
                total_participants,
                unique_players: total_participants, // Simplified - would need proper unique counting
                average_participants_per_event,
                last_event_date,
                popularity_score,
            });
        }
        
        // Sort by popularity score (descending)
        game_stats.sort_by(|a, b| b.popularity_score.partial_cmp(&a.popularity_score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(game_stats)
    }
}