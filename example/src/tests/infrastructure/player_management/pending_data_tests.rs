// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Pending player data tests
//! 
//! Tests for pending player data creation, merging, and event score handling.

#[cfg(test)]
mod tests {
    use crate::{
        core::types::{PendingPlayerData, EventScore},
        tests::helpers::*,
    };

    #[test]
    fn test_pending_player_data_creation() {
        // Test PendingPlayerData struct creation
        let points = 100;
        let pending_data = create_test_pending_data(VALID_DISCORD_ID, points);
        
        assert_eq!(pending_data.discord_id, VALID_DISCORD_ID);
        assert_eq!(pending_data.total_pending_points, points);
        assert_eq!(pending_data.event_scores.len(), 1);
        assert!(pending_data.first_activity.micros() > 0);
    }

    #[test]
    fn test_pending_data_with_multiple_events() {
        // Test pending data with multiple event scores
        let mut pending_data = create_test_pending_data(VALID_DISCORD_ID, 50);
        
        // Add additional event scores
        pending_data.event_scores.push(EventScore {
            event_id: "test_event_2".to_string(),
            game_id: "test_game_2".to_string(),
            score: 75,
            participation_timestamp: test_timestamp(),
            streak_eligible: true,
        });
        
        pending_data.event_scores.push(EventScore {
            event_id: "test_event_3".to_string(),
            game_id: "test_game_3".to_string(),
            score: 25,
            participation_timestamp: later_test_timestamp(),
            streak_eligible: false,
        });
        
        // Update total points
        pending_data.total_pending_points = 150;
        
        assert_eq!(pending_data.event_scores.len(), 3);
        assert_eq!(pending_data.total_pending_points, 150);
        
        // Verify all event scores
        assert_eq!(pending_data.event_scores[0].score, 50);
        assert_eq!(pending_data.event_scores[1].score, 75);
        assert_eq!(pending_data.event_scores[2].score, 25);
    }

    #[test]
    fn test_event_score_creation() {
        // Test EventScore struct creation
        let event_score = EventScore {
            event_id: "tournament_1".to_string(),
            game_id: "tournament_game".to_string(),
            score: 200,
            participation_timestamp: test_timestamp(),
            streak_eligible: true,
        };
        
        assert_eq!(event_score.event_id, "tournament_1");
        assert_eq!(event_score.score, 200);
        assert!(event_score.participation_timestamp.micros() > 0);
    }

    #[test]
    fn test_pending_data_chronological_order() {
        // Test that event scores can be ordered chronologically
        let mut pending_data = PendingPlayerData {
            discord_id: VALID_DISCORD_ID.to_string(),
            total_pending_points: 150,
            event_scores: vec![
                EventScore {
                    event_id: "event_3".to_string(),
                    game_id: "game_3".to_string(),
                    score: 50,
                    participation_timestamp: later_test_timestamp(),
                    streak_eligible: true,
                },
                EventScore {
                    event_id: "event_1".to_string(),
                    game_id: "game_1".to_string(),
                    score: 75,
                    participation_timestamp: earlier_test_timestamp(),
                    streak_eligible: true,
                },
                EventScore {
                    event_id: "event_2".to_string(),
                    game_id: "game_2".to_string(),
                    score: 25,
                    participation_timestamp: test_timestamp(),
                    streak_eligible: false,
                },
            ],
            first_activity: test_timestamp(),
        };
        
        // Sort by timestamp
        pending_data.event_scores.sort_by_key(|score| score.participation_timestamp);
        
        // Verify chronological order
        assert_eq!(pending_data.event_scores[0].event_id, "event_1");
        assert_eq!(pending_data.event_scores[1].event_id, "event_2");
        assert_eq!(pending_data.event_scores[2].event_id, "event_3");
    }

    #[test]
    fn test_pending_data_total_pending_points_calculation() {
        // Test that total points matches sum of event scores
        let event_scores = vec![
            EventScore {
                event_id: "event_1".to_string(),
                game_id: "game_1".to_string(),
                score: 50,
                participation_timestamp: test_timestamp(),
                streak_eligible: true,
            },
            EventScore {
                event_id: "event_2".to_string(),
                game_id: "game_2".to_string(),
                score: 75,
                participation_timestamp: test_timestamp(),
                streak_eligible: true,
            },
            EventScore {
                event_id: "event_3".to_string(),
                game_id: "game_3".to_string(),
                score: 25,
                participation_timestamp: test_timestamp(),
                streak_eligible: false,
            },
        ];
        
        let total_from_events: u64 = event_scores.iter().map(|s| s.score).sum();
        
        let pending_data = PendingPlayerData {
            discord_id: VALID_DISCORD_ID.to_string(),
            total_pending_points: total_from_events,
            event_scores,
            first_activity: test_timestamp(),
        };
        
        assert_eq!(pending_data.total_pending_points, 150);
        assert_eq!(total_from_events, 150);
    }

    #[test]
    fn test_pending_data_empty_events() {
        // Test pending data with no event scores
        let pending_data = PendingPlayerData {
            discord_id: VALID_DISCORD_ID.to_string(),
            total_pending_points: 0,
            event_scores: vec![],
            first_activity: test_timestamp(),
        };
        
        assert_eq!(pending_data.discord_id, VALID_DISCORD_ID);
        assert_eq!(pending_data.total_pending_points, 0);
        assert_eq!(pending_data.event_scores.len(), 0);
        assert!(pending_data.first_activity.micros() > 0);
    }

    #[test]
    fn test_event_score_different_timestamps() {
        // Test event scores with different participation timestamps
        let early_event = EventScore {
            event_id: "early_event".to_string(),
            game_id: "early_game".to_string(),
            score: 100,
            participation_timestamp: earlier_test_timestamp(),
            streak_eligible: true,
        };
        
        let late_event = EventScore {
            event_id: "late_event".to_string(),
            game_id: "late_game".to_string(),
            score: 200,
            participation_timestamp: later_test_timestamp(),
            streak_eligible: true,
        };
        
        assert!(early_event.participation_timestamp < late_event.participation_timestamp);
        assert_eq!(early_event.score, 100);
        assert_eq!(late_event.score, 200);
    }

    #[test]
    fn test_pending_data_field_types() {
        // Test that all PendingPlayerData fields have correct types
        let pending_data = create_test_pending_data(VALID_DISCORD_ID, 100);
        
        // String field
        assert!(pending_data.discord_id.len() > 0);
        
        // Numeric field
        assert!(pending_data.total_pending_points >= 0);
        
        // Vector field
        assert!(pending_data.event_scores.len() >= 0);
        
        // Timestamp field
        assert!(pending_data.first_activity.micros() > 0);
        
        // Verify event score types
        for score in &pending_data.event_scores {
            assert!(score.event_id.len() > 0);
            assert!(score.score >= 0);
            assert!(score.participation_timestamp.micros() > 0);
        }
    }
}