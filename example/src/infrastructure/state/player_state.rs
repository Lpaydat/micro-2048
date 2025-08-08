// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Player state management operations
//!
//! This module handles all player-related blockchain state operations including
//! registration, profile management, activity tracking, and pending data processing.

use linera_sdk::linera_base_types::Timestamp;
use crate::core::types::*;
use crate::core::validation::player_validation::PlayerValidator;
use crate::infrastructure::errors::GameHubError;
use super::GameHubState;

/// Get player by Discord ID
pub async fn get_player(state: &GameHubState, discord_id: &str) -> Option<Player> {
    state.players.get(discord_id).await.ok().flatten()
}

/// Check if player exists
pub async fn player_exists(state: &GameHubState, discord_id: &str) -> bool {
    state.players.get(discord_id).await.ok().flatten().is_some()
}

/// Check if player is active (not banned or suspended)
pub async fn is_player_active(state: &GameHubState, discord_id: &str) -> bool {
    match get_player(state, discord_id).await {
        Some(player) => matches!(player.status, PlayerStatus::Active),
        None => false,
    }
}

/// Update player's last activity timestamp
pub async fn update_player_activity(
    state: &mut GameHubState,
    discord_id: &str,
    timestamp: Timestamp,
) -> Result<(), GameHubError> {
    let mut player = get_player(state, discord_id).await
        .ok_or(GameHubError::PlayerNotFound)?;
    
    player.last_active = timestamp;
    
    state.players.insert(discord_id, player).map_err(|_| GameHubError::DatabaseError)?;
    
    Ok(())
}

/// Register or update player with pending data merging
pub async fn register_or_update_player(
    state: &mut GameHubState,
    discord_id: &str,
    username: &str,
    avatar_url: Option<String>,
    timestamp: Timestamp,
) -> Result<Player, GameHubError> {
    // Validate input data
    PlayerValidator::validate_complete_player_registration(
        discord_id,
        username,
        avatar_url.as_deref(),
    )?;
    
    // Check if player already exists
    if let Some(mut existing_player) = get_player(state, discord_id).await {
        // Update existing player profile
        existing_player.username = username.to_string();
        existing_player.avatar_url = avatar_url;
        existing_player.last_active = timestamp;
        
        state.players.insert(discord_id, existing_player.clone()).map_err(|_| GameHubError::DatabaseError)?;
        return Ok(existing_player);
    }
    
    // Create new player, check for pending data
    let (total_points, participation_streak) = if let Some(pending_data) = state.pending_player_data.get(discord_id).await.ok().flatten() {
        let streak = super::utility_state::calculate_streak_from_pending(state, &pending_data).await;
        (pending_data.total_pending_points, streak)
    } else {
        (0, 0)
    };
    
    let new_player = Player {
        discord_id: discord_id.to_string(),
        username: username.to_string(),
        avatar_url,
        total_points,
        participation_streak,
        best_streak: participation_streak, // Initialize best_streak with current streak
        current_rank: None,
        status: PlayerStatus::Active,
        created_at: timestamp,
        last_active: timestamp,
    };
    
    state.players.insert(discord_id, new_player.clone()).map_err(|_| GameHubError::DatabaseError)?;
    
    // Remove pending data after successful registration
    if state.pending_player_data.get(discord_id).await.ok().flatten().is_some() {
        state.pending_player_data.remove(discord_id).map_err(|_| GameHubError::DatabaseError)?;
    }
    
    Ok(new_player)
}

/// Get player statistics
pub async fn get_player_stats(state: &GameHubState, discord_id: &str) -> Option<PlayerStats> {
    let player = get_player(state, discord_id).await?;
    
    Some(PlayerStats {
        total_points: player.total_points,
        participation_streak: player.participation_streak,
        current_rank: player.current_rank,
        status: player.status,
        created_at: player.created_at,
        last_active: player.last_active,
    })
}

/// Update player profile (username/avatar)
pub async fn update_player_profile(
    state: &mut GameHubState,
    discord_id: &str,
    username: Option<String>,
    avatar_url: Option<String>,
    timestamp: Timestamp,
) -> Result<(), GameHubError> {
    // Validate input data
    if let Some(ref new_username) = username {
        PlayerValidator::validate_username(new_username)?;
    }
    if let Some(ref new_avatar_url) = avatar_url {
        PlayerValidator::validate_avatar_url(new_avatar_url)?;
    }
    
    // Get existing player
    let mut player = get_player(state, discord_id).await
        .ok_or(GameHubError::PlayerNotFound)?;
    
    // Update fields if provided
    if let Some(new_username) = username {
        player.username = new_username;
    }
    if let Some(new_avatar_url) = avatar_url {
        player.avatar_url = Some(new_avatar_url);
    }
    
    // Update last active timestamp
    player.last_active = timestamp;
    
    // Save updated player
    state.players.insert(discord_id, player).map_err(|_| GameHubError::DatabaseError)?;
    
    // Log the profile update action
    super::admin_state::add_audit_log_entry(
        state,
        AdminAction::PlayerProfileUpdated { 
            player_id: discord_id.to_string() 
        },
        discord_id, // Player updating their own profile
        Some(discord_id),
        Some("Player profile updated"),
        timestamp,
    ).await?;
    
    Ok(())
}

/// Check if player has pending data
pub async fn has_pending_data(state: &GameHubState, discord_id: &str) -> bool {
    state.pending_player_data.get(discord_id).await.ok().flatten().is_some()
}

/// Get pending player data
pub async fn get_pending_data(state: &GameHubState, discord_id: &str) -> Option<PendingPlayerData> {
    state.pending_player_data.get(discord_id).await.ok().flatten()
}

/// Get all pending player data using MapView iteration
pub async fn get_all_pending_player_data(state: &GameHubState) -> Vec<PendingPlayerData> {
    let pending_ids = match state.pending_player_data.indices().await {
        Ok(indices) => indices,
        Err(_) => return Vec::new(),
    };
    
    let mut pending_data = Vec::new();
    for pending_id in pending_ids {
        if let Ok(Some(data)) = state.pending_player_data.get(&pending_id).await {
            pending_data.push(data);
        }
    }
    
    // Sort by first activity (most recent first)
    pending_data.sort_by(|a, b| b.first_activity.micros().cmp(&a.first_activity.micros()));
    pending_data
}

/// Calculate event-based points earned for a player (vs total accumulated points)
pub async fn get_player_event_points(state: &GameHubState, discord_id: &str) -> u64 {
    // Check if player has pending data with individual event scores
    if let Some(pending_data) = get_pending_data(state, discord_id).await {
        // Sum up points from individual events
        pending_data.event_scores.iter().map(|event| event.score).sum()
    } else {
        // For registered players without detailed event history, use total_points
        // In a complete system, we would track individual event participations
        get_player(state, discord_id).await.map(|p| p.total_points).unwrap_or(0)
    }
}

/// Get all players using MapView iteration
pub async fn get_all_players(state: &GameHubState) -> Vec<Player> {
    let player_ids = match state.players.indices().await {
        Ok(indices) => indices,
        Err(_) => return Vec::new(),
    };
    
    let mut players = Vec::new();
    for player_id in player_ids {
        if let Ok(Some(player)) = state.players.get(&player_id).await {
            players.push(player);
        }
    }
    
    // Sort players by total points (highest first) for leaderboard consistency
    players.sort_by(|a, b| b.total_points.cmp(&a.total_points));
    players
}