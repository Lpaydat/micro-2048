// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/*! ABI of the GameHub Application */

use async_graphql::{Request, Response};
use linera_sdk::linera_base_types::{ContractAbi, ServiceAbi};

// Core business logic modules
pub mod core;

// Infrastructure layer modules  
pub mod infrastructure;

// API layer modules
pub mod api;

// Test modules
#[cfg(test)]
pub mod tests;

// Re-export commonly used types and functionality
pub use core::types::{Player, PlayerStatus, PendingGame, DeveloperInfo};
pub use infrastructure::{operations::Operation, messages::Message, state::GameHubState};

// Re-export service runtime for mutations (without exposing full service implementation)
pub use linera_sdk::ServiceRuntime;

/// Trait for scheduling operations on the runtime
pub trait ScheduleOperation: Send + Sync {
    fn schedule_operation(&self, operation: &Operation);
}

/// Type aliases for runtime integration
/// The GameHubService implementation is defined in service.rs
/// Mutations use the ScheduleOperation trait instead of direct service access

/// The GameHub application ABI
pub struct GameHubAbi;

impl ContractAbi for GameHubAbi {
    type Operation = Operation;
    type Response = String;
}

impl ServiceAbi for GameHubAbi {
    type Query = Request;
    type QueryResponse = Response;
}
