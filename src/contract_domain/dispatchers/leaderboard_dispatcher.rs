//! Leaderboard Operations Dispatcher
//! 
//! Handles leaderboard-related operations including creation, updates, and management.

use game2048::{LeaderboardAction, LeaderboardSettings};
use crate::contract_domain::handlers::operations::LeaderboardOperationHandler;

/// Dispatcher for leaderboard operations
pub struct LeaderboardDispatcher;

impl LeaderboardDispatcher {
    /// Handle leaderboard actions
    pub async fn dispatch_leaderboard_action(
        contract: &mut crate::Game2048Contract,
        leaderboard_id: String,
        action: LeaderboardAction,
        settings: LeaderboardSettings,
        player: String,
        password_hash: String,
    ) {
        LeaderboardOperationHandler::handle_leaderboard_action(contract, leaderboard_id, action, settings, player, password_hash).await;
    }
}
