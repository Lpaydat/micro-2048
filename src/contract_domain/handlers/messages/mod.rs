//! Messages Handlers
//! 
//! Business logic handlers for messages organized by domain.

pub mod player;
pub mod transfer;
pub mod game;
pub mod leaderboard;

// Re-export handlers for easier access
pub use player::PlayerMessageHandler;
pub use transfer::TransferMessageHandler;
pub use game::GameMessageHandler;
pub use leaderboard::LeaderboardMessageHandler;
