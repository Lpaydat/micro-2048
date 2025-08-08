// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Player registration tests
//! 
//! Tests for player registration logic, data structure creation, and type conversions.

#[cfg(test)]
mod tests {
    use crate::{
        core::types::{Player, PlayerStatus, PlayerStats},
        tests::helpers::*,
    };

    #[test]
    fn test_player_struct_creation() {
        // Test Player struct creation with all fields
        let player = create_test_player(VALID_DISCORD_ID, VALID_USERNAME);
        
        assert_eq!(player.discord_id, VALID_DISCORD_ID);
        assert_eq!(player.username, VALID_USERNAME);
        assert_eq!(player.avatar_url, None);
        assert_eq!(player.total_points, 0);
        assert_eq!(player.participation_streak, 0);
        assert_eq!(player.best_streak, 0);
        assert_eq!(player.current_rank, None);
        assert_eq!(player.status, PlayerStatus::Active);
        assert!(player.created_at.micros() > 0);
        assert!(player.last_active.micros() > 0);
    }

    #[test]
    fn test_player_status_variants() {
        // Test all PlayerStatus enum variants
        let active_player = Player {
            status: PlayerStatus::Active,
            ..create_test_player(VALID_DISCORD_ID, VALID_USERNAME)
        };
        
        let banned_player = Player {
            status: PlayerStatus::Banned { reason: "Test ban".to_string() },
            ..create_test_player("banned_player_id", "BannedPlayer")
        };
        
        let suspended_player = Player {
            status: PlayerStatus::Suspended { reason: "Test suspension".to_string(), until: None },
            ..create_test_player("suspended_player_id", "SuspendedPlayer")
        };
        
        assert_eq!(active_player.status, PlayerStatus::Active);
        match banned_player.status {
            PlayerStatus::Banned { .. } => {}, // Expected
            _ => panic!("Expected Banned status"),
        }
        match suspended_player.status {
            PlayerStatus::Suspended { .. } => {}, // Expected
            _ => panic!("Expected Suspended status"),
        }
    }

    #[test]
    fn test_player_stats_structure() {
        // Test PlayerStats structure matches Player data
        let player = create_test_player_with_points(VALID_DISCORD_ID, VALID_USERNAME, 150);
        
        // Test PlayerStats creation with actual available fields
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
        assert_eq!(stats.status, PlayerStatus::Active);
        assert_eq!(stats.created_at, player.created_at);
        assert_eq!(stats.last_active, player.last_active);
    }

    #[test]
    fn test_player_creation_with_avatar() {
        // Test Player creation with avatar URL
        let player = create_test_player_with_avatar(VALID_DISCORD_ID, VALID_USERNAME, VALID_AVATAR_URL);
        
        assert_eq!(player.discord_id, VALID_DISCORD_ID);
        assert_eq!(player.username, VALID_USERNAME);
        assert_eq!(player.avatar_url, Some(VALID_AVATAR_URL.to_string()));
        assert_eq!(player.status, PlayerStatus::Active);
    }

    #[test]
    fn test_player_creation_with_points() {
        // Test Player creation with initial points
        let initial_points = 250;
        let player = create_test_player_with_points(VALID_DISCORD_ID, VALID_USERNAME, initial_points);
        
        assert_eq!(player.discord_id, VALID_DISCORD_ID);
        assert_eq!(player.username, VALID_USERNAME);
        assert_eq!(player.total_points, initial_points);
        assert_eq!(player.status, PlayerStatus::Active);
    }

    #[test]
    fn test_player_timestamps() {
        // Test that Player timestamps are set correctly
        let player = create_test_player(VALID_DISCORD_ID, VALID_USERNAME);
        
        // Both timestamps should be set
        assert!(player.created_at.micros() > 0);
        assert!(player.last_active.micros() > 0);
        
        // For new players, timestamps should be equal
        assert_eq!(player.created_at, player.last_active);
    }

    #[test]
    fn test_player_streak_initialization() {
        // Test that streak values are initialized correctly
        let player = create_test_player(VALID_DISCORD_ID, VALID_USERNAME);
        
        assert_eq!(player.participation_streak, 0);
        assert_eq!(player.best_streak, 0);
        
        // New players should have no rank initially
        assert_eq!(player.current_rank, None);
    }

    #[test]
    fn test_player_default_status() {
        // Test that new players have Active status by default
        let player = create_test_player(VALID_DISCORD_ID, VALID_USERNAME);
        
        assert_eq!(player.status, PlayerStatus::Active);
        
        // Verify status is not banned or suspended
        assert!(matches!(player.status, PlayerStatus::Active));
        assert!(!matches!(player.status, PlayerStatus::Banned { .. }));
        assert!(!matches!(player.status, PlayerStatus::Suspended { .. }));
    }

    #[test]
    fn test_player_field_types() {
        // Test that all Player fields have correct types
        let player = create_test_player(VALID_DISCORD_ID, VALID_USERNAME);
        
        // String fields
        assert!(player.discord_id.len() > 0);
        assert!(player.username.len() > 0);
        
        // Optional string field
        match player.avatar_url {
            Some(_) | None => {}, // Both variants are valid
        }
        
        // Numeric fields
        assert!(player.total_points >= 0);
        assert!(player.participation_streak >= 0);
        assert!(player.best_streak >= 0);
        
        // Optional numeric field
        match player.current_rank {
            Some(rank) => assert!(rank > 0),
            None => {}, // None is valid for new players
        }
        
        // Enum field
        match player.status {
            PlayerStatus::Active | PlayerStatus::Banned { .. } | PlayerStatus::Suspended { .. } => {},
        }
        
        // Timestamp fields
        assert!(player.created_at.micros() > 0);
        assert!(player.last_active.micros() > 0);
    }
}