//! Leaderboard Messages Dispatcher
//! 
//! Handles leaderboard-related messages including creation, game notifications, score updates, and flushing.

use crate::contract_domain::handlers::messages::LeaderboardMessageHandler;

/// Dispatcher for leaderboard messages
pub struct LeaderboardMessageDispatcher;

impl LeaderboardMessageDispatcher {
    /// Handle leaderboard creation messages
    pub async fn dispatch_create_leaderboard(
        contract: &mut crate::Game2048Contract,
        leaderboard_id: String,
        name: String,
        description: Option<String>,
        chain_id: String,
        host: String,
        start_time: u64,
        end_time: u64,
    ) {
        LeaderboardMessageHandler::handle_create_leaderboard(contract, leaderboard_id, name, description, chain_id, host, start_time, end_time).await;
    }

    /// Handle new game notifications
    pub async fn dispatch_leaderboard_new_game(
        contract: &mut crate::Game2048Contract,
        player: String,
        board_id: String,
        timestamp: u64,
    ) {
        LeaderboardMessageHandler::handle_leaderboard_new_game(contract, player, board_id, timestamp).await;
    }

    /// Handle score update messages
    pub async fn dispatch_update_score(
        contract: &mut crate::Game2048Contract,
        player: String,
        board_id: String,
        score: u64,
        is_end: bool,
        timestamp: u64,
    ) {
        LeaderboardMessageHandler::handle_update_score(contract, player, board_id, score, is_end, timestamp).await;
    }

    /// Handle flush messages
    pub async fn dispatch_flush(
        contract: &mut crate::Game2048Contract,
        board_ids: std::collections::HashMap<String, String>,
        scores: std::collections::HashMap<String, u64>,
    ) {
        LeaderboardMessageHandler::handle_flush(contract, board_ids, scores).await;
    }
}
