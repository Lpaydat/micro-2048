//! Player Operations Handler
//!
//! Handles player-related operations including registration, authentication, and admin management.

use game2048::{Message, RegistrationCheck};
use linera_sdk::linera_base_types::{Amount, ApplicationPermissions, ChainId};
use std::str::FromStr;

pub struct PlayerOperationHandler;

impl PlayerOperationHandler {
    pub async fn handle_register_player(
        contract: &mut crate::Game2048Contract,
        username: String,
        password_hash: String,
    ) {
        log::info!("🚀 [REGISTER] Starting registration for user: {}", username);
        
        if username.trim().is_empty() {
            panic!("Username cannot be empty");
        }
        let is_main_chain = contract.is_main_chain();
        log::info!("🚀 [REGISTER] Is main chain: {}", is_main_chain);
        if !is_main_chain {
            panic!("Only main chain can register player");
        }

        contract
            .check_player_registered(&username, RegistrationCheck::EnsureNotRegistered)
            .await;
        log::info!("🚀 [REGISTER] Player not registered yet, proceeding...");

        // Check pool size
        let pool_size = contract.state.unclaimed_chains.count();
        log::info!("🚀 [REGISTER] Chain pool size: {}", pool_size);

        // 🚀 CHAIN POOL: Try to claim a pre-created chain from the pool first
        // This is much faster than creating a new chain on-demand
        let chain_id = if let Some(pooled_chain_id) =
            contract.state.unclaimed_chains.front().await.unwrap()
        {
            log::info!("🚀 [REGISTER] Using pooled chain: {}", pooled_chain_id);
            // Pop the chain from the pool
            contract.state.unclaimed_chains.delete_front();
            // Parse the chain ID
            ChainId::from_str(&pooled_chain_id).expect("Invalid chain ID in pool")
        } else {
            log::info!("🚀 [REGISTER] Pool empty, creating new chain...");
            // Fallback: Pool is empty - create a new chain AND refill pool with 50 more
            let chain_ownership = contract.runtime.chain_ownership();
            let application_permissions = ApplicationPermissions::default();
            let amount = Amount::from_tokens(1);

            // Create one chain for this registration
            let new_chain_id = contract.runtime.open_chain(
                chain_ownership.clone(),
                application_permissions.clone(),
                amount,
            );
            log::info!("🚀 [REGISTER] Created new chain: {}", new_chain_id);

            // 🚀 AUTO-REFILL: Create 50 more chains to refill the pool
            // This ensures we don't hit the slow path repeatedly
            log::info!("🚀 [REGISTER] Refilling pool with 50 chains...");
            for i in 0..50 {
                let pool_chain_id = contract.runtime.open_chain(
                    chain_ownership.clone(),
                    application_permissions.clone(),
                    amount,
                );
                contract
                    .state
                    .unclaimed_chains
                    .push_back(pool_chain_id.to_string());
                if i % 10 == 0 {
                    log::info!("🚀 [REGISTER] Created pool chain {}/50", i + 1);
                }
            }
            log::info!("🚀 [REGISTER] Pool refilled");

            new_chain_id
        };

        log::info!("🚀 [REGISTER] Assigning chain {} to user {}", chain_id, username);

        let player = contract
            .state
            .players
            .load_entry_mut(&username)
            .await
            .unwrap();
        player.username.set(username.clone());
        player.password_hash.set(password_hash.clone());
        player.chain_id.set(chain_id.to_string());

        // 🚀 NEW: Set up cross-chain subscription for new player chain
        // Player chains should subscribe to main chain's active_tournaments stream
        let main_chain_id = contract.runtime.application_creator_chain_id();
        log::info!("🚀 [REGISTER] Main chain ID: {}", main_chain_id);

        // Send message to new player chain to subscribe to main chain's tournament events
        log::info!("🚀 [REGISTER] Sending SubscribeToMainChain message to {}", chain_id);
        contract
            .runtime
            .prepare_message(Message::SubscribeToMainChain {
                main_chain_id: main_chain_id.to_string(),
            })
            .with_tracking() // Ensure application is deployed on target chain
            .send_to(chain_id);

        log::info!("🚀 [REGISTER] Sending RegisterPlayer message to {}", chain_id);
        contract.register_player(chain_id, &username, &password_hash);
        
        log::info!("🚀 [REGISTER] Registration complete for user: {}", username);
    }

    pub async fn handle_toggle_admin(
        contract: &mut crate::Game2048Contract,
        username: String,
        player: String,
        password_hash: String,
    ) {
        // Validate password
        contract
            .validate_player_password(&player, &password_hash)
            .await;

        // Additional admin check
        if player != "lpaydat" {
            panic!("Only lpaydat can toggle admin");
        }
        let is_main_chain = contract.is_main_chain();
        if !is_main_chain {
            panic!("Only main chain can toggle admin");
        }

        contract
            .check_player_registered(&username, RegistrationCheck::EnsureRegistered)
            .await;

        let player = contract
            .state
            .players
            .load_entry_mut(&username)
            .await
            .unwrap();
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
        let stored_password_hash = Self::check_player_registered(
            contract,
            player_username,
            RegistrationCheck::EnsureRegistered,
        )
        .await;
        if stored_password_hash != provided_password_hash {
            panic!("Invalid password");
        }
    }

    /// Handle manual leaderboard refresh request from player
    /// Sends TriggerUpdate message directly to leaderboard chain
    pub async fn handle_request_leaderboard_refresh(
        contract: &mut crate::Game2048Contract,
        player: String,
        password_hash: String,
        leaderboard_id: String,
    ) {
        // Validate player password
        Self::validate_player_password(contract, &player, &password_hash).await;

        // Get current time
        let current_time = contract.runtime.system_time().micros();
        let my_chain_id = contract.runtime.chain_id().to_string();

        // Parse leaderboard_id as chain ID and send TriggerUpdate message
        if let Ok(leaderboard_chain_id) = ChainId::from_str(&leaderboard_id) {
            contract
                .runtime
                .prepare_message(Message::TriggerUpdate {
                    triggerer_chain_id: my_chain_id,
                    tournament_id: leaderboard_id.clone(),
                    timestamp: current_time,
                })
                .send_to(leaderboard_chain_id);
        } else {
            panic!("Invalid leaderboard ID format");
        }
    }
}
