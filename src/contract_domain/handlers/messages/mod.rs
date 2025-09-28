//! Messages Handlers
//!
//! Business logic handlers for messages organized by domain.

pub mod game;
pub mod leaderboard;
pub mod player;
pub mod transfer;

// Re-export handlers for easier access
pub use game::GameMessageHandler;
pub use leaderboard::LeaderboardMessageHandler;
pub use player::PlayerMessageHandler;
pub use transfer::TransferMessageHandler;
