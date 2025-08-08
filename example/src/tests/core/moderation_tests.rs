// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod permission_tests {
    use crate::{
        core::types::AdminAction,
        infrastructure::errors::GameHubError,
    };

    #[test]
    fn test_permission_error_types() {
        // Test that permission error types are properly defined
        let not_admin_error = GameHubError::NotAdmin;
        let not_moderator_error = GameHubError::NotModerator;
        let insufficient_permissions_error = GameHubError::InsufficientPermissions;

        // Verify error messages
        assert_eq!(format!("{}", not_admin_error), "Not an admin");
        assert_eq!(format!("{}", not_moderator_error), "Not a moderator");
        assert_eq!(format!("{}", insufficient_permissions_error), "Insufficient permissions");
    }

    #[test]
    fn test_admin_action_types() {
        // Test that admin action types are properly defined with struct data
        let game_approved = AdminAction::GameApproved {
            game_id: "test-game".to_string(),
            game_name: "Test Game".to_string(),
        };
        let player_banned = AdminAction::PlayerBanned {
            player_id: "test-player".to_string(),
            reason: "violation".to_string(),
        };
        let moderator_assigned = AdminAction::ModeratorAssigned {
            moderator_id: "test-mod".to_string(),
        };
        let moderator_removed = AdminAction::ModeratorRemoved {
            moderator_id: "test-mod".to_string(),
        };
        let player_suspended = AdminAction::PlayerSuspended {
            player_id: "test-player".to_string(),
            reason: "warning".to_string(),
            duration_hours: Some(24),
        };
        let player_unbanned = AdminAction::PlayerUnbanned {
            player_id: "test-player".to_string(),
        };
        let game_rejected = AdminAction::GameRejected {
            game_id: "test-game".to_string(),
            reason: "quality".to_string(),
        };

        // Verify that the actions can be created and cloned
        let _cloned_game_approved = game_approved.clone();
        let _cloned_player_banned = player_banned.clone();
        let _cloned_moderator_assigned = moderator_assigned.clone();
        let _cloned_moderator_removed = moderator_removed.clone();
        let _cloned_player_suspended = player_suspended.clone();
        let _cloned_player_unbanned = player_unbanned.clone();
        let _cloned_game_rejected = game_rejected.clone();

        // Test matching admin actions
        match game_approved {
            AdminAction::GameApproved { game_id, game_name } => {
                assert_eq!(game_id, "test-game");
                assert_eq!(game_name, "Test Game");
            },
            _ => panic!("Expected GameApproved"),
        }

        match player_banned {
            AdminAction::PlayerBanned { player_id, reason } => {
                assert_eq!(player_id, "test-player");
                assert_eq!(reason, "violation");
            },
            _ => panic!("Expected PlayerBanned"),
        }

        match moderator_assigned {
            AdminAction::ModeratorAssigned { moderator_id } => {
                assert_eq!(moderator_id, "test-mod");
            },
            _ => panic!("Expected ModeratorAssigned"),
        }
    }

    #[test]
    fn test_player_error_types() {
        // Test player-specific error types
        let player_not_found = GameHubError::PlayerNotFound;
        let player_already_exists = GameHubError::PlayerAlreadyExists;

        assert_eq!(format!("{}", player_not_found), "Player not found");
        assert_eq!(format!("{}", player_already_exists), "Player already exists");

        // Test error with structured data  
        let player_banned = GameHubError::PlayerBanned {
            reason: "Cheating detected".to_string(),
        };
        assert!(format!("{}", player_banned).contains("Player is banned: Cheating detected"));
    }

    #[test]
    fn test_game_error_types() {
        // Test game-specific error types
        let game_not_found = GameHubError::GameNotFound { game_id: "test_game".to_string() };
        let game_not_approved = GameHubError::GameNotApproved;
        let game_already_exists = GameHubError::GameAlreadyExists;
        let invalid_contract_address = GameHubError::InvalidContractAddress;

        assert_eq!(format!("{}", game_not_found), "Game not found: test_game");
        assert_eq!(format!("{}", game_not_approved), "Game not approved");
        assert_eq!(format!("{}", game_already_exists), "Game already exists");
        assert_eq!(format!("{}", invalid_contract_address), "Invalid contract address");
    }

    #[test]
    fn test_validation_error_types() {
        // Test validation error types
        let invalid_input = GameHubError::InvalidInput {
            field: "username".to_string(),
            reason: "contains invalid characters".to_string(),
        };

        let input_too_long = GameHubError::InputTooLong {
            field: "description".to_string(),
            max_length: 500,
        };

        let input_too_short = GameHubError::InputTooShort {
            field: "game_name".to_string(),
            min_length: 3,
        };

        assert!(format!("{}", invalid_input).contains("Invalid input for field 'username'"));
        assert!(format!("{}", input_too_long).contains("Input too long: field 'description'"));
        assert!(format!("{}", input_too_short).contains("Input too short: field 'game_name'"));
    }

    #[test]
    fn test_system_error_types() {
        // Test system error types
        let database_error = GameHubError::DatabaseError;
        let message_processing_error = GameHubError::MessageProcessingError;
        let configuration_error = GameHubError::ConfigurationError;

        assert_eq!(format!("{}", database_error), "Database error");
        assert_eq!(format!("{}", message_processing_error), "Message processing error");
        assert_eq!(format!("{}", configuration_error), "Configuration error");
    }

    #[test]
    fn test_error_cloning() {
        // Test that errors can be cloned
        let original_error = GameHubError::PlayerBanned {
            reason: "Test ban".to_string(),
        };

        let cloned_error = original_error.clone();

        match (&original_error, &cloned_error) {
            (GameHubError::PlayerBanned { reason: r1 }, GameHubError::PlayerBanned { reason: r2 }) => {
                assert_eq!(r1, r2);
            },
            _ => panic!("Error cloning failed"),
        }
    }
}