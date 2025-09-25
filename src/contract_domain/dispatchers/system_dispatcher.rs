//! System Operations Dispatcher
//! 
//! Handles system-level operations including faucet, shard management, and chain operations.

use crate::contract_domain::handlers::operations::SystemOperationHandler;

/// Dispatcher for system operations
pub struct SystemDispatcher;

impl SystemDispatcher {
    /// Handle faucet operations
    pub fn dispatch_faucet(contract: &mut crate::Game2048Contract) {
        SystemOperationHandler::handle_faucet(contract);
    }

    /// Handle new shard creation
    pub async fn dispatch_new_shard(contract: &mut crate::Game2048Contract) {
        SystemOperationHandler::handle_new_shard(contract).await;
    }

    /// Handle chain closing
    pub fn dispatch_close_chain(contract: &mut crate::Game2048Contract, chain_id: String) {
        SystemOperationHandler::handle_close_chain(contract, chain_id);
    }
}
