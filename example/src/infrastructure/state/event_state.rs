// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Event state management operations
//!
//! This module handles all event-related blockchain state operations including
//! event lifecycle management, queries, and admin operations.

use linera_sdk::linera_base_types::Timestamp;
use crate::core::types::*;
use crate::infrastructure::errors::GameHubError;
use super::GameHubState;

/// Get event by ID
pub async fn get_event(state: &GameHubState, event_id: &str) -> Option<Event> {
    state.events.get(event_id).await.ok().flatten()
}

/// Check if event exists
pub async fn event_exists(state: &GameHubState, event_id: &str) -> bool {
    state.events.get(event_id).await.ok().flatten().is_some()
}

/// Get game ID for event
pub async fn get_game_id_for_event(state: &GameHubState, event_id: &str) -> Option<String> {
    get_event(state, event_id).await.map(|event| event.game_id)
}

/// Get all events with sorting (most recent first for admin interface)
pub async fn get_all_events(state: &GameHubState) -> Vec<Event> {
    let event_ids = match state.events.indices().await {
        Ok(indices) => indices,
        Err(_) => return Vec::new(),
    };
    
    let mut events = Vec::new();
    for event_id in event_ids {
        if let Ok(Some(event)) = state.events.get(&event_id).await {
            events.push(event);
        }
    }
    
    // Sort events by start time (most recent first)
    events.sort_by(|a, b| b.start_time.micros().cmp(&a.start_time.micros()));
    events
}

/// Get all events for a specific game
pub async fn get_events_by_game(state: &GameHubState, game_id: &str) -> Vec<Event> {
    let event_ids = match state.events.indices().await {
        Ok(indices) => indices,
        Err(_) => return Vec::new(),
    };
    
    let mut events = Vec::new();
    for event_id in event_ids {
        if let Ok(Some(event)) = state.events.get(&event_id).await {
            if event.game_id == game_id {
                events.push(event);
            }
        }
    }
    
    // Sort events by start time (most recent first)
    events.sort_by(|a, b| b.start_time.micros().cmp(&a.start_time.micros()));
    events
}

/// Get events by status
pub async fn get_events_by_status(state: &GameHubState, status: EventStatus) -> Vec<Event> {
    let event_ids = match state.events.indices().await {
        Ok(indices) => indices,
        Err(_) => return Vec::new(),
    };
    
    let mut events = Vec::new();
    for event_id in event_ids {
        if let Ok(Some(event)) = state.events.get(&event_id).await {
            if event.status == status {
                events.push(event);
            }
        }
    }
    
    // Sort events by start time (most recent first)
    events.sort_by(|a, b| b.start_time.micros().cmp(&a.start_time.micros()));
    events
}

/// Create a new event with admin permission validation
pub async fn create_event(
    state: &mut GameHubState,
    caller_discord_id: &str,
    game_id: &str,
    name: &str,
    description: &str,
    start_time: Timestamp,
    end_time: Timestamp,
    is_mandatory: bool,
    max_participants: Option<u32>,
    prize_pool: Option<u64>,
    timestamp: Timestamp,
) -> Result<Event, GameHubError> {
    // Validate admin permissions
    super::admin_state::validate_admin_permission(state, caller_discord_id).await?;

    // Validate that the game exists and is approved
    if !state.games.get(game_id).await.ok().flatten().is_some() {
        return Err(GameHubError::GameNotFound {
            game_id: game_id.to_string(),
        });
    }

    // Validate time parameters
    if start_time >= end_time {
        return Err(GameHubError::InvalidTimeRange);
    }

    // Generate event ID
    let event_id = format!("event_{}_{}", game_id, timestamp.micros());

    // Create the event
    let event = Event {
        id: event_id.clone(),
        game_id: game_id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        start_time,
        end_time,
        is_mandatory,
        is_mandatory_for_streak: is_mandatory, // Set both fields for compatibility
        grace_period_hours: state.scoring_config.get().grace_period_hours as u32,
        max_participants,
        prize_pool,
        participant_count: 0,
        status: EventStatus::Upcoming,
        created_at: timestamp,
        created_by: caller_discord_id.to_string(),
    };

    // Store the event
    state.events.insert(&event_id, event.clone()).map_err(|_| GameHubError::StorageError)?;

    // Add audit log entry
    super::admin_state::add_audit_log_entry(
        state,
        AdminAction::EventCreated { 
            event_id: event_id.clone(),
            game_id: game_id.to_string(),
        },
        caller_discord_id,
        Some(&event_id),
        Some(&format!("Event '{}' created for game {}", name, game_id)),
        timestamp,
    ).await?;

    Ok(event)
}

/// Update an existing event with admin permission validation
pub async fn update_event(
    state: &mut GameHubState,
    caller_discord_id: &str,
    event_id: &str,
    name: Option<&str>,
    description: Option<&str>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    is_mandatory: Option<bool>,
    max_participants: Option<u32>,
    prize_pool: Option<u64>,
    timestamp: Timestamp,
) -> Result<(), GameHubError> {
    // Validate admin permissions
    super::admin_state::validate_admin_permission(state, caller_discord_id).await?;

    // Get the existing event
    let mut event = get_event(state, event_id).await
        .ok_or_else(|| GameHubError::EventNotFound {
            event_id: event_id.to_string(),
        })?;

    // Update fields if provided
    if let Some(name) = name {
        event.name = name.to_string();
    }
    if let Some(description) = description {
        event.description = description.to_string();
    }
    if let Some(start_time) = start_time {
        event.start_time = start_time;
    }
    if let Some(end_time) = end_time {
        event.end_time = end_time;
    }
    if let Some(is_mandatory) = is_mandatory {
        event.is_mandatory = is_mandatory;
        event.is_mandatory_for_streak = is_mandatory; // Update both fields
    }
    if let Some(max_participants) = max_participants {
        event.max_participants = Some(max_participants);
    }
    if let Some(prize_pool) = prize_pool {
        event.prize_pool = Some(prize_pool);
    }

    // Validate time consistency
    if event.start_time >= event.end_time {
        return Err(GameHubError::InvalidTimeRange);
    }

    // Store the updated event
    state.events.insert(event_id, event).map_err(|_| GameHubError::StorageError)?;

    // Add audit log entry
    super::admin_state::add_audit_log_entry(
        state,
        AdminAction::EventUpdated { 
            event_id: event_id.to_string(),
        },
        caller_discord_id,
        Some(event_id),
        Some(&format!("Event '{}' updated", event_id)),
        timestamp,
    ).await?;

    Ok(())
}

/// Set the mandatory status of an event for streak control
pub async fn set_event_mandatory(
    state: &mut GameHubState,
    caller_discord_id: &str,
    event_id: &str,
    is_mandatory: bool,
    timestamp: Timestamp,
) -> Result<(), GameHubError> {
    // Validate admin permissions
    super::admin_state::validate_admin_permission(state, caller_discord_id).await?;

    // Get the existing event
    let mut event = get_event(state, event_id).await
        .ok_or_else(|| GameHubError::EventNotFound {
            event_id: event_id.to_string(),
        })?;

    // Update mandatory status
    event.is_mandatory = is_mandatory;
    event.is_mandatory_for_streak = is_mandatory; // Update both fields

    // Store the updated event
    state.events.insert(event_id, event).map_err(|_| GameHubError::StorageError)?;

    // Add audit log entry
    super::admin_state::add_audit_log_entry(
        state,
        AdminAction::EventUpdated { 
            event_id: event_id.to_string(),
        },
        caller_discord_id,
        Some(event_id),
        Some(&format!("Event '{}' mandatory status set to {}", event_id, is_mandatory)),
        timestamp,
    ).await?;

    Ok(())
}