//! Message Dispatcher
//!
//! Main dispatcher for routing messages directly to handlers.
//! 
//! 🚀 MESSAGE-BASED ARCHITECTURE: Score updates now use SubmitScore message
//! directly from player chains to leaderboard chain. Shard-related messages
//! are deprecated and made no-ops for backward compatibility.

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
            // ═══════════════════════════════════════════════════════════════
            // ACTIVE MESSAGES (Message-based architecture)
            // ═══════════════════════════════════════════════════════════════
            
            // Player registration
            Message::RegisterPlayer {
                username,
                password_hash,
            } => {
                PlayerMessageHandler::handle_register_player(contract, username, password_hash)
                    .await;
            }

            // Token transfers
            Message::Transfer { chain_id, amount } => {
                TransferMessageHandler::handle_transfer(contract, chain_id, amount);
            }

            // Game board creation
            Message::CreateNewBoard {
                seed,
                player,
                timestamp,
                leaderboard_id,
                shard_id,
                start_time,
                end_time,
                rhythm_track_index,
            } => {
                GameMessageHandler::handle_create_new_board(
                    contract,
                    seed,
                    player,
                    timestamp,
                    leaderboard_id,
                    shard_id,
                    start_time,
                    end_time,
                    rhythm_track_index,
                )
                .await;
            }

            // Leaderboard/tournament creation
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

            // 🚀 PRIMARY: Direct score submission from player to leaderboard
            Message::SubmitScore {
                player,
                player_chain_id,
                board_id,
                score,
                highest_tile,
                game_status,
                timestamp,
                boards_in_tournament,
                start_time,
                end_time,
            } => {
                LeaderboardMessageHandler::handle_submit_score(
                    contract,
                    player,
                    player_chain_id,
                    board_id,
                    score,
                    highest_tile,
                    game_status,
                    timestamp,
                    boards_in_tournament,
                    start_time,
                    end_time,
                )
                .await;
            }

            // Player chain subscribes to main chain's active tournaments
            Message::SubscribeToMainChain { main_chain_id } => {
                PlayerMessageHandler::handle_subscribe_to_main_chain(contract, main_chain_id).await;
            }

            // ═══════════════════════════════════════════════════════════════
            // DEPRECATED MESSAGES (No-ops for backward compatibility)
            // These are kept to process any pending messages in the queue
            // but no longer do anything meaningful.
            // ═══════════════════════════════════════════════════════════════
            
            Message::LeaderboardNewGame { .. } => {
                // DEPRECATED: Board counting now via SubmitScore
            }
            
            Message::UpdateScore { .. } => {
                // DEPRECATED: Use SubmitScore instead
            }
            
            Message::Flush { .. } => {
                // DEPRECATED: No longer using shard flush system
            }

            Message::RegisterPlayerWithShard { .. } => {
                // DEPRECATED: No longer using shard registration
            }

            Message::RequestAggregationTrigger { .. } => {
                // DEPRECATED: No longer using triggerer system
            }

            Message::TriggerShardAggregation { .. } => {
                // DEPRECATED: No longer using shard aggregation
            }

            Message::RegisterFirstPlayer { .. } => {
                // DEPRECATED: No longer using triggerer registration
            }

            Message::UpdateShardTriggerCandidates { .. } => {
                // DEPRECATED: No longer using triggerer candidates
            }

            Message::TriggerUpdate { .. } => {
                // DEPRECATED: No longer using auto-trigger system
            }
        }
    }
}
