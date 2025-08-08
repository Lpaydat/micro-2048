// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use linera_sdk::linera_base_types::Timestamp;

/// Event log entry for activity tracking
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameHubEvent {
    pub id: String,
    pub event_type: EventType,
    pub description: String,
    pub actor_id: Option<String>, // Discord ID of who triggered the event
    pub target_id: Option<String>, // ID of the target (player, game, etc.)
    pub timestamp: Timestamp,
    pub metadata: Option<String>, // JSON string for additional data
}

/// Types of events that can occur in GameHub
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EventType {
    PlayerRegistered,
    PlayerBanned,
    PlayerSuspended,
    PlayerUnbanned,
    GameSubmitted,
    GameApproved,
    GameRejected,
    ScoreUpdated,
    AdminAction,
    BatchEventUpdate,
    EventCreated,
    EventUpdated,
    Error,
}

/// Event data structure for tournament/competition management
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub game_id: String,
    pub description: String,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub is_mandatory: bool, // For streak control
    pub is_mandatory_for_streak: bool, // Legacy field - kept for compatibility
    pub grace_period_hours: u32,
    pub max_participants: Option<u32>,
    pub prize_pool: Option<u64>,
    pub participant_count: u32,
    pub created_by: String, // Admin/Game Discord ID
    pub created_at: Timestamp,
    pub status: EventStatus,
}

/// Event status enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EventStatus {
    Upcoming,
    Active,
    Ended,
    Cancelled,
}