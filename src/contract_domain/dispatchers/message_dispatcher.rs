//! Message Dispatcher
//! 
//! Main dispatcher for routing messages to specialized handlers.

use crate::Message;

/// Main message dispatcher that routes messages to specialized dispatchers
pub struct MessageDispatcher;

impl MessageDispatcher {
    /// Dispatch a message to the appropriate specialized dispatcher
    pub async fn dispatch(
        contract: &mut crate::Game2048Contract,
        message: Message
    ) {
        match message {
            // Player messages
            Message::RegisterPlayer { username, password_hash } => {
                crate::contract_domain::dispatchers::PlayerMessageDispatcher::dispatch_register_player(contract, username, password_hash).await;
            }
            
            // Transfer messages
            Message::Transfer { chain_id, amount } => {
                crate::contract_domain::dispatchers::TransferMessageDispatcher::dispatch_transfer(contract, chain_id, amount);
            }
            
            // Game messages
            Message::CreateNewBoard { seed, player, timestamp, leaderboard_id, shard_id, end_time } => {
                crate::contract_domain::dispatchers::GameMessageDispatcher::dispatch_create_new_board(contract, seed, player, timestamp, leaderboard_id, shard_id, end_time).await;
            }
            
            // Leaderboard messages
            Message::CreateLeaderboard { leaderboard_id, name, description, chain_id, host, start_time, end_time, shard_ids } => {
                crate::contract_domain::dispatchers::LeaderboardMessageDispatcher::dispatch_create_leaderboard(contract, leaderboard_id, name, description, chain_id, host, start_time, end_time, shard_ids).await;
            }
            Message::LeaderboardNewGame { player, board_id, timestamp } => {
                crate::contract_domain::dispatchers::LeaderboardMessageDispatcher::dispatch_leaderboard_new_game(contract, player, board_id, timestamp).await;
            }
            Message::UpdateScore { player, board_id, score, is_end, timestamp } => {
                crate::contract_domain::dispatchers::LeaderboardMessageDispatcher::dispatch_update_score(contract, player, board_id, score, is_end, timestamp).await;
            }
            Message::Flush { board_ids, scores } => {
                crate::contract_domain::dispatchers::LeaderboardMessageDispatcher::dispatch_flush(contract, board_ids, scores).await;
            }
            
            // 🚀 NEW: Shard registration message
            Message::RegisterPlayerWithShard { player_chain_id, tournament_id, player_name } => {
                crate::contract_domain::handlers::messages::PlayerMessageHandler::handle_register_player_with_shard(contract, player_chain_id, tournament_id, player_name).await;
            }
            
            // 🚀 NEW: Aggregation trigger request (delegated triggerer pattern)
            Message::RequestAggregationTrigger { requester_chain_id, timestamp } => {
                if let Err(e) = contract.handle_aggregation_trigger_request(&requester_chain_id, timestamp).await {
                    // Log error but don't panic - unauthorized triggers are expected
                    eprintln!("Aggregation trigger rejected: {}", e);
                }
                // Also trigger self-update
                contract.update_leaderboard_from_shard_chains(Vec::new()).await;
            }
            
            // 🚀 NEW: Shard aggregation trigger from leaderboard
            Message::TriggerShardAggregation { timestamp: _ } => {
                // Aggregate scores when requested by leaderboard
                contract.aggregate_scores_from_player_chains(Vec::new()).await;
            }
        }
    }
}
