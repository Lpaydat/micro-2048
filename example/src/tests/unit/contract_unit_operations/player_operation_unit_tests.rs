// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Player operation unit tests
//! 
//! Unit tests for player-related operations including registration, profile updates,
//! moderation actions like banning and suspension.

#![cfg(test)]

use crate::Operation;

/// Test RegisterPlayer operation structure
/// 
/// Tests that RegisterPlayer operations can be created with proper field validation.
#[test]
fn test_register_player_operation_structure() {
    let register_op = Operation::RegisterPlayer {
        discord_id: "test_player_123456".to_string(),
        username: "TestPlayer#0001".to_string(),
        avatar_url: None,
    };

    // Verify operation structure
    match register_op {
        Operation::RegisterPlayer { discord_id, username, avatar_url } => {
            assert_eq!(discord_id, "test_player_123456");
            assert_eq!(username, "TestPlayer#0001");
            assert_eq!(avatar_url, None);
        }
        _ => panic!("Expected RegisterPlayer operation"),
    }
}

/// Test RegisterPlayer operation with avatar URL
/// 
/// Tests RegisterPlayer operation with optional avatar URL field.
#[test]
fn test_register_player_with_avatar_structure() {
    let register_op = Operation::RegisterPlayer {
        discord_id: "test_player_with_avatar".to_string(),
        username: "TestPlayerAvatar#0001".to_string(),
        avatar_url: Some("https://cdn.discordapp.com/avatars/123/avatar.png".to_string()),
    };

    match register_op {
        Operation::RegisterPlayer { discord_id, username, avatar_url } => {
            assert_eq!(discord_id, "test_player_with_avatar");
            assert_eq!(username, "TestPlayerAvatar#0001");
            assert_eq!(avatar_url, Some("https://cdn.discordapp.com/avatars/123/avatar.png".to_string()));
        }
        _ => panic!("Expected RegisterPlayer operation"),
    }
}

/// Test UpdatePlayerProfile operation structure
/// 
/// Tests that UpdatePlayerProfile operations support partial updates.
#[test]
fn test_update_player_profile_operation_structure() {
    let update_op = Operation::UpdatePlayerProfile {
        discord_id: "test_player_update".to_string(),
        username: Some("UpdatedUsername#0002".to_string()),
        avatar_url: Some("https://example.com/new-avatar.png".to_string()),
    };

    match update_op {
        Operation::UpdatePlayerProfile { discord_id, username, avatar_url } => {
            assert_eq!(discord_id, "test_player_update");
            assert_eq!(username, Some("UpdatedUsername#0002".to_string()));
            assert_eq!(avatar_url, Some("https://example.com/new-avatar.png".to_string()));
        }
        _ => panic!("Expected UpdatePlayerProfile operation"),
    }
}

/// Test UpdatePlayerProfile with partial update
/// 
/// Tests that UpdatePlayerProfile can handle None values for partial updates.
#[test]
fn test_update_player_profile_partial_structure() {
    // Test username-only update
    let username_only_op = Operation::UpdatePlayerProfile {
        discord_id: "test_player_partial".to_string(),
        username: Some("OnlyUsername#0003".to_string()),
        avatar_url: None,
    };

    match username_only_op {
        Operation::UpdatePlayerProfile { discord_id, username, avatar_url } => {
            assert_eq!(discord_id, "test_player_partial");
            assert_eq!(username, Some("OnlyUsername#0003".to_string()));
            assert_eq!(avatar_url, None);
        }
        _ => panic!("Expected UpdatePlayerProfile operation"),
    }

    // Test avatar-only update
    let avatar_only_op = Operation::UpdatePlayerProfile {
        discord_id: "test_player_avatar_only".to_string(),
        username: None,
        avatar_url: Some("https://example.com/avatar-only.png".to_string()),
    };

    match avatar_only_op {
        Operation::UpdatePlayerProfile { discord_id, username, avatar_url } => {
            assert_eq!(discord_id, "test_player_avatar_only");
            assert_eq!(username, None);
            assert_eq!(avatar_url, Some("https://example.com/avatar-only.png".to_string()));
        }
        _ => panic!("Expected UpdatePlayerProfile operation"),
    }
}

/// Test BanPlayer operation structure
/// 
/// Tests that BanPlayer operations include caller, target, and reason.
#[test]
fn test_ban_player_operation_structure() {
    let ban_op = Operation::BanPlayer {
        caller_discord_id: "admin_123456".to_string(),
        player_discord_id: "violator_789012".to_string(),
        reason: "Terms of service violation".to_string(),
    };

    match ban_op {
        Operation::BanPlayer { caller_discord_id, player_discord_id, reason } => {
            assert_eq!(caller_discord_id, "admin_123456");
            assert_eq!(player_discord_id, "violator_789012");
            assert_eq!(reason, "Terms of service violation");
        }
        _ => panic!("Expected BanPlayer operation"),
    }
}

/// Test SuspendPlayer operation structure
/// 
/// Tests that SuspendPlayer operations support duration specification.
#[test]
fn test_suspend_player_operation_structure() {
    let suspend_op = Operation::SuspendPlayer {
        caller_discord_id: "moderator_123456".to_string(),
        player_discord_id: "suspended_789012".to_string(),
        reason: "Inappropriate behavior".to_string(),
        duration_hours: Some(24),
    };

    match suspend_op {
        Operation::SuspendPlayer { caller_discord_id, player_discord_id, reason, duration_hours } => {
            assert_eq!(caller_discord_id, "moderator_123456");
            assert_eq!(player_discord_id, "suspended_789012");
            assert_eq!(reason, "Inappropriate behavior");
            assert_eq!(duration_hours, Some(24));
        }
        _ => panic!("Expected SuspendPlayer operation"),
    }
}

/// Test SuspendPlayer operation with indefinite suspension
/// 
/// Tests that SuspendPlayer supports indefinite suspensions (None duration).
#[test]
fn test_suspend_player_indefinite_structure() {
    let suspend_indefinite_op = Operation::SuspendPlayer {
        caller_discord_id: "admin_indefinite".to_string(),
        player_discord_id: "player_indefinite".to_string(),
        reason: "Serious violation".to_string(),
        duration_hours: None, // Indefinite suspension
    };

    match suspend_indefinite_op {
        Operation::SuspendPlayer { caller_discord_id, player_discord_id, reason, duration_hours } => {
            assert_eq!(caller_discord_id, "admin_indefinite");
            assert_eq!(player_discord_id, "player_indefinite");
            assert_eq!(reason, "Serious violation");
            assert_eq!(duration_hours, None);
        }
        _ => panic!("Expected SuspendPlayer operation"),
    }
}

/// Test UnbanPlayer operation structure
/// 
/// Tests that UnbanPlayer operations include caller and target player.
#[test]
fn test_unban_player_operation_structure() {
    let unban_op = Operation::UnbanPlayer {
        caller_discord_id: "admin_unban".to_string(),
        player_discord_id: "player_to_unban".to_string(),
    };

    match unban_op {
        Operation::UnbanPlayer { caller_discord_id, player_discord_id } => {
            assert_eq!(caller_discord_id, "admin_unban");
            assert_eq!(player_discord_id, "player_to_unban");
        }
        _ => panic!("Expected UnbanPlayer operation"),
    }
}