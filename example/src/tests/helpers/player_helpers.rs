// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Player test utilities and mock data creation

use crate::{
    core::types::{Player, PlayerStatus, PendingPlayerData, EventScore},
    tests::helpers::timestamp_helpers::test_timestamp,
};

/// Create a basic test player with default values
pub fn create_test_player(discord_id: &str, username: &str) -> Player {
    Player {
        discord_id: discord_id.to_string(),
        username: username.to_string(),
        avatar_url: None,
        total_points: 0,
        participation_streak: 0,
        best_streak: 0,
        current_rank: None,
        status: PlayerStatus::Active,
        created_at: test_timestamp(),
        last_active: test_timestamp(),
    }
}

/// Create a test player with custom values
pub fn create_test_player_with_points(discord_id: &str, username: &str, points: u64) -> Player {
    Player {
        discord_id: discord_id.to_string(),
        username: username.to_string(),
        avatar_url: None,
        total_points: points,
        participation_streak: 0,
        best_streak: 0,
        current_rank: None,
        status: PlayerStatus::Active,
        created_at: test_timestamp(),
        last_active: test_timestamp(),
    }
}

/// Create a test player with avatar URL
pub fn create_test_player_with_avatar(discord_id: &str, username: &str, avatar_url: &str) -> Player {
    Player {
        discord_id: discord_id.to_string(),
        username: username.to_string(),
        avatar_url: Some(avatar_url.to_string()),
        total_points: 0,
        participation_streak: 0,
        best_streak: 0,
        current_rank: None,
        status: PlayerStatus::Active,
        created_at: test_timestamp(),
        last_active: test_timestamp(),
    }
}

/// Create test pending player data
pub fn create_test_pending_data(discord_id: &str, points: u64) -> PendingPlayerData {
    PendingPlayerData {
        discord_id: discord_id.to_string(),
        total_pending_points: points,
        event_scores: vec![
            EventScore {
                event_id: "test_event_1".to_string(),
                game_id: "test_game_1".to_string(),
                score: points,
                participation_timestamp: test_timestamp(),
                streak_eligible: true,
            }
        ],
        first_activity: test_timestamp(),
    }
}

/// Valid Discord ID for testing
pub const VALID_DISCORD_ID: &str = "123456789012345678";

/// Invalid Discord ID for testing
pub const INVALID_DISCORD_ID: &str = "invalid_id";

/// Valid username for testing
pub const VALID_USERNAME: &str = "TestPlayer";

/// Valid avatar URL for testing
pub const VALID_AVATAR_URL: &str = "https://cdn.discordapp.com/avatars/123/avatar.png";