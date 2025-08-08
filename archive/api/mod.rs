//! API layer providing GraphQL service for external interfaces.
//! This layer maps domain models to GraphQL types and handles query/mutation operations.

pub mod service;

// Re-export API types
pub use service::{Game2048Service, QueryRoot, MutationRoot};