//! Player Operations Handler
//! 
//! Handles player-related operations including registration, authentication, and admin management.

use linera_sdk::{
    linera_base_types::{Amount, ApplicationPermissions},
};
use game2048::{RegistrationCheck, Message};

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

        // 🚀 NEW: Set up cross-chain subscription for new player chain
        // Player chains should subscribe to main chain's active_tournaments stream
        let main_chain_id = contract.runtime.application_creator_chain_id();
        log::info!("🔔 PLAYER_REGISTER: Setting up subscription from new player chain {} to main chain {} for active_tournaments", 
            chain_id, main_chain_id);
        
        // Send message to new player chain to subscribe to main chain's tournament events
        contract.runtime
            .prepare_message(Message::SubscribeToMainChain {
                main_chain_id: main_chain_id.to_string(),
            })
            .send_to(chain_id);

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

    /// Check if player is registered and optionally validate registration status
    pub async fn check_player_registered(
        contract: &mut crate::Game2048Contract,
        player_username: &str,
        check: RegistrationCheck,
    ) -> String {
        let player = contract
            .state
            .players
            .load_entry_or_insert(player_username)
            .await
            .unwrap();
        let username = player.username.get();

        let is_registered = !username.trim().is_empty();

        match check {
            RegistrationCheck::EnsureRegistered if !is_registered => {
                panic!("Player not registered");
            }
            RegistrationCheck::EnsureNotRegistered if is_registered => {
                panic!("Player already registered");
            }
            _ => {}
        }

        player.password_hash.get().to_string()
    }

    /// Validate player password against stored hash
    pub async fn validate_player_password(
        contract: &mut crate::Game2048Contract,
        player_username: &str,
        provided_password_hash: &str,
    ) {
        let stored_password_hash = Self::check_player_registered(contract, player_username, RegistrationCheck::EnsureRegistered).await;
        if stored_password_hash != provided_password_hash {
            panic!("Invalid password");
        }
    }
}
