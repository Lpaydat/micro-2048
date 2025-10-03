use game2048::Message;
/// System Operations Handler
///
/// Handles system-level operations including faucet, shard management, and chain operations.
use linera_sdk::linera_base_types::{
    Account, AccountOwner, Amount, ApplicationPermissions, ChainId,
};
use std::str::FromStr;

pub struct SystemOperationHandler;

impl SystemOperationHandler {
    pub fn handle_faucet(contract: &mut crate::Game2048Contract) {
        let current_balance = contract.runtime.chain_balance();

        if current_balance.saturating_mul(10) > Amount::from_tokens(2) {
            panic!("Faucet is not available");
        }

        let app_chain_id = contract.runtime.application_creator_chain_id();
        let chain_id = contract.runtime.chain_id();

        contract
            .runtime
            .prepare_message(Message::Transfer {
                chain_id,
                amount: Amount::from_tokens(1),
            })
            .send_to(app_chain_id);
    }

    pub async fn handle_new_shard(contract: &mut crate::Game2048Contract) {
        let leaderboard = contract
            .state
            .leaderboards
            .load_entry_mut("")
            .await
            .unwrap();

        let start_time = *leaderboard.start_time.get();
        let end_time = *leaderboard.end_time.get();

        let chain_ownership = contract.runtime.chain_ownership();
        let app_id = contract.runtime.application_id().forget_abi();
        let application_permissions = ApplicationPermissions::new_single(app_id);
        let amount = Amount::from_tokens(1);
        let shard_id =
            contract
                .runtime
                .open_chain(chain_ownership, application_permissions, amount);

        leaderboard.shard_ids.push_back(shard_id.to_string());
        leaderboard.current_shard_id.set(shard_id.to_string());

        let leaderboard_id = leaderboard.chain_id.get().clone();
        contract
            .upsert_leaderboard(
                ChainId::from_str(&leaderboard_id).unwrap(),
                "",
                "",
                "",
                start_time,
                end_time,
                Some(shard_id),
            )
            .await;
    }

    pub fn handle_close_chain(contract: &mut crate::Game2048Contract, chain_id: String) {
        let chain_id = ChainId::from_str(&chain_id).unwrap();
        let account = Account {
            chain_id,
            owner: AccountOwner::CHAIN,
        };
        let amount = contract
            .runtime
            .chain_balance()
            .saturating_sub(Amount::from_micros(50));
        contract
            .runtime
            .transfer(AccountOwner::CHAIN, account, amount);

        contract
            .runtime
            .close_chain()
            .expect("The application does not have permission to close the chain");
    }

    /// 🚀 ADMIN: Configure base triggerer count
    pub async fn handle_configure_triggerer_count(
        contract: &mut crate::Game2048Contract,
        admin_username: String,
        password_hash: String,
        base_triggerer_count: u32,
    ) {
        // Validate admin credentials
        contract
            .validate_player_password(&admin_username, &password_hash)
            .await;

        // Check if user is admin/mod
        let player = contract
            .state
            .players
            .load_entry_or_insert(&admin_username)
            .await
            .unwrap();

        if !player.is_mod.get() {
            panic!("Only admins can configure triggerer count");
        }

        // Validate count (1-20 range)
        if base_triggerer_count == 0 || base_triggerer_count > 20 {
            panic!("Base triggerer count must be between 1 and 20");
        }

        // Update configuration
        contract
            .state
            .admin_base_triggerer_count
            .set(base_triggerer_count);

        // Also update leaderboard if it exists
        if let Ok(leaderboard) = contract.state.leaderboards.load_entry_mut("").await {
            leaderboard
                .admin_base_triggerer_count
                .set(base_triggerer_count);
        }
    }
}
