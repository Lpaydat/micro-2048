// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use linera_sdk::graphql::GraphQLMutationRoot;
use crate::core::types::ScoringConfig;

/// Operations that can be executed on the GameHub contract
#[derive(Debug, Serialize, Deserialize, GraphQLMutationRoot)]
pub enum Operation {
    /// Player operations
    RegisterPlayer {
        discord_id: String,
        username: String,
        avatar_url: Option<String>,
    },

    /// Admin operations
    ApproveGame {
        caller_discord_id: String,
        game_id: String,
    },

    RejectGame {
        caller_discord_id: String,
        game_id: String,
        reason: String,
    },

    /// Moderation operations
    BanPlayer {
        caller_discord_id: String,
        player_discord_id: String,
        reason: String,
    },

    SuspendPlayer {
        caller_discord_id: String,
        player_discord_id: String,
        reason: String,
        duration_hours: Option<u32>,
    },

    UnbanPlayer {
        caller_discord_id: String,
        player_discord_id: String,
    },

    UnsuspendPlayer {
        caller_discord_id: String,
        player_discord_id: String,
    },

    /// Permission management
    AssignModerator {
        caller_discord_id: String,
        discord_id: String,
    },

    RemoveModerator {
        caller_discord_id: String,
        discord_id: String,
    },

    /// Update player profile
    UpdatePlayerProfile {
        discord_id: String,
        username: Option<String>,
        avatar_url: Option<String>,
    },

    /// Game management
    SuspendGame {
        caller_discord_id: String,
        game_id: String,
        reason: String,
    },

    ReactivateGame {
        caller_discord_id: String,
        game_id: String,
    },

    DeprecateGame {
        caller_discord_id: String,
        game_id: String,
    },

    /// Update scoring configuration
    UpdateScoringConfig {
        caller_discord_id: String,
        config: ScoringConfig,
    },

    /// Admin management operations
    AddAdmin {
        caller_discord_id: String,
        discord_id: String,
    },

    RemoveAdmin {
        caller_discord_id: String,
        discord_id: String,
    },

    /// Event management operations
    CreateEvent {
        caller_discord_id: String,
        game_id: String,
        name: String,
        description: String,
        start_time: u64, // Timestamp in microseconds
        end_time: u64,   // Timestamp in microseconds
        is_mandatory: bool,
        max_participants: Option<u32>,
        prize_pool: Option<u64>,
    },

    UpdateEvent {
        caller_discord_id: String,
        event_id: String,
        name: Option<String>,
        description: Option<String>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        is_mandatory: Option<bool>,
        max_participants: Option<u32>,
        prize_pool: Option<u64>,
    },

    SetEventMandatory {
        caller_discord_id: String,
        event_id: String,
        is_mandatory: bool,
    },

    /// Import/Export operations for CSV functionality
    ImportLeaderboardData {
        caller_discord_id: String,
        csv_data: String,
    },
}