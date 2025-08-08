// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Data structure tests for leaderboard calculations
//! 
//! Unit tests for basic leaderboard data structure creation and validation.

#![cfg(test)]

use crate::core::types::*;
use linera_sdk::linera_base_types::Timestamp;
use super::test_helpers::*;

#[test]
fn test_leaderboard_entry_creation() {
    let entry = create_test_leaderboard_entry(
        "player123456789012345",
        150,
        1,
        None,
    );

    assert_eq!(entry.player_discord_id, "player123456789012345");
    assert_eq!(entry.score, 150);
    assert_eq!(entry.rank, 1);
    assert_eq!(entry.participation_data.streak_level, 1);
    assert_eq!(entry.participation_data.streak_multiplier, 100);
    assert_eq!(entry.participation_data.total_points_earned, 150);
}

#[test]
fn test_player_event_update_creation() {
    let update = create_test_player_event_update(
        "player123456789012345",
        100,
        true,
        None,
    );

    assert_eq!(update.discord_id, "player123456789012345");
    assert_eq!(update.score, 100);
    assert_eq!(update.streak_eligible, true);
}

#[test]
fn test_participation_data_structure() {
    let timestamp = Timestamp::from(1000000);
    let participation_data = ParticipationData {
        streak_level: 5,
        streak_multiplier: 150, // 1.5x
        total_points_earned: 150,
        participation_timestamp: timestamp,
    };

    assert_eq!(participation_data.streak_level, 5);
    assert_eq!(participation_data.streak_multiplier, 150);
    assert_eq!(participation_data.total_points_earned, 150);
    assert_eq!(participation_data.participation_timestamp, timestamp);
}