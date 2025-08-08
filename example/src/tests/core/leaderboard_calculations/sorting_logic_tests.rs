// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Sorting logic tests for leaderboard calculations
//! 
//! Unit tests for leaderboard sorting algorithms and ranking logic.

#![cfg(test)]

use linera_sdk::linera_base_types::Timestamp;
use super::test_helpers::*;

#[test]
fn test_leaderboard_sorting_logic() {
    let time1 = Timestamp::from(1000000);
    let time2 = linera_sdk::linera_base_types::Timestamp::from(time1.micros() + 1000);
    let time3 = linera_sdk::linera_base_types::Timestamp::from(time1.micros() + 2000);

    let mut entries = vec![
        create_test_leaderboard_entry("player1", 100, 0, Some(time3)),
        create_test_leaderboard_entry("player2", 200, 0, Some(time1)),
        create_test_leaderboard_entry("player3", 200, 0, Some(time2)), // Same score as player2, but later
        create_test_leaderboard_entry("player4", 150, 0, Some(time1)),
    ];

    // Sort by score (descending), then by participation timestamp (ascending for ties)
    entries.sort_by(|a, b| {
        match b.score.cmp(&a.score) {
            std::cmp::Ordering::Equal => a.participation_data.participation_timestamp.micros().cmp(&b.participation_data.participation_timestamp.micros()),
            other => other,
        }
    });

    // Verify sorting
    assert_eq!(entries[0].player_discord_id, "player2"); // Score 200, earlier timestamp
    assert_eq!(entries[1].player_discord_id, "player3"); // Score 200, later timestamp
    assert_eq!(entries[2].player_discord_id, "player4"); // Score 150
    assert_eq!(entries[3].player_discord_id, "player1"); // Score 100
}

#[test]
fn test_leaderboard_ranking_with_ties() {
    let time1 = Timestamp::from(1000000);
    let time2 = linera_sdk::linera_base_types::Timestamp::from(time1.micros() + 1000);
    let time3 = linera_sdk::linera_base_types::Timestamp::from(time1.micros() + 2000);

    let mut entries = vec![
        create_test_leaderboard_entry("player1", 200, 0, Some(time1)),
        create_test_leaderboard_entry("player2", 200, 0, Some(time2)), // Tie for first
        create_test_leaderboard_entry("player3", 150, 0, Some(time3)),
    ];

    // Sort and assign ranks with proper tie handling
    entries.sort_by(|a, b| {
        match b.score.cmp(&a.score) {
            std::cmp::Ordering::Equal => a.participation_data.participation_timestamp.micros().cmp(&b.participation_data.participation_timestamp.micros()),
            other => other,
        }
    });

    // Assign ranks handling ties
    let mut current_rank = 1u32;
    for i in 0..entries.len() {
        if i > 0 && entries[i].score != entries[i-1].score {
            current_rank = (i + 1) as u32;
        }
        entries[i].rank = current_rank;
    }

    // Verify ranking with ties
    assert_eq!(entries[0].rank, 1); // Player1 - rank 1 (earlier timestamp)
    assert_eq!(entries[1].rank, 1); // Player2 - rank 1 (tie, same score)
    assert_eq!(entries[2].rank, 3); // Player3 - rank 3 (skipping rank 2 due to tie)
}

#[test]
fn test_leaderboard_update_with_complex_ties() {
    let time1 = Timestamp::from(1000000);
    let time2 = linera_sdk::linera_base_types::Timestamp::from(time1.micros() + 1000);
    let time3 = linera_sdk::linera_base_types::Timestamp::from(time1.micros() + 2000);
    let time4 = linera_sdk::linera_base_types::Timestamp::from(time1.micros() + 3000);
    let time5 = linera_sdk::linera_base_types::Timestamp::from(time1.micros() + 4000);

    let mut entries = vec![
        create_test_leaderboard_entry("player1", 300, 0, Some(time1)), // 1st place
        create_test_leaderboard_entry("player2", 200, 0, Some(time2)), // Tie for 2nd
        create_test_leaderboard_entry("player3", 200, 0, Some(time3)), // Tie for 2nd
        create_test_leaderboard_entry("player4", 200, 0, Some(time4)), // Tie for 2nd
        create_test_leaderboard_entry("player5", 100, 0, Some(time5)), // 5th place
    ];

    // Sort by score (descending), then by participation timestamp (ascending for ties)
    entries.sort_by(|a, b| {
        match b.score.cmp(&a.score) {
            std::cmp::Ordering::Equal => a.participation_data.participation_timestamp.micros().cmp(&b.participation_data.participation_timestamp.micros()),
            other => other,
        }
    });

    // Assign ranks with proper tie handling and gap calculation
    let mut current_rank = 1u32;
    for i in 0..entries.len() {
        if i > 0 && entries[i].score != entries[i-1].score {
            current_rank = (i + 1) as u32;
        }
        entries[i].rank = current_rank;
    }

    // Verify complex tie handling
    assert_eq!(entries[0].player_discord_id, "player1");
    assert_eq!(entries[0].rank, 1); // 1st place (300 points)

    assert_eq!(entries[1].player_discord_id, "player2");
    assert_eq!(entries[1].rank, 2); // Tied for 2nd (200 points, earliest)

    assert_eq!(entries[2].player_discord_id, "player3");
    assert_eq!(entries[2].rank, 2); // Tied for 2nd (200 points)

    assert_eq!(entries[3].player_discord_id, "player4");
    assert_eq!(entries[3].rank, 2); // Tied for 2nd (200 points, latest)

    assert_eq!(entries[4].player_discord_id, "player5");
    assert_eq!(entries[4].rank, 5); // 5th place (100 points) - skipped ranks 3 and 4 due to 3-way tie
}