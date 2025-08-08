// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Admin permission unit tests
//! 
//! Unit tests for administrative operations including admin management
//! and moderator assignment operations.

#![cfg(test)]

use crate::Operation;

/// Test AddAdmin operation structure
/// 
/// Tests that AddAdmin operations include caller and target admin identifiers.
#[test]
fn test_add_admin_operation_structure() {
    let add_admin_op = Operation::AddAdmin {
        caller_discord_id: "super_admin_123".to_string(),
        discord_id: "new_admin_456".to_string(),
    };

    match add_admin_op {
        Operation::AddAdmin { caller_discord_id, discord_id } => {
            assert_eq!(caller_discord_id, "super_admin_123");
            assert_eq!(discord_id, "new_admin_456");
        }
        _ => panic!("Expected AddAdmin operation"),
    }
}

/// Test RemoveAdmin operation structure
/// 
/// Tests that RemoveAdmin operations include caller and target admin identifiers.
#[test]
fn test_remove_admin_operation_structure() {
    let remove_admin_op = Operation::RemoveAdmin {
        caller_discord_id: "super_admin_remove".to_string(),
        discord_id: "admin_to_remove".to_string(),
    };

    match remove_admin_op {
        Operation::RemoveAdmin { caller_discord_id, discord_id } => {
            assert_eq!(caller_discord_id, "super_admin_remove");
            assert_eq!(discord_id, "admin_to_remove");
        }
        _ => panic!("Expected RemoveAdmin operation"),
    }
}

/// Test AssignModerator operation structure
/// 
/// Tests that AssignModerator operations include caller and target moderator identifiers.
#[test]
fn test_assign_moderator_operation_structure() {
    let assign_mod_op = Operation::AssignModerator {
        caller_discord_id: "admin_assigner".to_string(),
        discord_id: "new_moderator_789".to_string(),
    };

    match assign_mod_op {
        Operation::AssignModerator { caller_discord_id, discord_id } => {
            assert_eq!(caller_discord_id, "admin_assigner");
            assert_eq!(discord_id, "new_moderator_789");
        }
        _ => panic!("Expected AssignModerator operation"),
    }
}

/// Test RemoveModerator operation structure
/// 
/// Tests that RemoveModerator operations include caller and target moderator identifiers.
#[test]
fn test_remove_moderator_operation_structure() {
    let remove_mod_op = Operation::RemoveModerator {
        caller_discord_id: "admin_remover".to_string(),
        discord_id: "moderator_to_remove".to_string(),
    };

    match remove_mod_op {
        Operation::RemoveModerator { caller_discord_id, discord_id } => {
            assert_eq!(caller_discord_id, "admin_remover");
            assert_eq!(discord_id, "moderator_to_remove");
        }
        _ => panic!("Expected RemoveModerator operation"),
    }
}