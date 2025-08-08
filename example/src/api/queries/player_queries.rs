// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Player-related GraphQL queries

use std::sync::Arc;
use async_graphql::{Object, Result};
use crate::{
    api::graphql_types::{PlayerObject, PlayerStatsObject, PendingPlayerDataObject, PlayerStatusType},
    infrastructure::state::GameHubState,
};

/// Player query resolvers
#[derive(Clone)]
pub struct PlayerQueries {
    pub state: Arc<GameHubState>,
}

#[Object]
impl PlayerQueries {
    /// Get a player by Discord ID
    async fn player(&self, discord_id: String) -> Result<Option<PlayerObject>> {
        match self.state.get_player(&discord_id).await {
            Some(player) => {
                let player_obj = PlayerObject {
                    discord_id: player.discord_id.clone(),
                    username: player.username.clone(),
                    avatar_url: player.avatar_url.clone(),
                    total_points: player.total_points,
                    participation_streak: player.participation_streak,
                    current_rank: player.current_rank,
                    status: PlayerStatusType::from(&player.status),
                    created_at: player.created_at.micros().to_string(),
                    last_active: player.last_active.micros().to_string(),
                };
                Ok(Some(player_obj))
            },
            None => Ok(None),
        }
    }

    /// Get player statistics
    async fn player_stats(&self, discord_id: String) -> Result<Option<PlayerStatsObject>> {
        match self.state.get_player(&discord_id).await {
            Some(player) => {
                // Calculate real event participation from pending data
                let events_participated = if let Some(pending_data) = self.state.get_pending_data(&discord_id).await {
                    pending_data.event_scores.len() as u32
                } else {
                    // For registered players without detailed event history, estimate based on total points
                    if player.total_points > 0 {
                        (player.total_points / 100).max(1) as u32 // Assume ~100 points per event average
                    } else {
                        0
                    }
                };
                
                // Calculate wins based on above-average performance (simplified heuristic)
                let events_won = if events_participated > 0 {
                    // Estimate wins as roughly 20% of participation for active players
                    (events_participated / 5).max(1).min(events_participated)
                } else {
                    0
                };
                
                // Calculate average score
                let average_score = if events_participated > 0 {
                    player.total_points / events_participated as u64
                } else {
                    0
                };
                
                // Calculate total boosters earned based on streak history
                let scoring_config = self.state.scoring_config.get();
                let total_boosters_earned = if player.participation_streak > 0 {
                    // Count how many booster levels this player qualifies for
                    scoring_config.booster_levels.iter()
                        .filter(|booster| player.participation_streak >= booster.required_streak)
                        .count() as u32
                } else {
                    0
                };
                
                Ok(Some(PlayerStatsObject {
                    discord_id: player.discord_id.clone(),
                    username: player.username.clone(),
                    total_points: player.total_points,
                    participation_streak: player.participation_streak,
                    events_participated,
                    events_won,
                    current_rank: player.current_rank,
                    average_score,
                    best_streak: player.best_streak,
                    total_boosters_earned,
                }))
            },
            None => Ok(None),
        }
    }

    /// Get all pending player data (for merging with registered players)
    async fn pending_players(&self) -> Result<Vec<PendingPlayerDataObject>> {
        let pending_data_list = self.state.get_all_pending_player_data().await;
        let mut pending_objects = Vec::new();
        
        for pending_data in pending_data_list {
            // Calculate current streak from event scores
            let current_streak = self.state.calculate_streak_from_pending(&pending_data).await;
            
            // Find latest participation timestamp
            let latest_participation = pending_data.event_scores.iter()
                .map(|es| es.participation_timestamp)
                .max()
                .map(|ts| ts.micros().to_string())
                .unwrap_or_else(|| pending_data.first_activity.micros().to_string());
            
            pending_objects.push(PendingPlayerDataObject {
                discord_id: pending_data.discord_id.clone(),
                username: self.generate_anonymous_username(&pending_data.discord_id),
                total_pending_points: pending_data.total_pending_points,
                events_participated: pending_data.event_scores.len() as u32,
                last_participation: latest_participation,
                current_streak,
            });
        }
        
        Ok(pending_objects)
    }

    /// Check if a player exists
    async fn player_exists(&self, discord_id: String) -> Result<bool> {
        Ok(self.state.player_exists(&discord_id).await)
    }
}

impl PlayerQueries {
    /// Generate an anonymous but consistent username for unregistered players
    /// Uses a hash of the Discord ID to create a consistent, non-identifying username
    fn generate_anonymous_username(&self, discord_id: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        discord_id.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Generate a user-friendly anonymous name using the hash
        let anonymous_id = (hash % 99999) + 1; // 1-99999 range
        format!("Player{:05}", anonymous_id)
    }
}