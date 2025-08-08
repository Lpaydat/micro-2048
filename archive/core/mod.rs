//! Core domain layer containing pure business logic with no infrastructure dependencies.
//! This layer implements the domain models, services, and validators following DDD principles.

pub mod models;
pub mod services;
pub mod validators;
pub mod value_objects;
pub mod game_logic;

// Re-export core types for convenience
pub use models::*;
pub use value_objects::*;
pub use game_logic::Moves;