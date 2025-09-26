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
}
