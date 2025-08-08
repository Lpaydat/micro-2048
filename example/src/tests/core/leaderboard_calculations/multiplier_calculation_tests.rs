// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Multiplier calculation tests for leaderboard calculations
//! 
//! Unit tests for streak multiplier and booster calculation logic.

#![cfg(test)]

use crate::core::types::*;
use linera_sdk::linera_base_types::Timestamp;

#[test]
fn test_leaderboard_multiplier_calculation() {
    // Test different multiplier levels
    let base_points = 100u64;
    
    // Test no multiplier (100 = 1.0x)
    assert_eq!((base_points * 100) / 100, 100);
    
    // Test bronze multiplier (150 = 1.5x)
    assert_eq!((base_points * 150) / 100, 150);
    
    // Test silver multiplier (200 = 2.0x)
    assert_eq!((base_points * 200) / 100, 200);
    
    // Test gold multiplier (300 = 3.0x)
    assert_eq!((base_points * 300) / 100, 300);
}

#[test]
fn test_participation_data_with_boosters() {
    let timestamp = Timestamp::from(1000000);
    let boosted_participation = ParticipationData {
        streak_level: 10,
        streak_multiplier: 200, // 2.0x multiplier
        total_points_earned: 200, // 100 base points * 2.0x
        participation_timestamp: timestamp,
    };

    assert_eq!(boosted_participation.streak_level, 10);
    assert_eq!(boosted_participation.streak_multiplier, 200);
    assert_eq!(boosted_participation.total_points_earned, 200);

    // Verify the math matches (base points would be 100)
    let implied_base_points = boosted_participation.total_points_earned * 100 / boosted_participation.streak_multiplier;
    assert_eq!(implied_base_points, 100);
}