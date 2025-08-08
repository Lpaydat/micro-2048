// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for Analytics functionality and GraphQL types

#[cfg(test)]
mod tests {
    use crate::api::graphql_types::{
        AnalyticsObject, DateRangeInput, PlayerEngagementObject, GameStatsObject, ImportResultObject
    };
    use serde_json;

    #[test]
    fn test_date_range_input_structure() {
        let date_range = DateRangeInput {
            start_date: "2024-01-01".to_string(),
            end_date: "2024-12-31".to_string(),
        };

        assert_eq!(date_range.start_date, "2024-01-01");
        assert_eq!(date_range.end_date, "2024-12-31");
    }

    #[test]
    fn test_date_range_input_serialization() {
        let date_range = DateRangeInput {
            start_date: "2024-03-15".to_string(),
            end_date: "2024-06-15".to_string(),
        };

        let serialized = serde_json::to_string(&date_range).expect("Failed to serialize");
        let deserialized: DateRangeInput = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized.start_date, date_range.start_date);
        assert_eq!(deserialized.end_date, date_range.end_date);
    }

    #[test]
    fn test_player_engagement_object_structure() {
        let engagement = PlayerEngagementObject {
            date: "2024-08-06".to_string(),
            active_users: 150,
            new_registrations: 25,
            total_events: 8,
            total_participation: 320,
        };

        assert_eq!(engagement.date, "2024-08-06");
        assert_eq!(engagement.active_users, 150);
        assert_eq!(engagement.new_registrations, 25);
        assert_eq!(engagement.total_events, 8);
        assert_eq!(engagement.total_participation, 320);
    }

    #[test]
    fn test_analytics_object_structure() {
        let engagement_data = vec![
            PlayerEngagementObject {
                date: "2024-08-05".to_string(),
                active_users: 100,
                new_registrations: 15,
                total_events: 5,
                total_participation: 250,
            },
            PlayerEngagementObject {
                date: "2024-08-06".to_string(),
                active_users: 150,
                new_registrations: 25,
                total_events: 8,
                total_participation: 320,
            },
        ];

        let analytics = AnalyticsObject {
            total_players: 1500,
            active_games: 12,
            total_events: 45,
            active_players_in_period: 500,
            new_registrations_in_period: 40,
            player_engagement: engagement_data,
        };

        assert_eq!(analytics.total_players, 1500);
        assert_eq!(analytics.active_games, 12);
        assert_eq!(analytics.total_events, 45);
        assert_eq!(analytics.active_players_in_period, 500);
        assert_eq!(analytics.new_registrations_in_period, 40);
        assert_eq!(analytics.player_engagement.len(), 2);
        assert_eq!(analytics.player_engagement[0].active_users, 100);
        assert_eq!(analytics.player_engagement[1].active_users, 150);
    }

    #[test]
    fn test_game_stats_object_structure() {
        let game_stats = GameStatsObject {
            game_id: "game_123".to_string(),
            game_name: "Space Battle Arena".to_string(),
            total_events: 25,
            total_participants: 450,
            unique_players: 120,
            average_participants_per_event: 18.0,
            last_event_date: Some("2024-08-05".to_string()),
            popularity_score: 85.5,
        };

        assert_eq!(game_stats.game_id, "game_123");
        assert_eq!(game_stats.game_name, "Space Battle Arena");
        assert_eq!(game_stats.total_events, 25);
        assert_eq!(game_stats.total_participants, 450);
        assert_eq!(game_stats.unique_players, 120);
        assert_eq!(game_stats.average_participants_per_event, 18.0);
        assert_eq!(game_stats.last_event_date, Some("2024-08-05".to_string()));
        assert_eq!(game_stats.popularity_score, 85.5);
    }

    #[test]
    fn test_game_stats_popularity_calculation_logic() {
        // Test popularity score calculation based on events and participants
        let high_popularity = GameStatsObject {
            game_id: "popular_game".to_string(),
            game_name: "Popular Game".to_string(),
            total_events: 50,
            total_participants: 1000,
            unique_players: 300,
            average_participants_per_event: 20.0,
            last_event_date: Some("2024-08-06".to_string()),
            popularity_score: (50.0 * 0.3) + (1000.0 * 0.7), // Formula: events * 0.3 + participants * 0.7
        };

        let expected_score = (50.0 * 0.3) + (1000.0 * 0.7);
        assert_eq!(high_popularity.popularity_score, expected_score);
        assert!(high_popularity.popularity_score > 700.0); // Should be high popularity

        let low_popularity = GameStatsObject {
            game_id: "new_game".to_string(),
            game_name: "New Game".to_string(),
            total_events: 3,
            total_participants: 15,
            unique_players: 10,
            average_participants_per_event: 5.0,
            last_event_date: Some("2024-08-01".to_string()),
            popularity_score: (3.0 * 0.3) + (15.0 * 0.7),
        };

        let expected_low_score = (3.0 * 0.3) + (15.0 * 0.7);
        assert_eq!(low_popularity.popularity_score, expected_low_score);
        assert!(low_popularity.popularity_score < 20.0); // Should be low popularity
    }

    #[test]
    fn test_import_result_object_success_case() {
        let import_result = ImportResultObject {
            success: true,
            total_records: 100,
            successful_imports: 95,
            failed_imports: 5,
            errors: vec!["Invalid Discord ID on row 15".to_string(), "Duplicate username on row 42".to_string()],
            summary_message: "Import completed: 95/100 records processed successfully".to_string(),
        };

        assert!(import_result.success);
        assert_eq!(import_result.total_records, 100);
        assert_eq!(import_result.successful_imports, 95);
        assert_eq!(import_result.failed_imports, 5);
        assert_eq!(import_result.errors.len(), 2);
        assert_eq!(import_result.successful_imports + import_result.failed_imports, import_result.total_records);
    }

    #[test]
    fn test_import_result_object_failure_case() {
        let import_result = ImportResultObject {
            success: false,
            total_records: 50,
            successful_imports: 0,
            failed_imports: 50,
            errors: vec!["Invalid CSV format".to_string(), "Missing required columns".to_string()],
            summary_message: "Import failed: Invalid file format detected".to_string(),
        };

        assert!(!import_result.success);
        assert_eq!(import_result.total_records, 50);
        assert_eq!(import_result.successful_imports, 0);
        assert_eq!(import_result.failed_imports, 50);
        assert_eq!(import_result.errors.len(), 2);
        assert!(import_result.summary_message.contains("failed"));
    }

    #[test]
    fn test_analytics_object_serialization() {
        let analytics = AnalyticsObject {
            total_players: 1000,
            active_games: 10,
            total_events: 30,
            active_players_in_period: 300,
            new_registrations_in_period: 50,
            player_engagement: vec![
                PlayerEngagementObject {
                    date: "2024-08-06".to_string(),
                    active_users: 200,
                    new_registrations: 30,
                    total_events: 6,
                    total_participation: 400,
                }
            ],
        };

        let serialized = serde_json::to_string(&analytics).expect("Failed to serialize");
        let deserialized: AnalyticsObject = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized.total_players, analytics.total_players);
        assert_eq!(deserialized.active_games, analytics.active_games);
        assert_eq!(deserialized.player_engagement.len(), 1);
        assert_eq!(deserialized.player_engagement[0].active_users, 200);
    }

    #[test]
    fn test_empty_analytics_data() {
        let empty_analytics = AnalyticsObject {
            total_players: 0,
            active_games: 0,
            total_events: 0,
            active_players_in_period: 0,
            new_registrations_in_period: 0,
            player_engagement: vec![],
        };

        assert_eq!(empty_analytics.total_players, 0);
        assert_eq!(empty_analytics.player_engagement.len(), 0);
    }

    #[test]
    fn test_game_stats_with_no_events() {
        let no_events_stats = GameStatsObject {
            game_id: "inactive_game".to_string(),
            game_name: "Inactive Game".to_string(),
            total_events: 0,
            total_participants: 0,
            unique_players: 0,
            average_participants_per_event: 0.0,
            last_event_date: None,
            popularity_score: 0.0,
        };

        assert_eq!(no_events_stats.total_events, 0);
        assert_eq!(no_events_stats.total_participants, 0);
        assert_eq!(no_events_stats.unique_players, 0);
        assert_eq!(no_events_stats.average_participants_per_event, 0.0);
        assert_eq!(no_events_stats.last_event_date, None);
        assert_eq!(no_events_stats.popularity_score, 0.0);
    }

    #[test]
    fn test_date_range_edge_cases() {
        // Same start and end date
        let same_date_range = DateRangeInput {
            start_date: "2024-08-06".to_string(),
            end_date: "2024-08-06".to_string(),
        };
        assert_eq!(same_date_range.start_date, same_date_range.end_date);

        // Different date formats (should be handled by validation elsewhere)
        let iso_date_range = DateRangeInput {
            start_date: "2024-01-01T00:00:00Z".to_string(),
            end_date: "2024-12-31T23:59:59Z".to_string(),
        };
        assert!(iso_date_range.start_date.contains("T"));
        assert!(iso_date_range.end_date.contains("T"));
    }
}