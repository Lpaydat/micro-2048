//! Transfer Messages Handler
//! 
//! Handles transfer-related messages including token transfers.

use linera_sdk::linera_base_types::{Amount, ChainId};

pub struct TransferMessageHandler;

impl TransferMessageHandler {
    pub fn handle_transfer(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
        amount: Amount,
    ) {
        contract.transfer(chain_id, amount);
    }
}
