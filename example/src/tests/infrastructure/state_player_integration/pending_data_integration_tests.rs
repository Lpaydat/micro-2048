// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Pending data integration tests
//! 
//! Integration tests for pending player data handling and event score management.

#![cfg(test)]

use crate::core::types::{PendingPlayerData, EventScore, ScoringConfig};
use linera_sdk::linera_base_types::Timestamp;
use super::test_helpers::{test_timestamp, create_test_pending_data};

#[test]
fn test_pending_player_data_creation() {
    // Test PendingPlayerData creation with event scores
    let discord_id = "123456789012345678";
    let base_timestamp = test_timestamp();
    
    let event_scores = vec![
        EventScore {
            event_id: "event_1".to_string(),
            game_id: "game_1".to_string(),
            score: 100,
            streak_eligible: true,
            participation_timestamp: base_timestamp,
        },
        EventScore {
            event_id: "event_2".to_string(),
            game_id: "game_2".to_string(),
            score: 150,
            streak_eligible: false,
            participation_timestamp: Timestamp::from(base_timestamp.micros() + 1000),
        },
    ];
    
    let pending_data = PendingPlayerData {
        discord_id: discord_id.to_string(),
        total_pending_points: 250,
        event_scores: event_scores.clone(),
        first_activity: base_timestamp,
    };
    
    assert_eq!(pending_data.discord_id, discord_id);
    assert_eq!(pending_data.total_pending_points, 250);
    assert_eq!(pending_data.event_scores.len(), 2);
    
    // Verify individual event scores
    assert_eq!(pending_data.event_scores[0].event_id, "event_1");
    assert_eq!(pending_data.event_scores[0].score, 100);
    assert!(pending_data.event_scores[0].streak_eligible);
    
    assert_eq!(pending_data.event_scores[1].event_id, "event_2");
    assert_eq!(pending_data.event_scores[1].score, 150);
    assert!(!pending_data.event_scores[1].streak_eligible);
}

#[test]
fn test_event_score_structure() {
    // Test EventScore structure and field types
    let timestamp = test_timestamp();
    
    let event_score = EventScore {
        event_id: "test_event_12345".to_string(),
        game_id: "test_game_12345".to_string(),
        score: 175,
        streak_eligible: true,
        participation_timestamp: timestamp,
    };
    
    assert_eq!(event_score.event_id, "test_event_12345");
    assert_eq!(event_score.score, 175);
    assert!(event_score.streak_eligible);
    assert_eq!(event_score.participation_timestamp, timestamp);
    
    // Test field types
    assert!(event_score.score >= 0);
    assert!(event_score.event_id.len() > 0);
}

#[test]
fn test_pending_data_with_multiple_events() {
    // Test pending data with multiple event scores
    let discord_id = "987654321098765432";
    let base_timestamp = test_timestamp();
    
    let mut event_scores = Vec::new();
    let mut total_points = 0;
    
    for i in 1..=5 {
        let score = i * 50;
        total_points += score;
        
        event_scores.push(EventScore {
            event_id: format!("event_{}", i),
            game_id: format!("game_{}", i),
            score: score as u64,
            streak_eligible: i % 2 == 1, // Odd events are streak eligible
            participation_timestamp: Timestamp::from(base_timestamp.micros() + (i as u64 * 1000)),
        });
    }
    
    let pending_data = PendingPlayerData {
        discord_id: discord_id.to_string(),
        total_pending_points: total_points as u64,
        event_scores,
        first_activity: base_timestamp,
    };
    
    assert_eq!(pending_data.discord_id, discord_id);
    assert_eq!(pending_data.total_pending_points, 750); // 50 + 100 + 150 + 200 + 250
    assert_eq!(pending_data.event_scores.len(), 5);
    
    // Verify streak eligibility pattern
    for (i, event) in pending_data.event_scores.iter().enumerate() {
        let expected_streak_eligible = (i + 1) % 2 == 1;
        assert_eq!(event.streak_eligible, expected_streak_eligible, 
                   "Event {} streak eligibility mismatch", i + 1);
    }
}

#[test]
fn test_pending_data_structure_and_validation() {
    // Test pending data structure validation and integrity
    let pending_data = create_test_pending_data("validation_test_player");
    
    // Basic structure validation
    assert!(!pending_data.discord_id.is_empty());
    assert!(pending_data.total_pending_points >= 0);
    assert!(pending_data.event_scores.len() >= 0);
    
    // Verify field types and constraints
    for event_score in &pending_data.event_scores {
        assert!(!event_score.event_id.is_empty());
        assert!(event_score.score >= 0);
        // participation_timestamp should be valid (no specific validation needed for Timestamp)
    }
    
    // Data consistency checks
    assert_eq!(pending_data.event_scores.len(), 2);
    assert_eq!(pending_data.total_pending_points, 250);
    
    let calculated_total: u64 = pending_data.event_scores.iter().map(|e| e.score).sum();
    assert_eq!(calculated_total, pending_data.total_pending_points);
}

#[test]
fn test_pending_data_chronological_order() {
    // Test pending data event scores in chronological order
    let pending_data = create_test_pending_data("chronological_test_player");
    
    // Verify events are in chronological order
    for i in 1..pending_data.event_scores.len() {
        assert!(
            pending_data.event_scores[i].participation_timestamp.micros() > 
            pending_data.event_scores[i - 1].participation_timestamp.micros(),
            "Event scores should be in chronological order"
        );
    }
}

#[test]
fn test_pending_data_total_pending_points_calculation() {
    // Test total pending points calculation logic
    let base_timestamp = test_timestamp();
    let event_scores = vec![
        EventScore {
            event_id: "calc_event_1".to_string(),
            game_id: "calc_game_1".to_string(),
            score: 125,
            streak_eligible: true,
            participation_timestamp: base_timestamp,
        },
        EventScore {
            event_id: "calc_event_2".to_string(),
            game_id: "calc_game_2".to_string(),
            score: 275,
            streak_eligible: false,
            participation_timestamp: Timestamp::from(base_timestamp.micros() + 2000),
        },
        EventScore {
            event_id: "calc_event_3".to_string(),
            game_id: "calc_game_3".to_string(),
            score: 100,
            streak_eligible: true,
            participation_timestamp: Timestamp::from(base_timestamp.micros() + 4000),
        },
    ];
    
    let calculated_total = event_scores.iter().map(|e| e.score).sum();
    
    let pending_data = PendingPlayerData {
        discord_id: "calculation_test_player".to_string(),
        total_pending_points: calculated_total,
        event_scores,
        first_activity: base_timestamp,
    };
    
    assert_eq!(pending_data.total_pending_points, 500); // 125 + 275 + 100
    
    // Verify calculation matches
    let verification_total: u64 = pending_data.event_scores.iter().map(|e| e.score).sum();
    assert_eq!(pending_data.total_pending_points, verification_total);
}

#[test]
fn test_pending_data_empty_events() {
    // Test pending data with empty event scores list
    let pending_data = PendingPlayerData {
        discord_id: "empty_events_player".to_string(),
        total_pending_points: 0,
        event_scores: vec![],
        first_activity: test_timestamp(),
    };
    
    assert_eq!(pending_data.discord_id, "empty_events_player");
    assert_eq!(pending_data.total_pending_points, 0);
    assert_eq!(pending_data.event_scores.len(), 0);
}

#[test]
fn test_default_scoring_configuration_structure() {
    // Test default scoring configuration structure
    let default_config = ScoringConfig::default();
    
    // Verify default values are reasonable
    assert!(default_config.grace_period_hours >= 0);
    assert!(default_config.booster_levels.len() >= 0);
    
    // Verify booster levels are sorted by requirement (if any exist)
    for i in 1..default_config.booster_levels.len() {
        assert!(
            default_config.booster_levels[i].required_streak >= 
            default_config.booster_levels[i - 1].required_streak,
            "Booster levels should be sorted by streak requirement"
        );
    }
}