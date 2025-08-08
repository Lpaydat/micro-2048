// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Business logic tests
//! 
//! Unit tests for cross-operation consistency and business logic patterns.

#![cfg(test)]

use crate::Operation;

/// Test operation consistency across related operations
/// 
/// Tests that related operations use consistent field patterns.
#[test]
fn test_operation_consistency() {
    let player_id = "consistency_test_player";
    let game_id = "consistency_test_game";
    let admin_id = "consistency_test_admin";

    // Player-related operations should use consistent ID patterns
    let register_op = Operation::RegisterPlayer {
        discord_id: player_id.to_string(),
        username: "ConsistencyTest#0001".to_string(),
        avatar_url: None,
    };

    let update_op = Operation::UpdatePlayerProfile {
        discord_id: player_id.to_string(),
        username: Some("UpdatedConsistency#0001".to_string()),
        avatar_url: None,
    };

    let ban_op = Operation::BanPlayer {
        caller_discord_id: "123456789012345678".to_string(),
        player_discord_id: player_id.to_string(),
        reason: "Consistency test".to_string(),
    };

    // Verify ID consistency (note: different operations use different field names)
    match register_op {
        Operation::RegisterPlayer { discord_id, .. } => assert_eq!(discord_id, player_id),
        _ => panic!("Expected RegisterPlayer operation"),
    }

    match update_op {
        Operation::UpdatePlayerProfile { discord_id, .. } => assert_eq!(discord_id, player_id),
        _ => panic!("Expected UpdatePlayerProfile operation"),
    }

    match ban_op {
        Operation::BanPlayer { player_discord_id, .. } => assert_eq!(player_discord_id, player_id),
        _ => panic!("Expected BanPlayer operation"),
    }

    // Game-related operations should use consistent patterns
    let approve_op = Operation::ApproveGame {
            caller_discord_id: "123456789012345678".to_string(),
            game_id: game_id.to_string(),
        };

    let suspend_game_op = Operation::SuspendGame {
        caller_discord_id: "123456789012345678".to_string(),
        game_id: game_id.to_string(),
        reason: "Consistency test".to_string(),
    };

    match approve_op {
        Operation::ApproveGame { caller_discord_id, game_id: id } => {
            assert_eq!(caller_discord_id, "123456789012345678");
            assert_eq!(id, game_id);
        }
        _ => panic!("Expected ApproveGame operation"),
    }

    match suspend_game_op {
        Operation::SuspendGame { caller_discord_id, game_id: id, .. } => {
            assert_eq!(caller_discord_id, "123456789012345678");
            assert_eq!(id, game_id);
        }
        _ => panic!("Expected SuspendGame operation"),
    }

    // Admin operations should use consistent patterns
    let add_admin_op = Operation::AddAdmin {
        caller_discord_id: "123456789012345678".to_string(),
        discord_id: admin_id.to_string(),
    };

    let assign_mod_op = Operation::AssignModerator {
        caller_discord_id: "123456789012345678".to_string(),
        discord_id: admin_id.to_string(),
    };

    match add_admin_op {
        Operation::AddAdmin { caller_discord_id, discord_id } => {
            assert_eq!(caller_discord_id, "123456789012345678");
            assert_eq!(discord_id, admin_id);
        }
        _ => panic!("Expected AddAdmin operation"),
    }

    match assign_mod_op {
        Operation::AssignModerator { caller_discord_id, discord_id } => {
            assert_eq!(caller_discord_id, "123456789012345678");
            assert_eq!(discord_id, admin_id);
        }
        _ => panic!("Expected AssignModerator operation"),
    }
}