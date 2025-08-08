// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use linera_sdk::linera_base_types::Timestamp;

/// Game status enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]  
pub enum GameStatus {
    Pending,
    Active,
    Suspended { reason: String },
    Deprecated,
}

/// Developer information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeveloperInfo {
    pub name: String,
    pub contact: String,
}

/// Enhanced game structure  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub description: String,
    pub contract_address: String,
    pub developer_info: DeveloperInfo,
    pub status: GameStatus,
    pub approved_by: Option<String>, // Admin Discord ID
    pub created_at: Timestamp,
    pub approved_at: Option<Timestamp>,
}

/// Pending game awaiting approval
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingGame {
    pub id: String,
    pub name: String,
    pub description: String,
    pub contract_address: String,
    pub developer_info: DeveloperInfo,
    pub created_at: Timestamp,
}