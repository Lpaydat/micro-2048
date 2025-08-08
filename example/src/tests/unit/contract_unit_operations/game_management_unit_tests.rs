// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Game management unit tests
//! 
//! Unit tests for game-related operations including approval, rejection,
//! suspension, reactivation, and deprecation.

#![cfg(test)]

use crate::Operation;

/// Test ApproveGame operation structure
/// 
/// Tests that ApproveGame operations include caller and game identifiers.
#[test]
fn test_approve_game_operation_structure() {
    let approve_op = Operation::ApproveGame {
        caller_discord_id: "admin_approver".to_string(),
        game_id: "awesome_game_123".to_string(),
    };

    match approve_op {
        Operation::ApproveGame { caller_discord_id, game_id } => {
            assert_eq!(caller_discord_id, "admin_approver");
            assert_eq!(game_id, "awesome_game_123");
        }
        _ => panic!("Expected ApproveGame operation"),
    }
}

/// Test RejectGame operation structure
/// 
/// Tests that RejectGame operations include caller, game, and rejection reason.
#[test]
fn test_reject_game_operation_structure() {
    let reject_op = Operation::RejectGame {
        caller_discord_id: "admin_rejector".to_string(),
        game_id: "rejected_game_456".to_string(),
        reason: "Does not meet quality standards".to_string(),
    };

    match reject_op {
        Operation::RejectGame { caller_discord_id, game_id, reason } => {
            assert_eq!(caller_discord_id, "admin_rejector");
            assert_eq!(game_id, "rejected_game_456");
            assert_eq!(reason, "Does not meet quality standards");
        }
        _ => panic!("Expected RejectGame operation"),
    }
}

/// Test SuspendGame operation structure
/// 
/// Tests that SuspendGame operations include caller, game, and suspension reason.
#[test]
fn test_suspend_game_operation_structure() {
    let suspend_op = Operation::SuspendGame {
        caller_discord_id: "admin_suspender".to_string(),
        game_id: "suspended_game_789".to_string(),
        reason: "Quality issues reported".to_string(),
    };

    match suspend_op {
        Operation::SuspendGame { caller_discord_id, game_id, reason } => {
            assert_eq!(caller_discord_id, "admin_suspender");
            assert_eq!(game_id, "suspended_game_789");
            assert_eq!(reason, "Quality issues reported");
        }
        _ => panic!("Expected SuspendGame operation"),
    }
}

/// Test ReactivateGame operation structure
/// 
/// Tests that ReactivateGame operations include caller and game identifiers.
#[test]
fn test_reactivate_game_operation_structure() {
    let reactivate_op = Operation::ReactivateGame {
        caller_discord_id: "admin_reactivator".to_string(),
        game_id: "reactivated_game_321".to_string(),
    };

    match reactivate_op {
        Operation::ReactivateGame { caller_discord_id, game_id } => {
            assert_eq!(caller_discord_id, "admin_reactivator");
            assert_eq!(game_id, "reactivated_game_321");
        }
        _ => panic!("Expected ReactivateGame operation"),
    }
}

/// Test DeprecateGame operation structure
/// 
/// Tests that DeprecateGame operations include caller and game identifiers.
#[test]
fn test_deprecate_game_operation_structure() {
    let deprecate_op = Operation::DeprecateGame {
        caller_discord_id: "admin_deprecator".to_string(),
        game_id: "deprecated_game_654".to_string(),
    };

    match deprecate_op {
        Operation::DeprecateGame { caller_discord_id, game_id } => {
            assert_eq!(caller_discord_id, "admin_deprecator");
            assert_eq!(game_id, "deprecated_game_654");
        }
        _ => panic!("Expected DeprecateGame operation"),
    }
}