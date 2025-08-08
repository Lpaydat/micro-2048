// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::{
        core::types::{Game, GameStatus, PendingGame, DeveloperInfo},
    };
    use linera_sdk::linera_base_types::Timestamp;

    #[test]
    fn test_game_struct_creation() {
        let current_time = Timestamp::from(1000);
        
        let game = Game {
            id: "chess_master".to_string(),
            name: "Chess Master".to_string(),
            description: "A strategic chess game".to_string(),
            contract_address: "0x1234567890abcdef".to_string(),
            developer_info: DeveloperInfo {
                name: "Chess Studios".to_string(),
                contact: "contact@chessstudios.com".to_string(),
            },
            status: GameStatus::Active,
            approved_by: Some("admin123".to_string()),
            created_at: current_time,
            approved_at: Some(current_time),
        };

        assert_eq!(game.id, "chess_master");
        assert_eq!(game.name, "Chess Master");
        assert_eq!(game.description, "A strategic chess game");
        assert_eq!(game.contract_address, "0x1234567890abcdef");
        assert_eq!(game.developer_info.name, "Chess Studios");
        assert_eq!(game.developer_info.contact, "contact@chessstudios.com");
        assert!(matches!(game.status, GameStatus::Active));
        assert_eq!(game.approved_by, Some("admin123".to_string()));
        assert_eq!(game.created_at, current_time);
        assert_eq!(game.approved_at, Some(current_time));
    }

    #[test]
    fn test_pending_game_struct() {
        let current_time = Timestamp::from(1000);
        
        let pending_game = PendingGame {
            id: "new_game".to_string(),
            name: "New Game".to_string(),
            description: "An exciting new game".to_string(),
            contract_address: "0xabcdef1234567890".to_string(),
            developer_info: DeveloperInfo {
                name: "Indie Dev".to_string(),
                contact: "dev@indiegames.com".to_string(),
            },
            created_at: current_time,
        };

        assert_eq!(pending_game.id, "new_game");
        assert_eq!(pending_game.name, "New Game");
        assert_eq!(pending_game.description, "An exciting new game");
        assert_eq!(pending_game.contract_address, "0xabcdef1234567890");
        assert_eq!(pending_game.developer_info.name, "Indie Dev");
        assert_eq!(pending_game.developer_info.contact, "dev@indiegames.com");
        assert_eq!(pending_game.created_at, current_time);
    }

    #[test]
    fn test_game_status_variants() {
        // Test Pending status
        let pending = GameStatus::Pending;
        assert!(matches!(pending, GameStatus::Pending));

        // Test Active status
        let active = GameStatus::Active;
        assert!(matches!(active, GameStatus::Active));

        // Test Suspended status
        let suspended = GameStatus::Suspended {
            reason: "Terms violation".to_string(),
        };
        if let GameStatus::Suspended { reason } = suspended {
            assert_eq!(reason, "Terms violation");
        } else {
            panic!("Expected Suspended status");
        }

        // Test Deprecated status
        let deprecated = GameStatus::Deprecated;
        assert!(matches!(deprecated, GameStatus::Deprecated));
    }

    #[test]
    fn test_developer_info_structure() {
        let dev_info = DeveloperInfo {
            name: "Awesome Games Inc.".to_string(),
            contact: "support@awesomegames.com".to_string(),
        };

        assert_eq!(dev_info.name, "Awesome Games Inc.");
        assert_eq!(dev_info.contact, "support@awesomegames.com");
    }

    #[test]
    fn test_game_approval_workflow() {
        let current_time = Timestamp::from(1000);
        let approval_time = Timestamp::from(2000);
        
        // Start with pending game
        let pending_game = PendingGame {
            id: "test_game".to_string(),
            name: "Test Game".to_string(),
            description: "A game for testing".to_string(),
            contract_address: "0x1111222233334444".to_string(),
            developer_info: DeveloperInfo {
                name: "Test Developer".to_string(),
                contact: "test@example.com".to_string(),
            },
            created_at: current_time,
        };

        // Convert to approved game
        let approved_game = Game {
            id: pending_game.id.clone(),
            name: pending_game.name.clone(),
            description: pending_game.description.clone(),
            contract_address: pending_game.contract_address.clone(),
            developer_info: pending_game.developer_info.clone(),
            status: GameStatus::Active,
            approved_by: Some("admin456".to_string()),
            created_at: pending_game.created_at,
            approved_at: Some(approval_time),
        };

        // Verify the conversion preserved data correctly
        assert_eq!(approved_game.id, pending_game.id);
        assert_eq!(approved_game.name, pending_game.name);
        assert_eq!(approved_game.description, pending_game.description);
        assert_eq!(approved_game.contract_address, pending_game.contract_address);
        assert_eq!(approved_game.developer_info.name, pending_game.developer_info.name);
        assert_eq!(approved_game.developer_info.contact, pending_game.developer_info.contact);
        assert_eq!(approved_game.created_at, pending_game.created_at);
        
        // Verify approval-specific fields
        assert!(matches!(approved_game.status, GameStatus::Active));
        assert_eq!(approved_game.approved_by, Some("admin456".to_string()));
        assert_eq!(approved_game.approved_at, Some(approval_time));
    }

    #[test]
    fn test_game_status_transitions() {
        let current_time = Timestamp::from(1000);
        
        // Create an active game
        let mut game = Game {
            id: "status_test".to_string(),
            name: "Status Test Game".to_string(),
            description: "Testing status transitions".to_string(),
            contract_address: "0x5555666677778888".to_string(),
            developer_info: DeveloperInfo {
                name: "Status Tester".to_string(),
                contact: "status@test.com".to_string(),
            },
            status: GameStatus::Active,
            approved_by: Some("admin789".to_string()),
            created_at: current_time,
            approved_at: Some(current_time),
        };

        // Test suspension
        game.status = GameStatus::Suspended {
            reason: "Policy violation".to_string(),
        };
        if let GameStatus::Suspended { reason } = &game.status {
            assert_eq!(reason, "Policy violation");
        } else {
            panic!("Expected Suspended status");
        }

        // Test reactivation
        game.status = GameStatus::Active;
        assert!(matches!(game.status, GameStatus::Active));

        // Test deprecation
        game.status = GameStatus::Deprecated;
        assert!(matches!(game.status, GameStatus::Deprecated));
    }

    #[test]
    fn test_serialization_deserialization() {
        let current_time = Timestamp::from(1000);
        
        // Test Game serialization
        let game = Game {
            id: "serialize_test".to_string(),
            name: "Serialization Test".to_string(),
            description: "Testing serialization".to_string(),
            contract_address: "0x9999aaaabbbbcccc".to_string(),
            developer_info: DeveloperInfo {
                name: "Serialize Dev".to_string(),
                contact: "serialize@test.com".to_string(),
            },
            status: GameStatus::Active,
            approved_by: Some("admin_serialize".to_string()),
            created_at: current_time,
            approved_at: Some(current_time),
        };

        let serialized = serde_json::to_string(&game).expect("Should serialize");
        let deserialized: Game = serde_json::from_str(&serialized).expect("Should deserialize");
        
        assert_eq!(game.id, deserialized.id);
        assert_eq!(game.name, deserialized.name);
        assert_eq!(game.description, deserialized.description);
        assert_eq!(game.contract_address, deserialized.contract_address);
        assert_eq!(game.developer_info.name, deserialized.developer_info.name);
        assert_eq!(game.developer_info.contact, deserialized.developer_info.contact);
        assert_eq!(game.approved_by, deserialized.approved_by);

        // Test PendingGame serialization
        let pending_game = PendingGame {
            id: "pending_serialize".to_string(),
            name: "Pending Serialization Test".to_string(),
            description: "Testing pending game serialization".to_string(),
            contract_address: "0xddddeeeeffffaaaa".to_string(),
            developer_info: DeveloperInfo {
                name: "Pending Dev".to_string(),
                contact: "pending@test.com".to_string(),
            },
            created_at: current_time,
        };

        let serialized_pending = serde_json::to_string(&pending_game).expect("Should serialize");
        let deserialized_pending: PendingGame = serde_json::from_str(&serialized_pending).expect("Should deserialize");
        
        assert_eq!(pending_game.id, deserialized_pending.id);
        assert_eq!(pending_game.name, deserialized_pending.name);
        assert_eq!(pending_game.description, deserialized_pending.description);
        assert_eq!(pending_game.contract_address, deserialized_pending.contract_address);
        assert_eq!(pending_game.developer_info.name, deserialized_pending.developer_info.name);
        assert_eq!(pending_game.developer_info.contact, deserialized_pending.developer_info.contact);
        assert_eq!(pending_game.created_at, deserialized_pending.created_at);
    }

    #[test]
    fn test_game_status_serialization() {
        // Test all GameStatus variants
        let statuses = vec![
            GameStatus::Pending,
            GameStatus::Active,
            GameStatus::Suspended { reason: "Test suspension".to_string() },
            GameStatus::Deprecated,
        ];

        for status in statuses {
            let serialized = serde_json::to_string(&status).expect("Should serialize");
            let deserialized: GameStatus = serde_json::from_str(&serialized).expect("Should deserialize");
            
            match (&status, &deserialized) {
                (GameStatus::Pending, GameStatus::Pending) => (),
                (GameStatus::Active, GameStatus::Active) => (),
                (GameStatus::Deprecated, GameStatus::Deprecated) => (),
                (GameStatus::Suspended { reason: r1 }, GameStatus::Suspended { reason: r2 }) => {
                    assert_eq!(r1, r2);
                },
                _ => panic!("Status serialization/deserialization mismatch"),
            }
        }
    }
}