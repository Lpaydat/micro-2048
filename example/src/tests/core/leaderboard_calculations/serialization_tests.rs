// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Serialization tests for leaderboard calculations
//! 
//! Unit tests for JSON serialization and deserialization of leaderboard types.

#![cfg(test)]

use super::test_helpers::*;

#[test]
fn test_leaderboard_entry_serialization() {
    let entry = create_test_leaderboard_entry(
        "player123456789012345",
        150,
        1,
        None,
    );

    // Test that the entry can be serialized (this ensures serde attributes work)
    let serialized = serde_json::to_string(&entry);
    assert!(serialized.is_ok());

    // Test deserialization
    let deserialized: Result<crate::core::types::LeaderboardEntry, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok());

    let deserialized_entry = deserialized.unwrap();
    assert_eq!(entry.player_discord_id, deserialized_entry.player_discord_id);
    assert_eq!(entry.score, deserialized_entry.score);
    assert_eq!(entry.rank, deserialized_entry.rank);
}

#[test]
fn test_player_event_update_serialization() {
    let update = create_test_player_event_update(
        "player123456789012345",
        100,
        true,
        None,
    );

    // Test serialization
    let serialized = serde_json::to_string(&update);
    assert!(serialized.is_ok());

    // Test deserialization
    let deserialized: Result<crate::core::types::PlayerEventUpdate, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok());

    let deserialized_update = deserialized.unwrap();
    assert_eq!(update.discord_id, deserialized_update.discord_id);
    assert_eq!(update.score, deserialized_update.score);
    assert_eq!(update.streak_eligible, deserialized_update.streak_eligible);
}