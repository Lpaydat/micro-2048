//! Player Operations Dispatcher
//! 
//! Handles player-related operations including registration and admin management.

use crate::contract_domain::handlers::operations::PlayerOperationHandler;

/// Dispatcher for player operations
pub struct PlayerDispatcher;

impl PlayerDispatcher {
    /// Handle player registration
    pub async fn dispatch_register_player(
        contract: &mut crate::Game2048Contract,
        username: String,
        password_hash: String,
    ) {
        PlayerOperationHandler::handle_register_player(contract, username, password_hash).await;
    }

    /// Handle admin toggle operations
    pub async fn dispatch_toggle_admin(
        contract: &mut crate::Game2048Contract,
        username: String,
        player: String,
        password_hash: String,
    ) {
        PlayerOperationHandler::handle_toggle_admin(contract, username, player, password_hash).await;
    }
}
