//! Contract Helper Functions
//!
//! Common utility functions and patterns for the Game2048 contract.

use linera_sdk::linera_base_types::{Account, AccountOwner, Amount, ChainId};

/// Contract utility functions
pub struct ContractHelpers;

impl ContractHelpers {
    /// Update the balance state with current chain balance
    pub fn update_balance(contract: &mut crate::Game2048Contract) {
        contract
            .state
            .balance
            .set(contract.runtime.chain_balance().to_string());
    }

    /// Check if this is the main application chain
    pub fn is_main_chain(contract: &mut crate::Game2048Contract) -> bool {
        contract.runtime.chain_id().to_string()
            == contract.runtime.application_creator_chain_id().to_string()
    }

    /// Send a transfer to another chain
    pub fn transfer(contract: &mut crate::Game2048Contract, destination: ChainId, amount: Amount) {
        let account = Account {
            chain_id: destination,
            owner: AccountOwner::CHAIN,
        };
        contract
            .runtime
            .transfer(AccountOwner::CHAIN, account, amount);
    }
}
