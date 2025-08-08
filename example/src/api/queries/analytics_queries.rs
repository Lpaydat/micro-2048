// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Analytics-related GraphQL queries

use std::sync::Arc;
use async_graphql::{Object, Result};
use crate::{
    api::graphql_types::{
        GameHistoryEntryObject, EventParticipationObject, StreakHistoryEntryObject,
        SystemHealthObject, AnalyticsObject, DateRangeInput, PlayerEngagementObject
    },
    infrastructure::state::GameHubState,
    core::types::{Player, EventScore},
};

/// Analytics query resolvers
#[derive(Clone)]
pub struct AnalyticsQueries {
    pub state: Arc<GameHubState>,
}

#[Object]
impl AnalyticsQueries {
    /// Get player game history
    async fn game_history(&self, discord_id: String) -> Result<Vec<GameHistoryEntryObject>> {
        // Check if player exists
        if !self.state.player_exists(&discord_id).await {
            return Ok(Vec::new());
        }

        // Get pending player data which contains event participation
        let mut game_history = Vec::new();
        
        if let Some(pending_data) = self.state.get_pending_data(&discord_id).await {
            // Group event scores by game_id
            let mut game_stats: std::collections::HashMap<String, (Vec<u64>, u32)> = std::collections::HashMap::new();
            
            for event_score in &pending_data.event_scores {
                let entry = game_stats.entry(event_score.game_id.clone()).or_insert((Vec::new(), 0));
                entry.0.push(event_score.score);
                if event_score.streak_eligible {
                    entry.1 += 1;
                }
            }
            
            // Convert to GameHistoryEntryObject
            for (game_id, (scores, current_streak)) in game_stats {
                let game_name = if let Some(game) = self.state.get_game(&game_id).await {
                    game.name
                } else {
                    "Unknown Game".to_string()
                };
                
                let events_participated = scores.len() as u32;
                let total_score = scores.iter().sum::<u64>();
                let best_score = scores.iter().max().copied().unwrap_or(0);
                
                // Find first and last participation dates for this game
                let game_event_scores: Vec<_> = pending_data.event_scores.iter()
                    .filter(|es| es.game_id == game_id)
                    .collect();
                    
                let first_participation = game_event_scores.iter()
                    .map(|es| es.participation_timestamp)
                    .min()
                    .map(|ts| ts.micros().to_string())
                    .unwrap_or_else(|| "".to_string());
                    
                let last_participation = game_event_scores.iter()
                    .map(|es| es.participation_timestamp)
                    .max()
                    .map(|ts| ts.micros().to_string())
                    .unwrap_or_else(|| "".to_string());
                
                game_history.push(GameHistoryEntryObject {
                    game_id,
                    game_name,
                    events_participated,
                    total_score,
                    best_score,
                    first_participation,
                    last_participation,
                    current_streak,
                });
            }
        }

        // Sort by last participation (most recent first)
        game_history.sort_by(|a, b| b.last_participation.cmp(&a.last_participation));
        
        Ok(game_history)
    }

    /// Get player event participation history
    async fn event_participation(&self, discord_id: String, limit: Option<i32>) -> Result<Vec<EventParticipationObject>> {
        let limit = limit.unwrap_or(20).max(1).min(100) as usize;
        
        // Get player data for points calculation
        let player = if let Some(player) = self.state.get_player(&discord_id).await {
            player
        } else {
            return Ok(Vec::new());
        };

        // Get pending player data which contains event scores
        let mut participations = Vec::new();
        
        if let Some(pending_data) = self.state.get_pending_data(&discord_id).await {
            for event_score in pending_data.event_scores.into_iter().take(limit) {
                // Calculate points first before moving fields
                let points_earned = self.calculate_event_points(&event_score, &player).await;
                
                // Get event details
                let event_name = if let Some(event) = self.state.get_event(&event_score.event_id).await {
                    event.name
                } else {
                    "Unknown Event".to_string()
                };

                // Get game name
                let game_name = if let Some(game) = self.state.get_game(&event_score.game_id).await {
                    game.name
                } else {
                    "Unknown Game".to_string()
                };

                participations.push(EventParticipationObject {
                    event_id: event_score.event_id,
                    event_name,
                    game_id: event_score.game_id,
                    game_name,
                    score: event_score.score,
                    rank: None, // Would need ranking calculation
                    points_earned,
                    booster_applied: if event_score.streak_eligible { Some("Eligible".to_string()) } else { None },
                    participation_timestamp: event_score.participation_timestamp.micros().to_string(),
                    streak_eligible: event_score.streak_eligible,
                });
            }
        }

        // Sort by participation timestamp (most recent first)
        participations.sort_by(|a, b| b.participation_timestamp.cmp(&a.participation_timestamp));
        
        Ok(participations)
    }

    /// Get player streak history
    async fn streak_history(&self, discord_id: String) -> Result<Vec<StreakHistoryEntryObject>> {
        // Check if player exists
        if !self.state.player_exists(&discord_id).await {
            return Ok(Vec::new());
        }

        let mut streak_history = Vec::new();
        
        if let (Some(player), Some(pending_data)) = (
            self.state.get_player(&discord_id).await,
            self.state.get_pending_data(&discord_id).await
        ) {
            if player.participation_streak > 0 {
                // Get scoring config to determine booster level
                let scoring_config = self.state.get_scoring_config().await;
                let (booster_name, booster_multiplier) = self.calculate_streak_booster(
                    player.participation_streak, 
                    &scoring_config
                );
                
                // Get streak-eligible events for current streak
                let streak_events: Vec<String> = pending_data.event_scores.iter()
                    .filter(|es| es.streak_eligible)
                    .map(|es| es.event_id.clone())
                    .collect();
                
                // Calculate approximate start date by looking at recent events
                let start_date = if let Some(oldest_event) = pending_data.event_scores.iter()
                    .filter(|es| es.streak_eligible)
                    .map(|es| es.participation_timestamp)
                    .min()
                {
                    oldest_event.micros().to_string()
                } else {
                    player.created_at.micros().to_string()
                };
                
                // Calculate bonus points from current streak
                let total_bonus_points = pending_data.event_scores.iter()
                    .filter(|es| es.streak_eligible)
                    .map(|es| {
                        // Calculate what the base points would be vs actual points
                        // This is simplified - would need more detailed tracking
                        let base_points = es.score / booster_multiplier as u64;
                        es.score - base_points
                    })
                    .sum();

                streak_history.push(StreakHistoryEntryObject {
                    streak_count: player.participation_streak,
                    start_date,
                    end_date: None, // Current active streak
                    events_included: streak_events,
                    booster_level: Some(booster_name),
                    total_bonus_points,
                });
            }
        }

        Ok(streak_history)
    }

    /// Get system analytics and health metrics (admin only)
    async fn system_health(&self) -> Result<SystemHealthObject> {
        let total_players = self.state.get_all_players().await.len() as u32;
        let total_games = self.state.get_all_games().await.len() as u32;
        let pending_games = self.state.get_all_pending_games().await.len() as u32;
        let total_events = self.state.get_all_events().await.len() as u32;
        let audit_entries = self.state.get_audit_log_entries().await.len() as u32;

        Ok(SystemHealthObject {
            total_players,
            total_games,
            pending_games,
            total_events,
            total_audit_entries: audit_entries,
            active_players: self.calculate_active_players().await,
            recent_registrations: self.calculate_recent_registrations().await,
        })
    }

    /// Get analytics data with date range filtering (admin only)
    async fn analytics(&self, date_range: Option<DateRangeInput>) -> Result<AnalyticsObject> {
        let all_players = self.state.get_all_players().await;
        let all_games = self.state.get_all_games().await;
        let all_events = self.state.get_all_events().await;
        
        let total_players = all_players.len() as u32;
        let total_games = all_games.len() as u32;
        let total_events = all_events.len() as u32;
        
        let (active_players_in_period, new_registrations_in_period, player_engagement) = 
            if let Some(date_range) = date_range {
                // Parse date range strings to timestamps for filtering
                let start_timestamp = self.parse_date_to_timestamp(&date_range.start_date);
                let end_timestamp = self.parse_date_to_timestamp(&date_range.end_date);
                
                if let (Some(start_ts), Some(end_ts)) = (start_timestamp, end_timestamp) {
                    // Filter players by registration date within range
                    let new_registrations = all_players.iter()
                        .filter(|player| {
                            player.created_at.micros() >= start_ts && 
                            player.created_at.micros() <= end_ts
                        })
                        .count() as u32;
                    
                    // Filter players by activity within range
                    let active_players = all_players.iter()
                        .filter(|player| {
                            player.last_active.micros() >= start_ts && 
                            player.last_active.micros() <= end_ts
                        })
                        .count() as u32;
                    
                    // Generate daily engagement metrics
                    let engagement = self.calculate_daily_engagement(&all_players, start_ts, end_ts).await;
                    
                    (active_players, new_registrations, engagement)
                } else {
                    // Invalid date format, return all-time metrics
                    (total_players, total_players, Vec::new())
                }
            } else {
                // Return all-time metrics when no date range specified
                (total_players, total_players, Vec::new())
            };

        Ok(AnalyticsObject {
            total_players,
            active_games: total_games,
            total_events,
            active_players_in_period,
            new_registrations_in_period,
            player_engagement,
        })
    }
}

impl AnalyticsQueries {
    /// Calculate streak booster level and multiplier
    fn calculate_streak_booster(&self, streak: u32, scoring_config: &crate::core::types::ScoringConfig) -> (String, u32) {
        for booster_level in scoring_config.booster_levels.iter().rev() {
            if streak >= booster_level.required_streak {
                return (booster_level.name.clone(), booster_level.multiplier.try_into().unwrap());
            }
        }
        ("None".to_string(), 100) // Default no booster (100%)
    }

    /// Parse date string to timestamp (microseconds)
    fn parse_date_to_timestamp(&self, date_str: &str) -> Option<u64> {
        // Simple date parsing - in production would use proper date parsing library
        // Expected format: "YYYY-MM-DD" or timestamp string
        if let Ok(timestamp) = date_str.parse::<u64>() {
            Some(timestamp)
        } else {
            // Try to parse as date string and convert to timestamp
            // This is a simplified implementation
            None
        }
    }

    /// Calculate daily engagement metrics
    async fn calculate_daily_engagement(&self, players: &[Player], start_ts: u64, end_ts: u64) -> Vec<PlayerEngagementObject> {
        let mut engagement_data = Vec::new();
        
        // Group players by activity day (simplified daily buckets)
        let days_in_range = ((end_ts - start_ts) / (24 * 60 * 60 * 1_000_000)).max(1);
        
        for day in 0..days_in_range {
            let day_start = start_ts + (day * 24 * 60 * 60 * 1_000_000);
            let day_end = day_start + (24 * 60 * 60 * 1_000_000);
            
            // Count active players for this day
            let active_count = players.iter()
                .filter(|player| {
                    player.last_active.micros() >= day_start && 
                    player.last_active.micros() < day_end
                })
                .count() as u32;
            
            if active_count > 0 {
                engagement_data.push(PlayerEngagementObject {
                    date: format!("day_{}", day),
                    active_users: active_count,
                    new_registrations: 0, // Would need date-specific registration tracking
                    total_events: 0, // Would need event tracking for this day
                    total_participation: active_count, // Use active_count as participation approximation
                });
            }
        }
        
        engagement_data
    }
    
    /// Calculate active players (based on recent activity)
    async fn calculate_active_players(&self) -> u32 {
        let all_players = self.state.get_all_players().await;
        
        // Use the most recent player activity as a reference point
        let latest_activity = all_players.iter()
            .map(|player| player.last_active.micros())
            .max()
            .unwrap_or(0);
        
        let seven_days_ago = latest_activity.saturating_sub(7 * 24 * 60 * 60 * 1_000_000);
        
        all_players.iter()
            .filter(|player| player.last_active.micros() >= seven_days_ago)
            .count() as u32
    }
    
    /// Calculate recent registrations (based on relative time)
    async fn calculate_recent_registrations(&self) -> u32 {
        let all_players = self.state.get_all_players().await;
        
        // Use the most recent player registration as a reference point
        let latest_registration = all_players.iter()
            .map(|player| player.created_at.micros())
            .max()
            .unwrap_or(0);
        
        let thirty_days_ago = latest_registration.saturating_sub(30 * 24 * 60 * 60 * 1_000_000);
        
        all_players.iter()
            .filter(|player| player.created_at.micros() >= thirty_days_ago)
            .count() as u32
    }

    /// Calculate actual points earned for an event, including streak multipliers
    async fn calculate_event_points(&self, event_score: &EventScore, player: &Player) -> u64 {
        if event_score.streak_eligible {
            // Use scoring service to calculate points with streak booster
            self.state.calculate_points_with_streak_booster(
                event_score.score, 
                player.participation_streak
            ).await
        } else {
            // Non-streak-eligible events use base score
            event_score.score
        }
    }
}