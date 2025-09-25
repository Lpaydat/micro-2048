//! Player Operations Handler
//! 
//! Handles player-related operations including registration and admin management.

use linera_sdk::{
    linera_base_types::{Amount, ApplicationPermissions},
};
use game2048::RegistrationCheck;

pub struct PlayerOperationHandler;

impl PlayerOperationHandler {
    pub async fn handle_register_player(
        contract: &mut crate::Game2048Contract,
        username: String,
        password_hash: String,
    ) {
        if username.trim().is_empty() {
            panic!("Username cannot be empty");
        }
        let is_main_chain = contract.is_main_chain();
        if !is_main_chain {
            panic!("Only main chain can register player");
        }

        contract.check_player_registered(&username, RegistrationCheck::EnsureNotRegistered)
            .await;

        let chain_ownership = contract.runtime.chain_ownership();
        let application_permissions = ApplicationPermissions::default();
        let amount = Amount::from_tokens(1);
        let chain_id = contract.runtime.open_chain(chain_ownership, application_permissions, amount);

        let player = contract.state.players.load_entry_mut(&username).await.unwrap();
        player.username.set(username.clone());
        player.password_hash.set(password_hash.clone());
        player.chain_id.set(chain_id.to_string());

        contract.register_player(chain_id, &username, &password_hash);
    }

    pub async fn handle_toggle_admin(
        contract: &mut crate::Game2048Contract,
        username: String,
        player: String,
        password_hash: String,
    ) {
        // Validate password
        contract.validate_player_password(&player, &password_hash).await;
        
        // Additional admin check
        if player != "lpaydat" {
            panic!("Only lpaydat can toggle admin");
        }
        let is_main_chain = contract.is_main_chain();
        if !is_main_chain {
            panic!("Only main chain can toggle admin");
        }

        contract.check_player_registered(&username, RegistrationCheck::EnsureRegistered)
            .await;

        let player = contract.state.players.load_entry_mut(&username).await.unwrap();
        player.is_mod.set(!*player.is_mod.get());
    }
}
