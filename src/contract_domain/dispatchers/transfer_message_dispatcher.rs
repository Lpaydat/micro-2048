//! Transfer Messages Dispatcher
//! 
//! Handles transfer-related messages including token transfers.

use linera_sdk::linera_base_types::{Amount, ChainId};
use crate::contract_domain::handlers::messages::TransferMessageHandler;

/// Dispatcher for transfer messages
pub struct TransferMessageDispatcher;

impl TransferMessageDispatcher {
    /// Handle transfer messages
    pub fn dispatch_transfer(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
        amount: Amount,
    ) {
        TransferMessageHandler::handle_transfer(contract, chain_id, amount);
    }
}
