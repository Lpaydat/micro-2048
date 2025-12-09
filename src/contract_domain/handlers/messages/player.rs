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
        contract
            .check_player_registered(&username, RegistrationCheck::EnsureNotRegistered)
            .await;

        let player = contract
            .state
            .players
            .load_entry_mut(&username)
            .await
            .unwrap();
        let chain_id = contract.runtime.chain_id().to_string();
        player.username.set(username.clone());
        player.password_hash.set(password_hash);
        player.chain_id.set(chain_id);
    }

    /// Handle subscription to main chain's active tournaments
    pub async fn handle_subscribe_to_main_chain(
        contract: &mut crate::Game2048Contract,
        main_chain_id: String,
    ) {
        use linera_sdk::linera_base_types::{ApplicationId, ChainId, StreamName};
        use std::str::FromStr;

        if let Ok(main_chain_id) = ChainId::from_str(&main_chain_id) {
            let stream_name = StreamName::from("active_tournaments".to_string());
            let application_id = ApplicationId::new(
                contract
                    .runtime
                    .application_id()
                    .application_description_hash,
            );

            contract
                .runtime
                .subscribe_to_events(main_chain_id, application_id, stream_name);
        }
    }
}
