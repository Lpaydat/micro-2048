//! Player Messages Dispatcher
//! 
//! Handles player-related messages including registration.

use crate::contract_domain::handlers::messages::PlayerMessageHandler;

/// Dispatcher for player messages
pub struct PlayerMessageDispatcher;

impl PlayerMessageDispatcher {
    /// Handle player registration messages
    pub async fn dispatch_register_player(
        contract: &mut crate::Game2048Contract,
        username: String,
        password_hash: String,
    ) {
        PlayerMessageHandler::handle_register_player(contract, username, password_hash).await;
    }
}
