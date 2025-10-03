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

    /// Auto-faucet if balance is too low
    pub fn auto_faucet(contract: &mut crate::Game2048Contract, faucet_amount: Option<u128>) {
        let current_balance = contract.runtime.chain_balance();
        if current_balance.saturating_mul(10) < Amount::from_tokens(5) {
            let app_chain_id = contract.runtime.application_creator_chain_id();
            let chain_id = contract.runtime.chain_id();

            contract
                .runtime
                .prepare_message(game2048::Message::Transfer {
                    chain_id,
                    amount: Amount::from_tokens(faucet_amount.unwrap_or(1)),
                })
                .send_to(app_chain_id);
        }
    }
}

/// Collection management utilities for specific concrete types used in the codebase
pub struct CollectionHelpers;

impl CollectionHelpers {
    /// Clear a string queue by reading and deleting items until empty
    pub async fn clear_string_queue(
        queue: &mut linera_sdk::views::QueueView<String>,
    ) -> Result<(), linera_sdk::views::ViewError> {
        loop {
            match queue.read_front(1).await {
                Ok(items) if !items.is_empty() => {
                    queue.delete_front();
                }
                _ => break,
            }
        }
        Ok(())
    }

    /// Clear a u32 queue by reading and deleting items until empty
    pub async fn clear_u32_queue(
        queue: &mut linera_sdk::views::QueueView<u32>,
    ) -> Result<(), linera_sdk::views::ViewError> {
        loop {
            match queue.read_front(1).await {
                Ok(items) if !items.is_empty() => {
                    queue.delete_front();
                }
                _ => break,
            }
        }
        Ok(())
    }
}
