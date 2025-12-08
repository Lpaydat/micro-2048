//! Operation Dispatcher
//!
//! Main dispatcher for routing operations directly to handlers.

use crate::contract_domain::handlers::operations::{
    GameOperationHandler, LeaderboardOperationHandler, PlayerOperationHandler,
    SystemOperationHandler,
};
use crate::Operation;

/// Main operation dispatcher that routes operations directly to handlers
pub struct OperationDispatcher;

impl OperationDispatcher {
    /// Dispatch an operation directly to the appropriate handler
    pub async fn dispatch(contract: &mut crate::Game2048Contract, operation: Operation) {
        match operation {
            // Player operations
            Operation::RegisterPlayer {
                username,
                password_hash,
            } => {
                PlayerOperationHandler::handle_register_player(contract, username, password_hash)
                    .await;
            }
            Operation::ToggleAdmin {
                username,
                player,
                password_hash,
            } => {
                PlayerOperationHandler::handle_toggle_admin(
                    contract,
                    username,
                    player,
                    password_hash,
                )
                .await;
            }

            // Game operations
            Operation::MakeMoves {
                board_id,
                moves,
                player,
                password_hash,
            } => {
                GameOperationHandler::handle_make_moves(
                    contract,
                    board_id,
                    moves,
                    player,
                    password_hash,
                )
                .await;
            }
            Operation::NewBoard {
                player,
                timestamp,
                password_hash,
                leaderboard_id,
            } => {
                GameOperationHandler::handle_new_board(
                    contract,
                    player,
                    timestamp,
                    password_hash,
                    leaderboard_id,
                )
                .await;
            }

            // Leaderboard operations
            Operation::LeaderboardAction {
                leaderboard_id,
                action,
                settings,
                player,
                password_hash,
            } => {
                LeaderboardOperationHandler::handle_leaderboard_action(
                    contract,
                    leaderboard_id,
                    action,
                    settings,
                    player,
                    password_hash,
                )
                .await;
            }

            // System operations
            Operation::Faucet => {
                SystemOperationHandler::handle_faucet(contract);
            }
            Operation::NewShard => {
                SystemOperationHandler::handle_new_shard(contract).await;
            }
            Operation::CloseChain { chain_id } => {
                SystemOperationHandler::handle_close_chain(contract, chain_id);
            }

            // Aggregation operations
            Operation::AggregateScores => {
                GameOperationHandler::handle_aggregate_scores(contract).await;
            }
            Operation::UpdateLeaderboard => {
                GameOperationHandler::handle_update_leaderboard(contract).await;
            }

            // Tournament and workload management operations
            Operation::UpdateActiveTournaments => {
                contract.emit_active_tournaments().await;
            }

            // Centralized aggregation request
            Operation::RequestAggregation { requester_chain_id } => {
                let timestamp = contract.runtime.system_time().micros();
                if let Err(e) = contract
                    .handle_aggregation_trigger_request(&requester_chain_id, timestamp)
                    .await
                {
                    panic!("Not authorized to trigger aggregation: {}", e);
                }
            }

            // Admin configuration
            Operation::ConfigureTriggererCount {
                admin_username,
                password_hash,
                base_triggerer_count,
            } => {
                SystemOperationHandler::handle_configure_triggerer_count(
                    contract,
                    admin_username,
                    password_hash,
                    base_triggerer_count,
                )
                .await;
            }

            // Manual leaderboard refresh
            Operation::RequestLeaderboardRefresh {
                player,
                password_hash,
                leaderboard_id,
            } => {
                PlayerOperationHandler::handle_request_leaderboard_refresh(
                    contract,
                    player,
                    password_hash,
                    leaderboard_id,
                )
                .await;
            }

            // Chain pool operations
            Operation::RefillChainPool { count } => {
                SystemOperationHandler::handle_refill_chain_pool(contract, count).await;
            }
            
            // Claim chain - just triggers block production to process inbox
            Operation::ClaimChain => {
                log::info!("⚡ [CLAIM] ClaimChain operation called on chain {}", 
                    contract.runtime.chain_id());
                // No-op operation - just triggers block production which processes inbox messages
                // The RegisterPlayer message in inbox will be processed during this block
            }
        }
    }
}
