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
    ) {
        GameOperationHandler::handle_new_board(contract, player, player_chain_id, timestamp, password_hash).await;
    }
}
