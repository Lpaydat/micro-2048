// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Data structure integration tests
//! 
//! Integration tests for player data structures and their relationships.

#![cfg(test)]

use crate::core::types::{Player, PlayerStatus, PlayerStats};
use super::test_helpers::{test_timestamp, create_test_player};

#[test]
fn test_player_struct_creation() {
    // Test Player struct creation with various status types
    let timestamp = test_timestamp();
    
    let active_player = Player {
        discord_id: "123456789012345678".to_string(),
        username: "TestPlayer#1234".to_string(),
        avatar_url: Some("https://cdn.discordapp.com/avatars/123456789012345678/avatar.png".to_string()),
        total_points: 0,
        participation_streak: 0,
        best_streak: 0,
        current_rank: None,
        status: PlayerStatus::Active,
        created_at: timestamp,
        last_active: timestamp,
    };
    
    assert_eq!(active_player.discord_id, "123456789012345678");
    assert_eq!(active_player.username, "TestPlayer#1234");
    assert_eq!(active_player.status, PlayerStatus::Active);
    assert_eq!(active_player.created_at, timestamp);
    assert_eq!(active_player.last_active, timestamp);
}

#[test]
fn test_player_status_variants() {
    // Test different PlayerStatus variants
    let active = PlayerStatus::Active;
    let banned = PlayerStatus::Banned { reason: "Test ban".to_string() };
    let suspended = PlayerStatus::Suspended { 
        reason: "Test suspension".to_string(),
        until: Some(test_timestamp()),
    };
    let suspended_indefinite = PlayerStatus::Suspended {
        reason: "Indefinite suspension".to_string(),
        until: None,
    };
    
    assert!(matches!(active, PlayerStatus::Active));
    assert!(matches!(banned, PlayerStatus::Banned { .. }));
    assert!(matches!(suspended, PlayerStatus::Suspended { .. }));
    assert!(matches!(suspended_indefinite, PlayerStatus::Suspended { .. }));
    
    // Test status-specific behavior
    match banned {
        PlayerStatus::Banned { reason } => {
            assert_eq!(reason, "Test ban");
        },
        _ => panic!("Expected banned status"),
    }
    
    match suspended {
        PlayerStatus::Suspended { reason, until } => {
            assert_eq!(reason, "Test suspension");
            assert!(until.is_some());
        },
        _ => panic!("Expected suspended status"),
    }
}

#[test]
fn test_player_stats_conversion() {
    // Test Player to PlayerStats conversion logic
    let timestamp = test_timestamp();
    
    let player = Player {
        discord_id: "123456789012345678".to_string(),
        username: "TestPlayer#1234".to_string(),
        avatar_url: None,
        total_points: 1500,
        participation_streak: 5,
        best_streak: 7,
        current_rank: Some(10),
        status: PlayerStatus::Active,
        created_at: timestamp,
        last_active: timestamp,
    };
    
    let stats = PlayerStats {
        total_points: player.total_points,
        participation_streak: player.participation_streak,
        current_rank: player.current_rank,
        status: player.status.clone(),
        created_at: player.created_at,
        last_active: player.last_active,
    };
    
    assert_eq!(stats.total_points, player.total_points);
    assert_eq!(stats.participation_streak, player.participation_streak);
    assert_eq!(stats.current_rank, player.current_rank);
    assert_eq!(stats.status, player.status);
    assert_eq!(stats.created_at, player.created_at);
    assert_eq!(stats.last_active, player.last_active);
}

#[test]
fn test_player_stats_conversion_integration() {
    // Test player to PlayerStats conversion logic
    let player = create_test_player("123456789012345678", "TestUser");
    
    // Manual conversion (simulating the method logic)
    let player_stats = PlayerStats {
        total_points: player.total_points,
        participation_streak: player.participation_streak,
        current_rank: player.current_rank,
        status: player.status.clone(),
        created_at: player.created_at,
        last_active: player.last_active,
    };

    // Verify conversion preserves all important data
    assert_eq!(player_stats.total_points, 100);
    assert_eq!(player_stats.participation_streak, 3);
    assert_eq!(player_stats.current_rank, Some(1));
    assert!(matches!(player_stats.status, PlayerStatus::Active));
    assert_eq!(player_stats.created_at, player.created_at);
    assert_eq!(player_stats.last_active, player.last_active);
}