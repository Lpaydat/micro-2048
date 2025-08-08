// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Batch processing state operations
//!
//! This module handles batch processing of player updates, event participation,
//! and bulk data operations for efficient cross-chain processing.

use crate::core::types::*;
use crate::infrastructure::errors::GameHubError;
use super::{GameHubState, player_state, event_state};

/// Process single player update
pub async fn process_player_update(
    state: &mut GameHubState,
    update: &PlayerEventUpdate,
    event_id: &str,
) -> Result<(), GameHubError> {
    if player_state::player_exists(state, &update.discord_id).await {
        process_registered_player_update(state, update, event_id).await
    } else {
        // Add to pending data
        add_pending_player_data(state, update, event_id).await
    }
}

/// Process registered player update
pub async fn process_registered_player_update(
    state: &mut GameHubState,
    update: &PlayerEventUpdate,
    _event_id: &str,
) -> Result<(), GameHubError> {
    let mut player = player_state::get_player(state, &update.discord_id).await
        .ok_or(GameHubError::PlayerNotFound)?;
    
    // Update player's last activity
    player.last_active = update.participation_timestamp;
    
    // Calculate new streak if eligible
    if update.streak_eligible {
        let new_streak = calculate_new_streak(state, &update.discord_id, player.last_active, update.participation_timestamp).await;
        player.participation_streak = new_streak;
        
        // Update best_streak if current streak is higher
        if player.participation_streak > player.best_streak {
            player.best_streak = player.participation_streak;
        }
    }
    
    // Calculate points with streak booster
    let boosted_points = calculate_points_with_streak_booster(state, update.score, player.participation_streak).await;
    player.total_points += boosted_points;
    
    // Save updated player
    state.players.insert(&update.discord_id, player).map_err(|_| GameHubError::DatabaseError)?;
    
    Ok(())
}

/// Process batch player updates
pub async fn process_batch_player_updates(
    state: &mut GameHubState,
    updates: Vec<PlayerEventUpdate>,
    event_id: &str,
) -> BatchUpdateResult {
    let mut successful_updates = Vec::new();
    let mut failed_updates = Vec::new();
    let mut unregistered_players = Vec::new();
    
    for update in updates {
        match process_player_update(state, &update, event_id).await {
            Ok(()) => {
                if player_state::player_exists(state, &update.discord_id).await {
                    successful_updates.push(update.discord_id);
                } else {
                    unregistered_players.push(update.discord_id);
                }
            }
            Err(error) => {
                failed_updates.push(BatchUpdateError {
                    player_discord_id: update.discord_id,
                    error,
                });
            }
        }
    }
    
    BatchUpdateResult {
        successful_updates,
        failed_updates,
        unregistered_players,
    }
}

/// Add pending player data
pub async fn add_pending_player_data(
    state: &mut GameHubState,
    update: &PlayerEventUpdate,
    event_id: &str,
) -> Result<(), GameHubError> {
    let event_score = EventScore {
        event_id: event_id.to_string(),
        game_id: event_state::get_game_id_for_event(state, event_id).await.unwrap_or_else(|| "unknown".to_string()),
        score: update.score,
        participation_timestamp: update.participation_timestamp,
        streak_eligible: update.streak_eligible,
    };
    
    if let Some(mut pending_data) = player_state::get_pending_data(state, &update.discord_id).await {
        // Update existing pending data
        pending_data.event_scores.push(event_score);
        pending_data.total_pending_points += update.score;
        
        state.pending_player_data.insert(&update.discord_id, pending_data).map_err(|_| GameHubError::DatabaseError)?;
    } else {
        // Create new pending data
        let pending_data = PendingPlayerData {
            discord_id: update.discord_id.clone(),
            event_scores: vec![event_score],
            total_pending_points: update.score,
            first_activity: update.participation_timestamp,
        };
        
        state.pending_player_data.insert(&update.discord_id, pending_data).map_err(|_| GameHubError::DatabaseError)?;
    }
    
    Ok(())
}

// ========== HELPER FUNCTIONS ==========

/// Calculate new streak for a player based on activity
async fn calculate_new_streak(
    state: &GameHubState,
    discord_id: &str,
    last_active: linera_sdk::linera_base_types::Timestamp,
    participation_timestamp: linera_sdk::linera_base_types::Timestamp,
) -> u32 {
    // Get the current player to check existing streak
    let current_player = match super::player_state::get_player(state, discord_id).await {
        Some(player) => player,
        None => return 1, // New player, start with streak of 1
    };
    
    // Get scoring configuration for grace period
    let scoring_config = state.scoring_config.get();
    let grace_period_hours = scoring_config.grace_period_hours;
    
    // Calculate time gap between last activity and new participation
    let time_gap_micros = participation_timestamp.micros().saturating_sub(last_active.micros());
    let time_gap_hours = time_gap_micros / (60 * 60 * 1_000_000); // Convert microseconds to hours
    
    // Check if the gap is within grace period
    if time_gap_hours <= grace_period_hours as u64 {
        // Within grace period - continue streak
        current_player.participation_streak + 1
    } else {
        // Gap too large - reset streak to 1
        1
    }
}

/// Calculate points with streak booster applied
async fn calculate_points_with_streak_booster(
    state: &GameHubState,
    base_score: u64,
    participation_streak: u32,
) -> u64 {
    // Get scoring configuration (should be initialized in contract setup)
    let scoring_config = state.scoring_config.get();
    
    // Find the best applicable booster level
    let mut best_multiplier = 100; // Default 1.0x multiplier
    for booster in &scoring_config.booster_levels {
        if participation_streak >= booster.required_streak {
            best_multiplier = booster.multiplier;
        }
    }
    
    // Calculate boosted points (multiplier is in percentage, so divide by 100)
    (base_score * best_multiplier) / 100
}