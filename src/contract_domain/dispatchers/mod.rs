//! Operation and Message Dispatchers
//! 
//! Modular dispatcher system for organizing operations and messages by business domain.

mod player_dispatcher;
mod game_dispatcher;
mod leaderboard_dispatcher;
mod system_dispatcher;
mod message_dispatcher;
mod player_message_dispatcher;
mod transfer_message_dispatcher;
mod game_message_dispatcher;
mod leaderboard_message_dispatcher;

pub use player_dispatcher::PlayerDispatcher;
pub use game_dispatcher::GameDispatcher;
pub use leaderboard_dispatcher::LeaderboardDispatcher;
pub use system_dispatcher::SystemDispatcher;
pub use message_dispatcher::MessageDispatcher;
pub use player_message_dispatcher::PlayerMessageDispatcher;
pub use transfer_message_dispatcher::TransferMessageDispatcher;
pub use game_message_dispatcher::GameMessageDispatcher;
pub use leaderboard_message_dispatcher::LeaderboardMessageDispatcher;

use crate::Operation;

/// Main operation dispatcher that routes operations to specialized dispatchers
pub struct OperationDispatcher;

impl OperationDispatcher {
    /// Dispatch an operation to the appropriate specialized dispatcher
    pub async fn dispatch(
        contract: &mut crate::Game2048Contract,
        operation: Operation
    ) {
        match operation {
            // Player operations
            Operation::RegisterPlayer { username, password_hash } => {
                PlayerDispatcher::dispatch_register_player(contract, username, password_hash).await;
            }
            Operation::ToggleAdmin { username, player, password_hash } => {
                PlayerDispatcher::dispatch_toggle_admin(contract, username, player, password_hash).await;
            }
            
            // Game operations
            Operation::MakeMoves { board_id, moves, player, password_hash } => {
                GameDispatcher::dispatch_make_moves(contract, board_id, moves, player, password_hash).await;
            }
            Operation::NewBoard { player, player_chain_id, timestamp, password_hash } => {
                GameDispatcher::dispatch_new_board(contract, player, player_chain_id, timestamp, password_hash).await;
            }
            
            // Leaderboard operations
            Operation::LeaderboardAction { leaderboard_id, action, settings, player, password_hash } => {
                LeaderboardDispatcher::dispatch_leaderboard_action(contract, leaderboard_id, action, settings, player, password_hash).await;
            }
            
            // System operations
            Operation::Faucet => {
                SystemDispatcher::dispatch_faucet(contract);
            }
            Operation::NewShard => {
                SystemDispatcher::dispatch_new_shard(contract).await;
            }
            Operation::CloseChain { chain_id } => {
                SystemDispatcher::dispatch_close_chain(contract, chain_id);
            }
        }
    }
}
