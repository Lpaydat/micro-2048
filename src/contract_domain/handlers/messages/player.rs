//! Player Messages Handler
//! 
//! Handles player-related messages including registration.

use game2048::RegistrationCheck;

pub struct PlayerMessageHandler;

impl PlayerMessageHandler {
    pub async fn handle_register_player(
        contract: &mut crate::Game2048Contract,
        username: String,
        password_hash: String,
    ) {
        contract.check_player_registered(&username, RegistrationCheck::EnsureNotRegistered)
            .await;

        let player = contract.state.players.load_entry_mut(&username).await.unwrap();
        let chain_id = contract.runtime.chain_id().to_string();
        player.username.set(username);
        player.password_hash.set(password_hash);
        player.chain_id.set(chain_id);
    }
}
