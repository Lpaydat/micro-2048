// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use crate::core::types::{PendingGame, LeaderboardEntry, PlayerEventUpdate};
use linera_sdk::linera_base_types::Timestamp;

/// Messages for cross-chain communication
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Game registration from other chains
    RegisterGame {
        game_info: PendingGame,
    },
    
    /// Comprehensive batch event update message for advanced cross-chain communication
    BatchEventUpdate {
        event_id: String,
        game_id: String,
        player_updates: Vec<PlayerEventUpdate>,
        final_leaderboard: Vec<LeaderboardEntry>,
        update_timestamp: Timestamp,
    },
}

/// Admin action types with structured data for comprehensive logging
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AdminAction {
    // Game management
    GameApproved { game_id: String, game_name: String },
    GameRejected { game_id: String, reason: String },
    GameSuspended { game_id: String, reason: String },
    GameReactivated { game_id: String },
    GameDeprecated { game_id: String },
    
    // Player moderation
    PlayerBanned { player_id: String, reason: String },
    PlayerSuspended { player_id: String, reason: String, duration_hours: Option<u32> },
    PlayerUnbanned { player_id: String },
    
    // Permission management
    ModeratorAssigned { moderator_id: String },
    ModeratorRemoved { moderator_id: String },
    AdminAdded { admin_id: String },
    AdminRemoved { admin_id: String },
    
    // Configuration changes
    ScoringConfigUpdated,
    
    // Other administrative actions
    PlayerProfileUpdated { player_id: String },
}

