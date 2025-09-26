//! Game Operations Dispatcher
//! 
//! Handles game-related operations including moves and board creation.

use crate::contract_domain::handlers::operations::GameOperationHandler;

/// Dispatcher for game operations
pub struct GameDispatcher;

impl GameDispatcher {
    /// Handle make moves operations
    pub async fn dispatch_make_moves(
        contract: &mut crate::Game2048Contract,
        board_id: String,
        moves: String,
        player: String,
        password_hash: String,
    ) {
        GameOperationHandler::handle_make_moves(contract, board_id, moves, player, password_hash).await;
    }

    /// Handle new board creation
    pub async fn dispatch_new_board(
        contract: &mut crate::Game2048Contract,
        player: String,
        player_chain_id: String,
        timestamp: u64,
        password_hash: String,
        tournament_id: String, // 🚀 NEW: Tournament ID parameter
    ) {
        GameOperationHandler::handle_new_board(contract, player, player_chain_id, timestamp, password_hash, tournament_id).await;
    }

    /// 🚀 IMPROVED: Handle score aggregation using monitored player chains from state
    pub async fn dispatch_aggregate_scores(
        contract: &mut crate::Game2048Contract,
    ) {
        GameOperationHandler::handle_aggregate_scores(contract).await;
    }

    /// 🚀 IMPROVED: Handle leaderboard update using registered shard chains from state  
    pub async fn dispatch_update_leaderboard(
        contract: &mut crate::Game2048Contract,
    ) {
        GameOperationHandler::handle_update_leaderboard(contract).await;
    }
}
