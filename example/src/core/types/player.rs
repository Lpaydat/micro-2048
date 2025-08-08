// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use linera_sdk::linera_base_types::Timestamp;

/// Player status enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PlayerStatus {
    Active,
    Suspended {
        reason: String,
        until: Option<Timestamp>,
    },
    Banned {
        reason: String,
    },
}

/// Enhanced player structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub discord_id: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub total_points: u64,
    pub participation_streak: u32,
    pub best_streak: u32, // Track historical maximum streak
    pub current_rank: Option<u32>,
    pub status: PlayerStatus,
    pub created_at: Timestamp,
    pub last_active: Timestamp,
}

/// Player statistics for queries and optimization
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerStats {
    pub total_points: u64,
    pub participation_streak: u32,
    pub current_rank: Option<u32>,
    pub status: PlayerStatus,
    pub created_at: Timestamp,
    pub last_active: Timestamp,
}

/// Player event update for cross-chain messaging
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerEventUpdate {
    pub discord_id: String,
    pub score: u64,
    pub participation_timestamp: Timestamp,
    pub streak_eligible: bool, // Whether this participation counts for streak
}

/// Event score for pending players
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventScore {
    pub event_id: String,
    pub game_id: String,
    pub score: u64,
    pub participation_timestamp: Timestamp,
    pub streak_eligible: bool,
}

/// Pending player data for unregistered players
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingPlayerData {
    pub discord_id: String,
    pub event_scores: Vec<EventScore>,
    pub total_pending_points: u64,
    pub first_activity: Timestamp,
}