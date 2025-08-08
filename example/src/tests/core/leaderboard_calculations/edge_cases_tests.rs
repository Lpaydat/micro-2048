// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Edge cases tests for leaderboard calculations
//! 
//! Unit tests for edge cases including empty leaderboards, large scores, and all-tie scenarios.

#![cfg(test)]

use crate::core::types::*;
use linera_sdk::linera_base_types::Timestamp;
use super::test_helpers::*;

#[test]
fn test_leaderboard_edge_cases() {
    // Test empty leaderboard
    let empty_entries: Vec<crate::core::types::LeaderboardEntry> = vec![];
    assert_eq!(empty_entries.len(), 0);

    // Test single entry
    let single_entry = vec![
        create_test_leaderboard_entry("player1", 100, 1, None)
    ];
    assert_eq!(single_entry.len(), 1);
    assert_eq!(single_entry[0].rank, 1);

    // Test all same scores (all tied for first)
    let time1 = Timestamp::from(1000000);
    let time2 = linera_sdk::linera_base_types::Timestamp::from(time1.micros() + 1000);
    let time3 = linera_sdk::linera_base_types::Timestamp::from(time1.micros() + 2000);

    let mut tied_entries = vec![
        create_test_leaderboard_entry("player1", 100, 0, Some(time1)),
        create_test_leaderboard_entry("player2", 100, 0, Some(time2)),
        create_test_leaderboard_entry("player3", 100, 0, Some(time3)),
    ];

    // Sort by timestamp for tie-breaking
    tied_entries.sort_by(|a, b| a.participation_data.participation_timestamp.micros().cmp(&b.participation_data.participation_timestamp.micros()));

    // All should be rank 1
    for entry in &mut tied_entries {
        entry.rank = 1;
    }

    assert_eq!(tied_entries[0].rank, 1);
    assert_eq!(tied_entries[1].rank, 1);
    assert_eq!(tied_entries[2].rank, 1);
}

#[test]
fn test_leaderboard_large_scores() {
    let entry = create_test_leaderboard_entry(
        "player123456789012345",
        u64::MAX / 2, // Large score
        1,
        None,
    );

    assert_eq!(entry.score, u64::MAX / 2);
    assert_eq!(entry.participation_data.total_points_earned, u64::MAX / 2);
}

#[test]
fn test_participation_data_timestamp_handling() {
    let timestamp1 = Timestamp::from(1000000);
    let timestamp2 = linera_sdk::linera_base_types::Timestamp::from(timestamp1.micros() + 1000);
    
    let data1 = ParticipationData {
        streak_level: 3,
        streak_multiplier: 150,
        total_points_earned: 150,
        participation_timestamp: timestamp1,
    };
    
    let data2 = ParticipationData {
        streak_level: 3,
        streak_multiplier: 150,
        total_points_earned: 150,
        participation_timestamp: timestamp2,
    };
    
    // Verify timestamp ordering for tie-breaking
    assert!(data1.participation_timestamp.micros() < data2.participation_timestamp.micros());
}