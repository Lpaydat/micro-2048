// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use linera_sdk::linera_base_types::Timestamp;

/// Audit log entry for administrative actions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditLogEntry {
    pub id: String, // Unique identifier for the log entry
    pub action: AdminAction,
    pub performed_by: String, // Discord ID of the admin/moderator who performed the action
    pub target: Option<String>, // Discord ID or Game ID that was affected (if applicable)
    pub timestamp: Timestamp,
    pub details: Option<String>, // Additional details about the action
}

/// Types of administrative actions that can be logged
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
    PlayerUnsuspended { player_id: String },
    
    // Permission management
    ModeratorAssigned { moderator_id: String },
    ModeratorRemoved { moderator_id: String },
    AdminAdded { admin_id: String },
    AdminRemoved { admin_id: String },
    
    // Configuration changes
    ScoringConfigUpdated,
    
    // Event management
    EventCreated { event_id: String, game_id: String },
    EventUpdated { event_id: String },
    
    // Other administrative actions
    PlayerProfileUpdated { player_id: String },
    CsvDataImported { records_processed: u32, successful: u32, failed: u32 },
}