//! Domain services containing business logic that doesn't naturally fit within a single entity.
//! These services coordinate between multiple domain entities and implement complex business rules.

pub mod game_session_service;
pub mod participant_service;
pub mod competition_service;

// Re-export services for convenience
pub use game_session_service::GameSessionService;
pub use participant_service::ParticipantService;
pub use competition_service::CompetitionService;