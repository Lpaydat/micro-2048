// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Leaderboard-related GraphQL queries

use std::sync::Arc;
use async_graphql::{Object, Result};
use crate::{
    api::graphql_types::{
        LeaderboardEntryObject, MainLeaderboardEntryObject, PlayerObject, 
        EnhancedParticipationDataObject, PlayerStatusType
    },
    infrastructure::state::GameHubState,
};

/// Leaderboard query resolvers
#[derive(Clone)]
pub struct LeaderboardQueries {
    pub state: Arc<GameHubState>,
}

#[Object]
impl LeaderboardQueries {
    /// Get leaderboard with optional limit (legacy method)
    async fn leaderboard(&self, limit: Option<i32>) -> Result<Vec<LeaderboardEntryObject>> {
        let limit = limit.unwrap_or(50).max(1).min(100) as usize; // Reasonable limits
        
        let players = self.state.get_all_players().await;
        let mut leaderboard = Vec::new();
        for (rank, player) in players.into_iter().take(limit).enumerate() {
            // Calculate real event-based points earned vs total accumulated points
            let event_points = self.state.get_player_event_points(&player.discord_id).await;
            
            leaderboard.push(LeaderboardEntryObject {
                player_discord_id: player.discord_id.clone(),
                player_username: player.username.clone(),
                score: player.total_points,
                rank: (rank + 1) as u32,
                points_earned: event_points, // Real event-based calculation
                completion_time: Some(player.last_active.micros().to_string()),
            });
        }
        Ok(leaderboard)
    }

    /// Get main leaderboard with pagination and enhanced participation data
    async fn main_leaderboard(&self, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<MainLeaderboardEntryObject>> {
        let limit = limit.unwrap_or(50).max(1).min(100) as usize;
        let offset = offset.unwrap_or(0).max(0) as usize;
        
        let players = self.state.get_all_players().await;
        let mut leaderboard = Vec::new();
        
        // Apply pagination
        for (index, player) in players.into_iter().skip(offset).take(limit).enumerate() {
            // Get current scoring config for booster calculation
            let scoring_config = self.state.get_scoring_config().await;
            
            // Calculate current booster level based on streak
            let (streak_level, streak_multiplier) = self.calculate_streak_booster(
                player.participation_streak, 
                &scoring_config
            );
            
            
            leaderboard.push(MainLeaderboardEntryObject {
                player: PlayerObject {
                    discord_id: player.discord_id.clone(),
                    username: player.username.clone(),
                    avatar_url: player.avatar_url.clone(),
                    total_points: player.total_points,
                    participation_streak: player.participation_streak,
                    current_rank: Some((offset + index + 1) as u32),
                    status: PlayerStatusType::from(&player.status),
                    created_at: player.created_at.micros().to_string(),
                    last_active: player.last_active.micros().to_string(),
                },
                participation_data: EnhancedParticipationDataObject {
                    streak_level,
                    streak_multiplier,
                    total_points_earned: player.total_points,
                    events_participated: self.calculate_events_participated(&player).await,
                    last_event_timestamp: Some(player.last_active.micros().to_string()),
                    booster_history: self.calculate_booster_history(&player, &scoring_config),
                },
            });
        }
        Ok(leaderboard)
    }

    /// Get event leaderboard for a specific event
    async fn event_leaderboard(&self, event_id: String, limit: Option<i32>) -> Result<Vec<LeaderboardEntryObject>> {
        let limit = limit.unwrap_or(50).max(1).min(100) as usize;
        
        // Check if event exists
        if self.state.get_event(&event_id).await.is_none() {
            return Ok(Vec::new());
        }

        // Collect all participants for this event
        let players = self.state.get_all_players().await;
        let mut participants = Vec::new();
        
        for player in players {
            if let Some(pending_data) = self.state.get_pending_data(&player.discord_id).await {
                // Find the event score for this specific event
                if let Some(event_score) = pending_data.event_scores.iter()
                    .find(|es| es.event_id == event_id)
                {
                    participants.push((player, event_score.clone()));
                }
            }
        }
        
        // Sort by score (descending)
        participants.sort_by(|a, b| b.1.score.cmp(&a.1.score));
        
        // Create leaderboard entries
        let mut leaderboard = Vec::new();
        for (rank, (player, event_score)) in participants.into_iter().take(limit).enumerate() {
            leaderboard.push(LeaderboardEntryObject {
                player_discord_id: player.discord_id.clone(),
                player_username: player.username.clone(),
                score: event_score.score,
                rank: (rank + 1) as u32,
                points_earned: event_score.score, // Simplified
                completion_time: Some(event_score.participation_timestamp.micros().to_string()),
            });
        }
        
        Ok(leaderboard)
    }

    /// Get game leaderboard for a specific game
    async fn game_leaderboard(&self, game_id: String, limit: Option<i32>) -> Result<Vec<LeaderboardEntryObject>> {
        let limit = limit.unwrap_or(50).max(1).min(100) as usize;
        
        // Check if game exists
        if !self.state.is_game_approved(&game_id).await {
            return Ok(Vec::new());
        }

        // Collect all players and their game-specific performance
        let players = self.state.get_all_players().await;
        let mut game_participants = Vec::new();
        
        for player in players {
            if let Some(pending_data) = self.state.get_pending_data(&player.discord_id).await {
                // Aggregate scores for this specific game
                let game_scores: Vec<_> = pending_data.event_scores.iter()
                    .filter(|es| es.game_id == game_id)
                    .collect();
                
                if !game_scores.is_empty() {
                    let total_score = game_scores.iter().map(|es| es.score).sum::<u64>();
                    let latest_participation = game_scores.iter()
                        .map(|es| es.participation_timestamp)
                        .max()
                        .unwrap();
                    
                    game_participants.push((player, total_score, latest_participation));
                }
            }
        }
        
        // Sort by total score for this game (descending)
        game_participants.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Create leaderboard entries
        let mut leaderboard = Vec::new();
        for (rank, (player, total_score, latest_participation)) in game_participants.into_iter().take(limit).enumerate() {
            leaderboard.push(LeaderboardEntryObject {
                player_discord_id: player.discord_id.clone(),
                player_username: player.username.clone(),
                score: total_score,
                rank: (rank + 1) as u32,
                points_earned: total_score, // Simplified
                completion_time: Some(latest_participation.micros().to_string()),
            });
        }
        
        Ok(leaderboard)
    }
}

impl LeaderboardQueries {
    /// Calculate streak booster level and multiplier
    fn calculate_streak_booster(&self, streak: u32, scoring_config: &crate::core::types::ScoringConfig) -> (String, u32) {
        for booster_level in scoring_config.booster_levels.iter().rev() {
            if streak >= booster_level.required_streak {
                return (booster_level.name.clone(), booster_level.multiplier.try_into().unwrap());
            }
        }
        ("None".to_string(), 100) // Default no booster (100%)
    }
    
    /// Calculate events participated for a player
    async fn calculate_events_participated(&self, player: &crate::core::types::Player) -> u32 {
        if let Some(pending_data) = self.state.get_pending_data(&player.discord_id).await {
            pending_data.event_scores.len() as u32
        } else {
            // Estimate based on total points for players without detailed history
            if player.total_points > 0 {
                (player.total_points / 100).max(1) as u32 // Assume ~100 points per event average
            } else {
                0
            }
        }
    }
    
    /// Calculate booster history for a player
    fn calculate_booster_history(&self, player: &crate::core::types::Player, scoring_config: &crate::core::types::ScoringConfig) -> Vec<String> {
        let mut history = Vec::new();
        
        // Add all booster levels the player qualifies for
        for booster_level in &scoring_config.booster_levels {
            if player.participation_streak >= booster_level.required_streak {
                history.push(format!("{} ({}x multiplier)", 
                    booster_level.name, 
                    booster_level.multiplier as f64 / 100.0));
            }
        }
        
        if history.is_empty() {
            history.push("No boosters earned".to_string());
        }
        
        history
    }
}