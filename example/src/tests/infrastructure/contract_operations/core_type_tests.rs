// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Core type tests
//! 
//! Tests for core data structures, helper functions, and type operations.

#[cfg(test)]
mod tests {
    use crate::core::types::{DeveloperInfo, PendingGame, Player, PlayerStatus};
    use linera_sdk::linera_base_types::Timestamp;

    // Helper function to create a test timestamp
    fn test_timestamp() -> Timestamp {
        Timestamp::from(1000000)
    }

    // Helper function to create a test player
    fn create_test_player(discord_id: &str, username: &str) -> Player {
        Player {
            discord_id: discord_id.to_string(),
            username: username.to_string(),
            avatar_url: None,
            total_points: 0,
            participation_streak: 0,
            best_streak: 0,
            current_rank: None,
            status: PlayerStatus::Active,
            created_at: test_timestamp(),
            last_active: test_timestamp(),
        }
    }

    // Helper function to create a test pending game
    fn create_test_pending_game(id: &str, name: &str) -> PendingGame {
        PendingGame {
            id: id.to_string(),
            name: name.to_string(),
            description: "Test game description".to_string(),
            contract_address: format!("0x{}", "1234567890abcdef".repeat(5)),
            developer_info: DeveloperInfo {
                name: "Test Developer".to_string(),
                contact: "dev@example.com".to_string(),
            },
            created_at: test_timestamp(),
        }
    }

    #[test]
    fn test_helper_functions() {
        // Test helper functions work correctly
        let player = create_test_player("123456789012345678", "TestUser");
        assert_eq!(player.discord_id, "123456789012345678");
        assert_eq!(player.username, "TestUser");
        assert_eq!(player.status, PlayerStatus::Active);
        assert_eq!(player.total_points, 0);
        assert_eq!(player.participation_streak, 0);

        let pending_game = create_test_pending_game("game123", "Test Game");
        assert_eq!(pending_game.id, "game123");
        assert_eq!(pending_game.name, "Test Game");
        assert_eq!(pending_game.description, "Test game description");
        assert_eq!(pending_game.developer_info.name, "Test Developer");
        assert_eq!(pending_game.developer_info.contact, "dev@example.com");
    }

    #[test]
    fn test_player_status_types() {
        // Test different player status types
        let active_player = Player {
            discord_id: "111111111111111111".to_string(),
            username: "ActivePlayer".to_string(),
            avatar_url: None,
            total_points: 100,
            participation_streak: 5,
            best_streak: 7,
            current_rank: Some(1),
            status: PlayerStatus::Active,
            created_at: test_timestamp(),
            last_active: test_timestamp(),
        };

        let suspended_player = Player {
            discord_id: "222222222222222222".to_string(),
            username: "SuspendedPlayer".to_string(),
            avatar_url: None,
            total_points: 50,
            participation_streak: 0,
            best_streak: 2,
            current_rank: None,
            status: PlayerStatus::Suspended {
                reason: "Test suspension".to_string(),
                until: Some(test_timestamp()),
            },
            created_at: test_timestamp(),
            last_active: test_timestamp(),
        };

        let banned_player = Player {
            discord_id: "333333333333333333".to_string(),
            username: "BannedPlayer".to_string(),
            avatar_url: None,
            total_points: 0,
            participation_streak: 0,
            best_streak: 0,
            current_rank: None,
            status: PlayerStatus::Banned {
                reason: "Test ban".to_string(),
            },
            created_at: test_timestamp(),
            last_active: test_timestamp(),
        };

        // Verify status types
        assert!(matches!(active_player.status, PlayerStatus::Active));
        assert!(matches!(suspended_player.status, PlayerStatus::Suspended { .. }));
        assert!(matches!(banned_player.status, PlayerStatus::Banned { .. }));
    }

    #[test]
    fn test_developer_info_structure() {
        // Test developer info structure
        let dev_info = DeveloperInfo {
            name: "Awesome Game Studio".to_string(),
            contact: "studio@awesomegames.com".to_string(),
        };

        assert_eq!(dev_info.name, "Awesome Game Studio");
        assert_eq!(dev_info.contact, "studio@awesomegames.com");
    }

    #[test]
    fn test_timestamp_operations() {
        // Test timestamp operations
        let ts1 = test_timestamp();
        let ts2 = Timestamp::from(2000000);
        
        assert_eq!(ts1.micros(), 1000000);
        assert_eq!(ts2.micros(), 2000000);
        assert!(ts2.micros() > ts1.micros());
    }
}