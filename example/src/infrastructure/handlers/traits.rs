// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Common traits and utilities for operation handlers

use crate::core::types::EventType;
use crate::infrastructure::state::GameHubState;
use linera_sdk::linera_base_types::Timestamp;

/// Base trait for operation handlers
pub trait OperationHandler {
    type Operation;
    type Result;
    
    async fn handle<T>(
        contract: &mut T,
        operation: Self::Operation
    ) -> Self::Result
    where
        T: ContractInterface;
}

/// Trait that defines what handlers need from the contract
pub trait ContractInterface {
    fn get_state(&mut self) -> &mut GameHubState;
    fn get_timestamp(&mut self) -> Timestamp;
    fn log_event(&mut self, event_type: EventType, description: String, actor_id: Option<String>, target_id: Option<String>);
}

/// Helper methods available to all operation handlers
pub struct HandlerUtils;

impl HandlerUtils {
    /// Log an event with consistent format
    pub fn log_event<T: ContractInterface>(
        contract: &mut T,
        event_type: EventType,
        description: String,
        actor_id: Option<String>,
        target_id: Option<String>,
    ) {
        contract.log_event(event_type, description, actor_id, target_id);
    }
    
    /// Get current blockchain timestamp
    pub fn get_timestamp<T: ContractInterface>(contract: &mut T) -> Timestamp {
        contract.get_timestamp()
    }
    
    /// Format success response with consistent pattern
    pub fn success_response(operation: &str, target: &str, details: Option<&str>) -> String {
        match details {
            Some(details) => format!("{} {} successfully: {}", operation, target, details),
            None => format!("{} {} successfully", operation, target),
        }
    }
    
    /// Format error response with consistent pattern
    pub fn error_response(operation: &str, error: &str) -> String {
        format!("Error {}: {}", operation, error)
    }
}