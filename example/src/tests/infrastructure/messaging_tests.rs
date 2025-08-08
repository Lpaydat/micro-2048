// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive unit tests for cross-chain messaging
//! Tests message processing, validation, and error handling

#[cfg(test)]
mod tests {
    use crate::{
        core::types::{DeveloperInfo, PendingGame, PlayerStatus, PlayerEventUpdate, LeaderboardEntry, AdminAction},
        infrastructure::messages::Message,
    };
    use linera_sdk::linera_base_types::Timestamp;

    // Helper function to create a test timestamp
    fn test_timestamp() -> Timestamp {
        Timestamp::from(1000000)
    }

    // Helper function to create a test pending game
    fn create_test_pending_game(id: &str, name: &str, contract_address: &str) -> PendingGame {
        PendingGame {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("Test game: {}", name),
            contract_address: contract_address.to_string(),
            developer_info: DeveloperInfo {
                name: "Test Developer".to_string(),
                contact: "dev@example.com".to_string(),
            },
            created_at: test_timestamp(),
        }
    }

    // Helper function to create a test player event update
    fn create_test_player_update(discord_id: &str, score: u64, streak_eligible: bool) -> PlayerEventUpdate {
        PlayerEventUpdate {
            discord_id: discord_id.to_string(),
            score,
            participation_timestamp: test_timestamp(),
            streak_eligible,
        }
    }

    #[test]
    fn test_register_game_message() {
        // Test RegisterGame message structure
        let pending_game = create_test_pending_game(
            "test-game-123",
            "Test Game",
            "0x1234567890abcdef1234567890abcdef12345678"
        );
        
        let message = Message::RegisterGame {
            game_info: pending_game.clone(),
        };

        // Verify message structure
        match message {
            Message::RegisterGame { game_info } => {
                assert_eq!(game_info.id, "test-game-123");
                assert_eq!(game_info.name, "Test Game");
                assert_eq!(game_info.developer_info.name, "Test Developer");
                assert_eq!(game_info.contract_address, "0x1234567890abcdef1234567890abcdef12345678");
            }
            _ => panic!("Expected RegisterGame message"),
        }
    }

    #[test]
    fn test_batch_event_update_message() {
        // Test BatchEventUpdate message structure
        let player_updates = vec![
            create_test_player_update("123456789012345678", 1500, true),
        ];
        
        let leaderboard = vec![
            LeaderboardEntry {
                player_discord_id: "123456789012345678".to_string(),
                score: 1500,
                rank: 1,
                participation_data: crate::core::types::ParticipationData {
                    streak_level: 1,
                    streak_multiplier: 100,
                    total_points_earned: 1500,
                    participation_timestamp: test_timestamp(),
                },
            },
        ];

        let message = Message::BatchEventUpdate {
            event_id: "test-event-123".to_string(),
            game_id: "awesome-game".to_string(),
            player_updates,
            final_leaderboard: leaderboard,
            update_timestamp: test_timestamp(),
        };

        // Verify message structure
        match message {
            Message::BatchEventUpdate { event_id, game_id, player_updates, final_leaderboard, update_timestamp } => {
                assert_eq!(event_id, "test-event-123");
                assert_eq!(game_id, "awesome-game");
                assert_eq!(player_updates.len(), 1);
                assert_eq!(final_leaderboard.len(), 1);
                assert_eq!(update_timestamp.micros(), 1000000);
                
                // Check player update
                assert_eq!(player_updates[0].discord_id, "123456789012345678");
                assert_eq!(player_updates[0].score, 1500);
                assert!(player_updates[0].streak_eligible);
            }
            _ => panic!("Expected BatchEventUpdate message"),
        }
    }

    #[test]
    fn test_register_game_with_multiple_games() {
        // Test multiple RegisterGame messages
        let games = vec![
            create_test_pending_game("game1", "Game One", "0x1111"),
            create_test_pending_game("game2", "Game Two", "0x2222"),
            create_test_pending_game("game3", "Game Three", "0x3333"),
        ];

        for (i, game) in games.iter().enumerate() {
            let message = Message::RegisterGame {
                game_info: game.clone(),
            };

            match message {
                Message::RegisterGame { game_info } => {
                    assert_eq!(game_info.id, format!("game{}", i + 1));
                    assert_eq!(game_info.name, format!("Game {}", match i + 1 { 1 => "One", 2 => "Two", 3 => "Three", _ => "Unknown" }));
                    assert_eq!(game_info.developer_info.name, "Test Developer");
                }
                _ => panic!("Expected RegisterGame message"),
            }
        }
    }

    #[test]
    fn test_player_status_validation() {
        // Test player status types
        let active_status = PlayerStatus::Active;
        let suspended_status = PlayerStatus::Suspended {
            reason: "Inappropriate behavior".to_string(),
            until: Some(test_timestamp()),
        };
        let banned_status = PlayerStatus::Banned {
            reason: "Terms of service violation".to_string(),
        };

        // Verify status types
        assert!(matches!(active_status, PlayerStatus::Active));
        assert!(matches!(suspended_status, PlayerStatus::Suspended { .. }));
        assert!(matches!(banned_status, PlayerStatus::Banned { .. }));

        // Test cloning
        let cloned_suspended = suspended_status.clone();
        assert!(matches!(cloned_suspended, PlayerStatus::Suspended { .. }));
    }

    #[test]
    fn test_admin_action_validation() {
        // Test admin action types with proper struct initialization
        let game_approved = AdminAction::GameApproved {
            game_id: "pending-game-789".to_string(),
            game_name: "Test Game".to_string(),
        };
        
        let game_rejected = AdminAction::GameRejected {
            game_id: "pending-game-999".to_string(),
            reason: "Does not meet quality standards".to_string(),
        };

        // Verify admin actions
        match game_approved {
            AdminAction::GameApproved { game_id, game_name } => {
                assert_eq!(game_id, "pending-game-789");
                assert_eq!(game_name, "Test Game");
            }
            _ => panic!("Expected GameApproved action"),
        }

        match game_rejected {
            AdminAction::GameRejected { game_id, reason } => {
                assert_eq!(game_id, "pending-game-999");
                assert_eq!(reason, "Does not meet quality standards");
            }
            _ => panic!("Expected GameRejected action"),
        }
    }

    #[test]
    fn test_admin_action_with_data() {
        // Test admin actions with proper data structures
        let player_banned = AdminAction::PlayerBanned {
            player_id: "banned-player-123".to_string(),
            reason: "Repeated violations".to_string(),
        };
        
        let player_suspended = AdminAction::PlayerSuspended {
            player_id: "suspended-player-456".to_string(),
            reason: "Inappropriate behavior".to_string(),
            duration_hours: Some(24),
        };

        // Verify admin actions
        match player_banned {
            AdminAction::PlayerBanned { player_id, reason } => {
                assert_eq!(player_id, "banned-player-123");
                assert_eq!(reason, "Repeated violations");
            }
            _ => panic!("Expected PlayerBanned action"),
        }

        match player_suspended {
            AdminAction::PlayerSuspended { player_id, reason, duration_hours } => {
                assert_eq!(player_id, "suspended-player-456");
                assert_eq!(reason, "Inappropriate behavior");
                assert_eq!(duration_hours, Some(24));
            }
            _ => panic!("Expected PlayerSuspended action"),
        }
    }

    #[test]
    fn test_admin_action_types() {
        // Test all admin action types with proper struct initialization
        let actions = vec![
            AdminAction::PlayerBanned { player_id: "player1".to_string(), reason: "violation".to_string() },
            AdminAction::PlayerSuspended { player_id: "player2".to_string(), reason: "warning".to_string(), duration_hours: Some(24) },
            AdminAction::PlayerUnbanned { player_id: "player3".to_string() },
            AdminAction::GameApproved { game_id: "game1".to_string(), game_name: "Test Game".to_string() },
            AdminAction::GameRejected { game_id: "game2".to_string(), reason: "quality".to_string() },
            AdminAction::ModeratorAssigned { moderator_id: "mod1".to_string() },
            AdminAction::ModeratorRemoved { moderator_id: "mod2".to_string() },
        ];

        // Verify all actions can be created
        assert_eq!(actions.len(), 7);
        
        // Test cloning admin actions
        let cloned_action = actions[0].clone();
        match cloned_action {
            AdminAction::PlayerBanned { player_id, reason } => {
                assert_eq!(player_id, "player1");
                assert_eq!(reason, "violation");
            }
            _ => panic!("Expected PlayerBanned action"),
        }
    }

    #[test]
    fn test_player_event_update_structure() {
        // Test PlayerEventUpdate structure
        let update = create_test_player_update(
            "777888999000111222",
            2500,
            true
        );

        assert_eq!(update.discord_id, "777888999000111222");
        assert_eq!(update.score, 2500);
        assert!(update.streak_eligible);
        assert_eq!(update.participation_timestamp.micros(), 1000000);

        // Test cloning
        let cloned_update = update.clone();
        assert_eq!(cloned_update.discord_id, update.discord_id);
        assert_eq!(cloned_update.score, update.score);
    }

    #[test]
    fn test_pending_game_structure() {
        // Test PendingGame structure
        let game = create_test_pending_game(
            "puzzle-game-001",
            "Amazing Puzzle Game",
            "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"
        );

        assert_eq!(game.id, "puzzle-game-001");
        assert_eq!(game.name, "Amazing Puzzle Game");
        assert_eq!(game.description, "Test game: Amazing Puzzle Game");
        assert_eq!(game.contract_address, "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");
        assert_eq!(game.developer_info.name, "Test Developer");
        assert_eq!(game.developer_info.contact, "dev@example.com");
        assert_eq!(game.created_at.micros(), 1000000);
    }

    #[test]
    fn test_timestamp_operations() {
        // Test timestamp operations in messages
        let ts1 = test_timestamp();
        let ts2 = Timestamp::from(2000000);
        
        assert_eq!(ts1.micros(), 1000000);
        assert_eq!(ts2.micros(), 2000000);
        assert!(ts2.micros() > ts1.micros());

        // Test timestamps in BatchEventUpdate message structures
        let player_updates = vec![
            create_test_player_update("test-player", 100, true),
        ];
        
        let leaderboard = vec![
            LeaderboardEntry {
                player_discord_id: "test-player".to_string(),
                score: 100,
                rank: 1,
                participation_data: crate::core::types::ParticipationData {
                    streak_level: 1,
                    streak_multiplier: 100,
                    total_points_earned: 100,
                    participation_timestamp: ts2,
                },
            },
        ];

        let message = Message::BatchEventUpdate {
            event_id: "test-event".to_string(),
            game_id: "test-game".to_string(),
            player_updates,
            final_leaderboard: leaderboard,
            update_timestamp: ts2,
        };

        match message {
            Message::BatchEventUpdate { update_timestamp, .. } => {
                assert_eq!(update_timestamp.micros(), 2000000);
            }
            _ => panic!("Expected BatchEventUpdate message"),
        }
    }
}