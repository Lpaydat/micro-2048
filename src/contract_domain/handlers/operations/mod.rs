//! Operations Handlers
//! 
//! Business logic handlers for operations organized by domain.

pub mod player;
pub mod game;
pub mod leaderboard;
pub mod shard;
pub mod tournament;
pub mod stream;
pub mod system;

// Re-export handlers for easier access
pub use player::PlayerOperationHandler;
pub use game::GameOperationHandler;
pub use leaderboard::LeaderboardOperationHandler;
pub use shard::ShardOperationHandler;
pub use tournament::TournamentOperationHandler;
pub use stream::StreamProcessingHandler;
pub use system::SystemOperationHandler;
