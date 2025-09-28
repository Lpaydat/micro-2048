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
        player.username.set(username.clone());
        player.password_hash.set(password_hash);
        player.chain_id.set(chain_id.clone());

        // No need to emit events for player registration - 
        // scores will be tracked when players start playing
    }

    /// 🚀 IMPROVED: Handle player registration with shard
    pub async fn handle_register_player_with_shard(
        contract: &mut crate::Game2048Contract,
        player_chain_id: String,
        tournament_id: String,
        player_name: String,
    ) {
        // Use the improved registration method with proper workload tracking
        contract.register_player_with_shard(player_chain_id, tournament_id, player_name).await;
        
        // 🚀 NEW: Emit workload update when new players register
        contract.emit_shard_workload().await;
    }
    
    /// 🚀 NEW: Handle subscription to main chain's active tournaments
    pub async fn handle_subscribe_to_main_chain(
        contract: &mut crate::Game2048Contract,
        main_chain_id: String,
    ) {
        use std::str::FromStr;
        use linera_sdk::linera_base_types::{ChainId, StreamName, ApplicationId};
        
        log::info!("🔔 PLAYER_CHAIN: Subscribing to main chain {} for active_tournaments", main_chain_id);
        
        if let Ok(main_chain_id) = ChainId::from_str(&main_chain_id) {
            let stream_name = StreamName::from("active_tournaments".to_string());
            let application_id = ApplicationId::new(contract.runtime.application_id().application_description_hash);
            
            log::info!("🔔 PLAYER_CHAIN: Setting up subscription - Chain: {}, Stream: '{}', App: {}", 
                main_chain_id, "active_tournaments", application_id);
            
            contract.runtime.subscribe_to_events(main_chain_id, application_id, stream_name);
            
            log::info!("🔔 PLAYER_CHAIN: ✅ Successfully subscribed to active_tournaments from main chain {}", main_chain_id);
        } else {
            log::error!("🔔 PLAYER_CHAIN: ❌ Failed to parse main_chain_id: {}", main_chain_id);
        }
    }
}
