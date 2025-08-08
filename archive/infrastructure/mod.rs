//! Infrastructure layer handling blockchain-specific concerns using Linera SDK.
//! This layer implements the technical details of blockchain interaction while keeping
//! the core domain logic pure and testable.

pub mod state;
pub mod contract;

// Re-export key infrastructure types
pub use state::GamePlatformState;
pub use contract::{Operation, Message, OperationResponse, Game2048Abi};