// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::{
        core::types::{Player, PlayerStatus},
    };
    use linera_sdk::linera_base_types::Timestamp;

    #[test]
    fn test_player_struct_creation() {
        let current_time = Timestamp::from(1000);
        
        let player = Player {
            discord_id: "123456789012345678".to_string(),
            username: "TestPlayer#1234".to_string(),
            avatar_url: Some("https://cdn.discordapp.com/avatars/123456789012345678/avatar.png".to_string()),
            total_points: 0,
            participation_streak: 0,
            best_streak: 0,
            current_rank: None,
            status: PlayerStatus::Active,
            created_at: current_time,
            last_active: current_time,
        };

        assert_eq!(player.discord_id, "123456789012345678");
        assert_eq!(player.username, "TestPlayer#1234");
        assert_eq!(player.total_points, 0);
        assert_eq!(player.participation_streak, 0);
        assert_eq!(player.status, PlayerStatus::Active);
        assert_eq!(player.created_at, current_time);
        assert_eq!(player.last_active, current_time);
        assert!(player.avatar_url.is_some());
        assert!(player.current_rank.is_none());
    }

    #[test]
    fn test_player_with_progression() {
        let create_time = Timestamp::from(1000);
        let update_time = Timestamp::from(2000);
        
        let mut player = Player {
            discord_id: "987654321098765432".to_string(),
            username: "ProgPlayer#5678".to_string(),
            avatar_url: None,
            total_points: 1500,
            participation_streak: 5,
            best_streak: 8,
            current_rank: Some(42),
            status: PlayerStatus::Active,
            created_at: create_time,
            last_active: update_time,
        };

        // Test progression updates
        player.total_points += 500;
        player.participation_streak += 1;
        player.current_rank = Some(35);
        player.last_active = Timestamp::from(3000);

        assert_eq!(player.total_points, 2000);
        assert_eq!(player.participation_streak, 6);
        assert_eq!(player.current_rank, Some(35));
        assert_eq!(player.last_active.micros(), 3000);
        assert!(player.avatar_url.is_none());
    }

    #[test]
    fn test_player_status_variants() {
        // Test Active status
        let active = PlayerStatus::Active;
        assert_eq!(active, PlayerStatus::Active);

        // Test Banned status
        let banned = PlayerStatus::Banned {
            reason: "Cheating detected".to_string(),
        };
        match banned {
            PlayerStatus::Banned { reason } => {
                assert_eq!(reason, "Cheating detected");
            },
            _ => panic!("Expected Banned status"),
        }

        // Test Suspended status with expiration
        let suspended_with_expiry = PlayerStatus::Suspended {
            reason: "Toxic behavior".to_string(),
            until: Some(Timestamp::from(2000)),
        };
        match suspended_with_expiry {
            PlayerStatus::Suspended { reason, until } => {
                assert_eq!(reason, "Toxic behavior");
                assert_eq!(until, Some(Timestamp::from(2000)));
            },
            _ => panic!("Expected Suspended status"),
        }

        // Test Suspended status without expiration (indefinite)
        let suspended_indefinite = PlayerStatus::Suspended {
            reason: "Severe violation".to_string(),
            until: None,
        };
        match suspended_indefinite {
            PlayerStatus::Suspended { reason, until } => {
                assert_eq!(reason, "Severe violation");
                assert_eq!(until, None);
            },
            _ => panic!("Expected Suspended status"),
        }
    }

    #[test]
    fn test_player_status_transitions() {
        let create_time = Timestamp::from(1000);
        
        let mut player = Player {
            discord_id: "111222333444555666".to_string(),
            username: "StatusTest#9876".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            total_points: 750,
            participation_streak: 3,
            best_streak: 3,
            current_rank: Some(20),
            status: PlayerStatus::Active,
            created_at: create_time,
            last_active: create_time,
        };

        // Test suspension
        player.status = PlayerStatus::Suspended {
            reason: "Inappropriate behavior".to_string(),
            until: Some(Timestamp::from(5000)),
        };
        
        match &player.status {
            PlayerStatus::Suspended { reason, until } => {
                assert_eq!(reason, "Inappropriate behavior");
                assert_eq!(until, &Some(Timestamp::from(5000)));
            },
            _ => panic!("Expected Suspended status"),
        }

        // Test reactivation
        player.status = PlayerStatus::Active;
        assert_eq!(player.status, PlayerStatus::Active);

        // Test ban
        player.status = PlayerStatus::Banned {
            reason: "Terms of service violation".to_string(),
        };
        
        match &player.status {
            PlayerStatus::Banned { reason } => {
                assert_eq!(reason, "Terms of service violation");
            },
            _ => panic!("Expected Banned status"),
        }
    }

    #[test]
    fn test_player_rank_management() {
        let current_time = Timestamp::from(1000);
        
        // Test player without rank
        let unranked_player = Player {
            discord_id: "unranked123456789".to_string(),
            username: "UnrankedPlayer#0001".to_string(),
            avatar_url: None,
            total_points: 50,
            participation_streak: 1,
            best_streak: 1,
            current_rank: None,
            status: PlayerStatus::Active,
            created_at: current_time,
            last_active: current_time,
        };
        
        assert!(unranked_player.current_rank.is_none());
        
        // Test player with rank
        let ranked_player = Player {
            discord_id: "ranked987654321".to_string(),
            username: "RankedPlayer#0002".to_string(),
            avatar_url: Some("https://example.com/ranked_avatar.png".to_string()),
            total_points: 2500,
            participation_streak: 10,
            best_streak: 15,
            current_rank: Some(1),
            status: PlayerStatus::Active,
            created_at: current_time,
            last_active: current_time,
        };
        
        assert_eq!(ranked_player.current_rank, Some(1));
        assert_eq!(ranked_player.total_points, 2500);
        assert_eq!(ranked_player.participation_streak, 10);
    }

    #[test]
    fn test_player_discord_integration() {
        let current_time = Timestamp::from(1000);
        
        // Test player with full Discord integration
        let discord_player = Player {
            discord_id: "555666777888999000".to_string(),
            username: "DiscordGamer#1337".to_string(),
            avatar_url: Some("https://cdn.discordapp.com/avatars/555666777888999000/a1b2c3d4e5f6.png".to_string()),
            total_points: 1200,
            participation_streak: 4,
            best_streak: 6,
            current_rank: Some(15),
            status: PlayerStatus::Active,
            created_at: current_time,
            last_active: current_time,
        };

        // Verify Discord ID format (should be 18 digits)
        assert_eq!(discord_player.discord_id.len(), 18);
        assert!(discord_player.discord_id.chars().all(|c| c.is_ascii_digit()));
        
        // Verify Discord username format (should contain #)
        assert!(discord_player.username.contains('#'));
        
        // Verify Discord avatar URL format
        if let Some(ref avatar_url) = discord_player.avatar_url {
            assert!(avatar_url.starts_with("https://cdn.discordapp.com/avatars/"));
            assert!(avatar_url.contains(&discord_player.discord_id));
        }
    }

    #[test]
    fn test_player_serialization_deserialization() {
        let current_time = Timestamp::from(1000);
        
        // Test Player serialization with all fields populated
        let player = Player {
            discord_id: "123456789012345678".to_string(),
            username: "SerializeTest#9999".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            total_points: 1500,
            participation_streak: 7,
            best_streak: 10,
            current_rank: Some(3),
            status: PlayerStatus::Active,
            created_at: current_time,
            last_active: current_time,
        };

        let serialized = serde_json::to_string(&player).expect("Should serialize Player");
        let deserialized: Player = serde_json::from_str(&serialized).expect("Should deserialize Player");
        
        assert_eq!(player.discord_id, deserialized.discord_id);
        assert_eq!(player.username, deserialized.username);
        assert_eq!(player.avatar_url, deserialized.avatar_url);
        assert_eq!(player.total_points, deserialized.total_points);
        assert_eq!(player.participation_streak, deserialized.participation_streak);
        assert_eq!(player.current_rank, deserialized.current_rank);
        assert_eq!(player.status, deserialized.status);
        assert_eq!(player.created_at, deserialized.created_at);
        assert_eq!(player.last_active, deserialized.last_active);
    }

    #[test]  
    fn test_player_status_serialization() {
        // Test all PlayerStatus variants serialization
        let statuses = vec![
            PlayerStatus::Active,
            PlayerStatus::Suspended { 
                reason: "Test suspension".to_string(),
                until: Some(Timestamp::from(2000)),
            },
            PlayerStatus::Suspended {
                reason: "Indefinite suspension".to_string(), 
                until: None,
            },
            PlayerStatus::Banned { 
                reason: "Test ban".to_string(),
            },
        ];

        for status in statuses {
            let serialized = serde_json::to_string(&status).expect("Should serialize PlayerStatus");
            let deserialized: PlayerStatus = serde_json::from_str(&serialized).expect("Should deserialize PlayerStatus");
            
            match (&status, &deserialized) {
                (PlayerStatus::Active, PlayerStatus::Active) => {},
                (
                    PlayerStatus::Suspended { reason: r1, until: u1 },
                    PlayerStatus::Suspended { reason: r2, until: u2 }
                ) => {
                    assert_eq!(r1, r2);
                    assert_eq!(u1, u2);
                },
                (
                    PlayerStatus::Banned { reason: r1 },
                    PlayerStatus::Banned { reason: r2 }
                ) => {
                    assert_eq!(r1, r2);
                },
                _ => panic!("PlayerStatus serialization/deserialization mismatch"),
            }
        }
    }

    #[test]
    fn test_player_cloning() {
        let current_time = Timestamp::from(1000);
        
        let original_player = Player {
            discord_id: "clone_test_123456".to_string(),
            username: "CloneTest#0000".to_string(),
            avatar_url: Some("https://example.com/clone_avatar.png".to_string()),
            total_points: 999,
            participation_streak: 2,
            best_streak: 4,
            current_rank: Some(99),
            status: PlayerStatus::Suspended {
                reason: "Testing clone".to_string(),
                until: Some(current_time),
            },
            created_at: current_time,
            last_active: current_time,
        };

        let cloned_player = original_player.clone();
        
        assert_eq!(original_player.discord_id, cloned_player.discord_id);
        assert_eq!(original_player.username, cloned_player.username);
        assert_eq!(original_player.avatar_url, cloned_player.avatar_url);
        assert_eq!(original_player.total_points, cloned_player.total_points);
        assert_eq!(original_player.participation_streak, cloned_player.participation_streak);
        assert_eq!(original_player.current_rank, cloned_player.current_rank);
        assert_eq!(original_player.status, cloned_player.status);
        assert_eq!(original_player.created_at, cloned_player.created_at);
        assert_eq!(original_player.last_active, cloned_player.last_active);
    }
}