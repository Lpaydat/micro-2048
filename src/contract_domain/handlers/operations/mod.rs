//! Operations Handlers
//!
//! Business logic handlers for operations organized by domain.

pub mod game;
pub mod leaderboard;
pub mod player;
pub mod shard;
pub mod system;
pub mod tournament;

// Re-export handlers for easier access
pub use game::GameOperationHandler;
pub use leaderboard::LeaderboardOperationHandler;
pub use player::PlayerOperationHandler;
pub use shard::ShardOperationHandler;
pub use system::SystemOperationHandler;
pub use tournament::TournamentOperationHandler;
