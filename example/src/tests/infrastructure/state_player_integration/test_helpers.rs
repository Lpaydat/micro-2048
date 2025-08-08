// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Test helper functions for state player integration tests
//! 
//! Shared utility functions used across state player integration test modules.

use crate::core::types::{Player, PlayerStatus, PendingPlayerData, EventScore, AuditLogEntry, AdminAction};
use linera_sdk::linera_base_types::Timestamp;

/// Helper function to create a test timestamp
pub fn test_timestamp() -> Timestamp {
    Timestamp::from(1000000)
}

/// Create a test player with specified parameters
pub fn create_test_player(discord_id: &str, username: &str) -> Player {
    let timestamp = test_timestamp();
    
    Player {
        discord_id: discord_id.to_string(),
        username: username.to_string(),
        avatar_url: Some("https://cdn.discordapp.com/avatars/123456789012345678/avatar.png".to_string()),
        total_points: 100,
        participation_streak: 3,
        best_streak: 5,
        current_rank: Some(1),
        status: PlayerStatus::Active,
        created_at: timestamp,
        last_active: timestamp,
    }
}

/// Create test pending player data with multiple event scores
pub fn create_test_pending_data(discord_id: &str) -> PendingPlayerData {
    let base_timestamp = test_timestamp();
    
    let event_scores = vec![
        EventScore {
            event_id: "event_1".to_string(),
            game_id: "game_1".to_string(),
            score: 100,
            streak_eligible: true,
            participation_timestamp: base_timestamp,
        },
        EventScore {
            event_id: "event_2".to_string(),
            game_id: "game_2".to_string(),
            score: 150,
            streak_eligible: true,
            participation_timestamp: Timestamp::from(base_timestamp.micros() + 1000),
        },
    ];
    
    PendingPlayerData {
        discord_id: discord_id.to_string(),
        total_pending_points: 250,
        event_scores,
        first_activity: base_timestamp,
    }
}

/// Create test audit log entry
pub fn create_test_audit_log_entry(action: AdminAction, performed_by: &str) -> AuditLogEntry {
    AuditLogEntry {
        id: "test_audit_123".to_string(),
        action,
        performed_by: performed_by.to_string(),
        target: None,
        timestamp: test_timestamp(),
        details: Some("Test audit log entry".to_string()),
    }
}