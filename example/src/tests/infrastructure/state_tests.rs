// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for all data models

#[cfg(test)]
mod tests {
    use linera_sdk::linera_base_types::Timestamp;
    use crate::core::types::{
        Player, PlayerStatus, Game, GameStatus, PendingGame, DeveloperInfo, GameHubEvent, EventType
    };

    #[test]
    fn test_player_struct_with_discord_integration() {
        // Test Player struct with Discord integration fields and participation streak tracking
        let timestamp = Timestamp::from(1000000);
        
        let player = Player {
            discord_id: "123456789012345678".to_string(),
            username: "TestPlayer#1234".to_string(),
            avatar_url: Some("https://cdn.discordapp.com/avatars/123456789012345678/abc123.png".to_string()),
            total_points: 1500,
            participation_streak: 5,
            best_streak: 8,
            current_rank: Some(42),
            status: PlayerStatus::Active,
            created_at: timestamp,
            last_active: timestamp,
        };

        // Verify Discord integration fields
        assert_eq!(player.discord_id, "123456789012345678");
        assert_eq!(player.username, "TestPlayer#1234");
        assert!(player.avatar_url.is_some());
        assert_eq!(player.avatar_url.unwrap(), "https://cdn.discordapp.com/avatars/123456789012345678/abc123.png");

        // Verify participation streak tracking
        assert_eq!(player.participation_streak, 5);
        assert_eq!(player.total_points, 1500);
        assert_eq!(player.current_rank, Some(42));
        assert_eq!(player.status, PlayerStatus::Active);
        assert_eq!(player.created_at, timestamp);
        assert_eq!(player.last_active, timestamp);
    }

    #[test]
    fn test_player_status_enum() {
        // Test all PlayerStatus variants
        let active_status = PlayerStatus::Active;
        let suspended_status = PlayerStatus::Suspended {
            reason: "Inappropriate behavior".to_string(),
            until: Some(Timestamp::from(2000000)),
        };
        let banned_status = PlayerStatus::Banned {
            reason: "Terms of service violation".to_string(),
        };

        // Verify status types
        assert_eq!(active_status, PlayerStatus::Active);
        
        match suspended_status {
            PlayerStatus::Suspended { reason, until } => {
                assert_eq!(reason, "Inappropriate behavior");
                assert_eq!(until, Some(Timestamp::from(2000000)));
            }
            _ => panic!("Expected Suspended status"),
        }

        match banned_status {
            PlayerStatus::Banned { reason } => {
                assert_eq!(reason, "Terms of service violation");
            }
            _ => panic!("Expected Banned status"),
        }
    }

    #[test]
    fn test_game_struct() {
        // Test Game struct with enhanced fields
        let timestamp = Timestamp::from(1500000);
        
        let game = Game {
            id: "awesome-puzzle-game".to_string(),
            name: "Awesome Puzzle Game".to_string(),
            description: "A challenging puzzle game that tests your logic skills".to_string(),
            contract_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            developer_info: DeveloperInfo {
                name: "Puzzle Masters Studio".to_string(),
                contact: "contact@puzzlemasters.com".to_string(),
            },
            status: GameStatus::Active,
            approved_by: Some("admin123".to_string()),
            created_at: timestamp,
            approved_at: Some(timestamp),
        };

        // Verify game fields
        assert_eq!(game.id, "awesome-puzzle-game");
        assert_eq!(game.name, "Awesome Puzzle Game");
        assert_eq!(game.description, "A challenging puzzle game that tests your logic skills");
        assert_eq!(game.contract_address, "0x1234567890abcdef1234567890abcdef12345678");
        assert_eq!(game.developer_info.name, "Puzzle Masters Studio");
        assert_eq!(game.developer_info.contact, "contact@puzzlemasters.com");
        assert_eq!(game.status, GameStatus::Active);
        assert_eq!(game.approved_by, Some("admin123".to_string()));
        assert_eq!(game.created_at, timestamp);
        assert_eq!(game.approved_at, Some(timestamp));
    }

    #[test]
    fn test_game_status_enum() {
        // Test all GameStatus variants
        let active = GameStatus::Active;
        let pending = GameStatus::Pending;
        let suspended = GameStatus::Suspended { reason: "Test suspension".to_string() };
        let deprecated = GameStatus::Deprecated;

        // Verify all statuses
        assert_eq!(active, GameStatus::Active);
        assert_eq!(pending, GameStatus::Pending);
        assert_eq!(suspended, GameStatus::Suspended { reason: "Test suspension".to_string() });
        assert_eq!(deprecated, GameStatus::Deprecated);
    }

    #[test]
    fn test_pending_game_struct() {
        // Test PendingGame struct
        let timestamp = Timestamp::from(1750000);
        
        let pending_game = PendingGame {
            id: "new-strategy-game".to_string(),
            name: "Epic Strategy Game".to_string(),
            description: "A complex strategy game with multiple victory conditions".to_string(),
            contract_address: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string(),
            developer_info: DeveloperInfo {
                name: "Strategy Game Studios".to_string(),
                contact: "devteam@strategygames.com".to_string(),
            },
            created_at: timestamp,
        };

        // Verify pending game fields
        assert_eq!(pending_game.id, "new-strategy-game");
        assert_eq!(pending_game.name, "Epic Strategy Game");
        assert_eq!(pending_game.description, "A complex strategy game with multiple victory conditions");
        assert_eq!(pending_game.contract_address, "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");
        assert_eq!(pending_game.developer_info.name, "Strategy Game Studios");
        assert_eq!(pending_game.developer_info.contact, "devteam@strategygames.com");
        assert_eq!(pending_game.created_at, timestamp);
    }

    #[test]
    fn test_developer_info_struct() {
        // Test DeveloperInfo struct
        let dev_info = DeveloperInfo {
            name: "Indie Game Developer".to_string(),
            contact: "indie@gamedev.com".to_string(),
        };

        assert_eq!(dev_info.name, "Indie Game Developer");
        assert_eq!(dev_info.contact, "indie@gamedev.com");
    }

    #[test]
    fn test_gamehub_event_struct() {
        // Test GameHubEvent struct
        let timestamp = Timestamp::from(2000000);
        
        let event = GameHubEvent {
            id: "event_123456_789".to_string(),
            event_type: EventType::PlayerRegistered,
            description: "New player registered in the system".to_string(),
            actor_id: Some("123456789012345678".to_string()),
            target_id: Some("new-player-id".to_string()),
            timestamp,
            metadata: Some("Additional event metadata".to_string()),
        };

        // Verify event fields
        assert_eq!(event.id, "event_123456_789");
        assert_eq!(event.event_type, EventType::PlayerRegistered);
        assert_eq!(event.description, "New player registered in the system");
        assert_eq!(event.actor_id, Some("123456789012345678".to_string()));
        assert_eq!(event.target_id, Some("new-player-id".to_string()));
        assert_eq!(event.timestamp, timestamp);
        assert_eq!(event.metadata, Some("Additional event metadata".to_string()));
    }

    #[test]
    fn test_event_type_enum() {
        // Test all EventType variants
        let player_registered = EventType::PlayerRegistered;
        let player_banned = EventType::PlayerBanned;
        let player_suspended = EventType::PlayerSuspended;
        let player_unbanned = EventType::PlayerUnbanned;
        let game_approved = EventType::GameApproved;
        let game_rejected = EventType::GameRejected;
        let game_submitted = EventType::GameSubmitted;
        let admin_action = EventType::AdminAction;

        // Verify all event types
        assert_eq!(player_registered, EventType::PlayerRegistered);
        assert_eq!(player_banned, EventType::PlayerBanned);
        assert_eq!(player_suspended, EventType::PlayerSuspended);
        assert_eq!(player_unbanned, EventType::PlayerUnbanned);
        assert_eq!(game_approved, EventType::GameApproved);
        assert_eq!(game_rejected, EventType::GameRejected);
        assert_eq!(game_submitted, EventType::GameSubmitted);
        assert_eq!(admin_action, EventType::AdminAction);
    }

    #[test]
    fn test_timestamp_handling() {
        // Test timestamp operations in data models
        let ts1 = Timestamp::from(1000000);
        let ts2 = Timestamp::from(2000000);
        let ts3 = Timestamp::from(3000000);

        // Create player with different timestamps
        let player = Player {
            discord_id: "timestamp-test-player".to_string(),
            username: "TimestampTestUser".to_string(),
            avatar_url: None,
            total_points: 0,
            participation_streak: 0,
            best_streak: 0,
            current_rank: None,
            status: PlayerStatus::Active,
            created_at: ts1,
            last_active: ts2,
        };

        // Verify timestamps
        assert_eq!(player.created_at.micros(), 1000000);
        assert_eq!(player.last_active.micros(), 2000000);
        assert!(player.last_active.micros() > player.created_at.micros());

        // Test timestamp in events
        let event = GameHubEvent {
            id: "timestamp-event".to_string(),
            event_type: EventType::PlayerRegistered,
            description: "Test event".to_string(),
            actor_id: None,
            target_id: None,
            timestamp: ts3,
            metadata: None,
        };

        assert_eq!(event.timestamp.micros(), 3000000);
    }

    #[test]
    fn test_optional_fields() {
        // Test optional fields in data models
        let player_no_avatar = Player {
            discord_id: "no-avatar-player".to_string(),
            username: "NoAvatarUser".to_string(),
            avatar_url: None,
            total_points: 100,
            participation_streak: 2,
            best_streak: 3,
            current_rank: None,
            status: PlayerStatus::Active,
            created_at: Timestamp::from(1000000),
            last_active: Timestamp::from(1000000),
        };

        assert!(player_no_avatar.avatar_url.is_none());
        assert!(player_no_avatar.current_rank.is_none());

        // Test game without approval info
        let unapproved_game = Game {
            id: "unapproved-game".to_string(),
            name: "Unapproved Game".to_string(),
            description: "A game pending approval".to_string(),
            contract_address: "0x1111111111111111111111111111111111111111".to_string(),
            developer_info: DeveloperInfo {
                name: "Test Dev".to_string(),
                contact: "test@dev.com".to_string(),
            },
            status: GameStatus::Pending,
            approved_by: None,
            created_at: Timestamp::from(1000000),
            approved_at: None,
        };

        assert!(unapproved_game.approved_by.is_none());
        assert!(unapproved_game.approved_at.is_none());
    }

    #[test]
    fn test_data_model_cloning() {
        // Test that data models can be cloned
        let original_player = Player {
            discord_id: "clone-test-player".to_string(),
            username: "CloneTestUser".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            total_points: 500,
            participation_streak: 3,
            best_streak: 5,
            current_rank: Some(10),
            status: PlayerStatus::Active,
            created_at: Timestamp::from(1000000),
            last_active: Timestamp::from(1500000),
        };

        let cloned_player = original_player.clone();
        
        assert_eq!(original_player.discord_id, cloned_player.discord_id);
        assert_eq!(original_player.username, cloned_player.username);
        assert_eq!(original_player.total_points, cloned_player.total_points);
        assert_eq!(original_player.status, cloned_player.status);
    }
}