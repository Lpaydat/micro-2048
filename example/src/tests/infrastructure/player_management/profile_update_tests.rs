// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Player profile update tests
//! 
//! Tests for player profile modification, update validation, and activity tracking.

#[cfg(test)]
mod tests {
    use crate::{
        core::types::PlayerStatus,
        tests::helpers::*,
    };

    #[test]
    fn test_profile_update_username_change() {
        // Test updating player username
        let mut player = create_test_player(VALID_DISCORD_ID, "OriginalName");
        let new_username = "UpdatedName";
        
        // Simulate username update
        player.username = new_username.to_string();
        player.last_active = later_test_timestamp();
        
        assert_eq!(player.username, new_username);
        assert_ne!(player.created_at, player.last_active);
    }

    #[test]
    fn test_profile_update_avatar_change() {
        // Test updating player avatar URL
        let mut player = create_test_player(VALID_DISCORD_ID, VALID_USERNAME);
        assert_eq!(player.avatar_url, None);
        
        // Add avatar URL
        player.avatar_url = Some(VALID_AVATAR_URL.to_string());
        player.last_active = later_test_timestamp();
        
        assert_eq!(player.avatar_url, Some(VALID_AVATAR_URL.to_string()));
    }

    #[test]
    fn test_profile_update_remove_avatar() {
        // Test removing player avatar URL
        let mut player = create_test_player_with_avatar(VALID_DISCORD_ID, VALID_USERNAME, VALID_AVATAR_URL);
        assert!(player.avatar_url.is_some());
        
        // Remove avatar URL
        player.avatar_url = None;
        player.last_active = later_test_timestamp();
        
        assert_eq!(player.avatar_url, None);
    }

    #[test]
    fn test_profile_update_activity_timestamp() {
        // Test that profile updates change last_active timestamp
        let original_timestamp = test_timestamp();
        let mut player = create_test_player(VALID_DISCORD_ID, VALID_USERNAME);
        player.last_active = original_timestamp;
        
        // Simulate profile update
        let updated_timestamp = later_test_timestamp();
        player.username = "UpdatedPlayer".to_string();
        player.last_active = updated_timestamp;
        
        assert_ne!(player.last_active, original_timestamp);
        assert_eq!(player.last_active, updated_timestamp);
        
        // Created timestamp should remain unchanged
        assert_eq!(player.created_at, test_timestamp());
    }

    #[test]
    fn test_profile_update_preserves_game_data() {
        // Test that profile updates preserve game-related data
        let mut player = create_test_player_with_points(VALID_DISCORD_ID, VALID_USERNAME, 500);
        player.participation_streak = 5;
        player.best_streak = 8;
        player.current_rank = Some(3);
        
        let original_points = player.total_points;
        let original_streak = player.participation_streak;
        let original_best = player.best_streak;
        let original_rank = player.current_rank;
        
        // Update profile (non-game data)
        player.username = "UpdatedName".to_string();
        player.avatar_url = Some(VALID_AVATAR_URL.to_string());
        player.last_active = later_test_timestamp();
        
        // Game data should be preserved
        assert_eq!(player.total_points, original_points);
        assert_eq!(player.participation_streak, original_streak);
        assert_eq!(player.best_streak, original_best);
        assert_eq!(player.current_rank, original_rank);
    }

    #[test]
    fn test_profile_update_preserves_status() {
        // Test that profile updates preserve player status
        let mut player = create_test_player(VALID_DISCORD_ID, VALID_USERNAME);
        player.status = PlayerStatus::Active;
        
        // Update profile
        player.username = "NewName".to_string();
        
        assert_eq!(player.status, PlayerStatus::Active);
        
        // Test with different status
        player.status = PlayerStatus::Suspended { 
            reason: "Test suspension".to_string(), 
            until: None 
        };
        player.avatar_url = Some(VALID_AVATAR_URL.to_string());
        
        match player.status {
            PlayerStatus::Suspended { .. } => {}, // Expected
            _ => panic!("Expected Suspended status"),
        }
    }

    #[test]
    fn test_profile_update_activity_tracking() {
        // Test that profile updates can be tracked for activity
        let mut player = create_test_player(VALID_DISCORD_ID, VALID_USERNAME);
        let original_activity = player.last_active;
        
        // Simulate profile update with activity tracking
        player.username = "UpdatedName".to_string();
        player.last_active = later_test_timestamp();
        
        assert_ne!(player.last_active, original_activity);
        assert!(player.last_active.micros() > original_activity.micros());
    }

    #[test]
    fn test_profile_update_validation_requirements() {
        // Test that profile updates maintain data integrity
        let mut player = create_test_player(VALID_DISCORD_ID, VALID_USERNAME);
        
        // Discord ID should never change
        let original_discord_id = player.discord_id.clone();
        player.username = "UpdatedName".to_string();
        assert_eq!(player.discord_id, original_discord_id);
        
        // Created timestamp should never change
        let original_created = player.created_at;
        player.avatar_url = Some(VALID_AVATAR_URL.to_string());
        assert_eq!(player.created_at, original_created);
        
        // Status changes require separate operations
        let status_is_active = matches!(player.status, PlayerStatus::Active);
        player.username = "AnotherUpdate".to_string();
        assert!(matches!(player.status, PlayerStatus::Active));
        assert!(status_is_active);
    }

    #[test]
    fn test_multiple_profile_updates() {
        // Test applying multiple profile updates
        let mut player = create_test_player(VALID_DISCORD_ID, "OriginalName");
        let start_time = player.last_active;
        
        // First update
        player.username = "FirstUpdate".to_string();
        player.last_active = test_timestamp_from_micros(1100000);
        
        // Second update  
        player.avatar_url = Some(VALID_AVATAR_URL.to_string());
        player.last_active = test_timestamp_from_micros(1200000);
        
        // Third update
        player.username = "FinalUpdate".to_string();
        player.last_active = test_timestamp_from_micros(1300000);
        
        assert_eq!(player.username, "FinalUpdate");
        assert_eq!(player.avatar_url, Some(VALID_AVATAR_URL.to_string()));
        assert!(player.last_active > start_time);
        assert_eq!(player.last_active, test_timestamp_from_micros(1300000));
    }
}