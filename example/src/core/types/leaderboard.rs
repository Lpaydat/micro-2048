// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use linera_sdk::linera_base_types::Timestamp;

/// Leaderboard entry with ranking and participation data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaderboardEntry {
    pub player_discord_id: String,
    pub score: u64,
    pub rank: u32,
    pub participation_data: ParticipationData,
}

/// Participation data for streak tracking and multipliers
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParticipationData {
    pub streak_level: u32,
    pub streak_multiplier: u64, // Multiplier * 100 (e.g., 150 = 1.5x multiplier)
    pub total_points_earned: u64,
    pub participation_timestamp: Timestamp,
}

/// Event data for real-time leaderboard updates
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaderboardUpdateEvent {
    pub event_id: String,
    pub updated_players: Vec<String>, // Discord IDs of players who were updated
    pub unregistered_players: Vec<String>, // Discord IDs of unregistered players
    pub top_players: Vec<LeaderboardEntry>, // Top players for real-time display
    pub update_timestamp: Timestamp,
}