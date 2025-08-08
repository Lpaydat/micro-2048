// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use crate::infrastructure::errors::GameHubError;

/// Result of batch player update processing
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchUpdateResult {
    pub successful_updates: Vec<String>, // Discord IDs of successfully updated players
    pub failed_updates: Vec<BatchUpdateError>, // Failed updates with error details
    pub unregistered_players: Vec<String>, // Discord IDs of unregistered players (stored as pending)
}

/// Error details for failed batch updates
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchUpdateError {
    pub player_discord_id: String,
    pub error: GameHubError, // Proper strongly-typed error
}