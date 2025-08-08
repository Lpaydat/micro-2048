// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Test helper functions for leaderboard calculation tests
//! 
//! Shared utility functions used across leaderboard test modules.

use crate::core::types::*;
use linera_sdk::linera_base_types::Timestamp;

/// Create a test leaderboard entry with specified parameters
pub fn create_test_leaderboard_entry(
    player_discord_id: &str,
    score: u64,
    rank: u32,
    participation_timestamp: Option<linera_sdk::linera_base_types::Timestamp>,
) -> LeaderboardEntry {
    let timestamp = participation_timestamp.unwrap_or_else(|| Timestamp::from(1000000));
    LeaderboardEntry {
        player_discord_id: player_discord_id.to_string(),
        score,
        rank,
        participation_data: ParticipationData {
            streak_level: 1,
            streak_multiplier: 100,
            total_points_earned: score,
            participation_timestamp: timestamp,
        },
    }
}

/// Create a test player event update with specified parameters
pub fn create_test_player_event_update(
    discord_id: &str,
    score: u64,
    streak_eligible: bool,
    participation_timestamp: Option<linera_sdk::linera_base_types::Timestamp>,
) -> PlayerEventUpdate {
    PlayerEventUpdate {
        discord_id: discord_id.to_string(),
        score,
        streak_eligible,
        participation_timestamp: participation_timestamp.unwrap_or_else(|| Timestamp::from(1000000)),
    }
}