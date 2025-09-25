//! Game Messages Dispatcher
//! 
//! Handles game-related messages including board creation.

use crate::contract_domain::handlers::messages::GameMessageHandler;

/// Dispatcher for game messages
pub struct GameMessageDispatcher;

impl GameMessageDispatcher {
    /// Handle new board creation messages
    pub async fn dispatch_create_new_board(
        contract: &mut crate::Game2048Contract,
        seed: String,
        player: String,
        timestamp: u64,
        leaderboard_id: String,
        shard_id: String,
        end_time: u64,
    ) {
        GameMessageHandler::handle_create_new_board(contract, seed, player, timestamp, leaderboard_id, shard_id, end_time).await;
    }
}
