//! Message Dispatcher
//!
//! Main dispatcher for routing messages directly to handlers.

use crate::contract_domain::handlers::messages::{
    GameMessageHandler, LeaderboardMessageHandler, PlayerMessageHandler, TransferMessageHandler,
};
use crate::Message;

/// Main message dispatcher that routes messages directly to handlers
pub struct MessageDispatcher;

impl MessageDispatcher {
    /// Dispatch a message directly to the appropriate handler
    pub async fn dispatch(contract: &mut crate::Game2048Contract, message: Message) {
        match message {
            // Player messages
            Message::RegisterPlayer {
                username,
                password_hash,
            } => {
                PlayerMessageHandler::handle_register_player(contract, username, password_hash)
                    .await;
            }

            // Transfer messages
            Message::Transfer { chain_id, amount } => {
                TransferMessageHandler::handle_transfer(contract, chain_id, amount);
            }

            // Game messages
            Message::CreateNewBoard {
                seed,
                player,
                timestamp,
                leaderboard_id,
                shard_id,
                end_time,
            } => {
                GameMessageHandler::handle_create_new_board(
                    contract,
                    seed,
                    player,
                    timestamp,
                    leaderboard_id,
                    shard_id,
                    end_time,
                )
                .await;
            }

            // Leaderboard messages
            Message::CreateLeaderboard {
                leaderboard_id,
                name,
                description,
                chain_id,
                host,
                start_time,
                end_time,
                shard_ids,
                base_triggerer_count,
                total_shard_count,
            } => {
                LeaderboardMessageHandler::handle_create_leaderboard(
                    contract,
                    leaderboard_id,
                    name,
                    description,
                    chain_id,
                    host,
                    start_time,
                    end_time,
                    shard_ids,
                    base_triggerer_count,
                    total_shard_count,
                )
                .await;
            }
            Message::LeaderboardNewGame {
                player,
                board_id,
                timestamp,
            } => {
                LeaderboardMessageHandler::handle_leaderboard_new_game(
                    contract, player, board_id, timestamp,
                )
                .await;
            }
            Message::UpdateScore {
                player,
                board_id,
                score,
                is_end,
                timestamp,
            } => {
                LeaderboardMessageHandler::handle_update_score(
                    contract, player, board_id, score, is_end, timestamp,
                )
                .await;
            }
            Message::Flush { board_ids, scores } => {
                LeaderboardMessageHandler::handle_flush(contract, board_ids, scores).await;
            }

            // Shard registration message
            Message::RegisterPlayerWithShard {
                player_chain_id,
                tournament_id,
                player_name,
            } => {
                PlayerMessageHandler::handle_register_player_with_shard(
                    contract,
                    player_chain_id.clone(),
                    tournament_id,
                    player_name,
                )
                .await;
            }

            // Aggregation trigger request (delegated triggerer pattern)
            Message::RequestAggregationTrigger {
                requester_chain_id,
                timestamp,
            } => {
                use crate::contract_domain::handlers::operations::LeaderboardOperationHandler;
                if let Err(e) = LeaderboardOperationHandler::handle_aggregation_trigger_request(
                    contract,
                    &requester_chain_id,
                    timestamp,
                )
                .await
                {
                    // Log error but don't panic - unauthorized triggers are expected
                    eprintln!("Aggregation trigger rejected: {}", e);
                }
                // Also trigger self-update
                LeaderboardOperationHandler::update_leaderboard_from_shard_chains(
                    contract,
                    Vec::new(),
                )
                .await;
            }

            // Shard aggregation trigger from leaderboard
            Message::TriggerShardAggregation { timestamp: _ } => {
                use crate::contract_domain::handlers::operations::ShardOperationHandler;
                // Aggregate scores when requested by leaderboard
                ShardOperationHandler::aggregate_scores_from_player_chains(contract, Vec::new())
                    .await;
            }

            // Player chain subscribes to main chain's active tournaments
            Message::SubscribeToMainChain { main_chain_id } => {
                use crate::contract_domain::handlers::messages::PlayerMessageHandler;
                PlayerMessageHandler::handle_subscribe_to_main_chain(contract, main_chain_id).await;
            }

            // Shard registers first player with leaderboard for triggerer system
            Message::RegisterFirstPlayer {
                shard_chain_id,
                player_chain_id,
                tournament_id,
            } => {
                LeaderboardMessageHandler::handle_register_first_player(
                    contract,
                    shard_chain_id,
                    player_chain_id,
                    tournament_id,
                )
                .await;
            }

            // Shard sends multiple trigger candidates to leaderboard
            Message::UpdateShardTriggerCandidates {
                shard_chain_id,
                player_chain_ids,
                tournament_id,
            } => {
                LeaderboardMessageHandler::handle_update_shard_trigger_candidates(
                    contract,
                    shard_chain_id,
                    player_chain_ids,
                    tournament_id,
                )
                .await;
            }

            // Player chain sends trigger update request to leaderboard
            Message::TriggerUpdate {
                triggerer_chain_id,
                tournament_id,
                timestamp,
            } => {
                LeaderboardMessageHandler::handle_trigger_update(
                    contract,
                    triggerer_chain_id,
                    tournament_id,
                    timestamp,
                )
                .await;
            }
        }
    }
}
