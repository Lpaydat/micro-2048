#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;
mod contract_domain;

use linera_sdk::{
    linera_base_types::{Account, AccountOwner, Amount, ChainId},
    abi::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};
use state::{Leaderboard, LeaderboardShard};

use self::state::Game2048;
use game2048::{Message, Operation, RegistrationCheck};

pub struct Game2048Contract {
    state: Game2048,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(Game2048Contract);

impl WithContractAbi for Game2048Contract {
    type Abi = game2048::Game2048Abi;
}

impl Contract for Game2048Contract {
    type Message = Message;
    type Parameters = ();
    type InstantiationArgument = u32;
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = Game2048::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        Game2048Contract { state, runtime }
    }

    async fn instantiate(&mut self, _seed: Self::InstantiationArgument) {
        self.runtime.application_parameters();

        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        leaderboard.leaderboard_id.set("".to_string());
        leaderboard
            .chain_id
            .set(self.runtime.chain_id().to_string());
        leaderboard.host.set("".to_string());
        leaderboard.start_time.set(0);
        leaderboard.end_time.set(0);
        leaderboard.name.set("".to_string());
        leaderboard.description.set("".to_string());
        leaderboard.total_boards.set(0);
        leaderboard.total_players.set(0);
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        use crate::contract_domain::OperationDispatcher;
        
        OperationDispatcher::dispatch(self, operation).await;

        self.state
            .balance
            .set(self.runtime.chain_balance().to_string());
    }
    async fn execute_message(&mut self, message: Self::Message) {
        use crate::contract_domain::MessageDispatcher;
        
        MessageDispatcher::dispatch(self, message).await;

        self.state
            .balance
            .set(self.runtime.chain_balance().to_string());
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl Game2048Contract {
    fn is_main_chain(&mut self) -> bool {
        self.runtime.chain_id().to_string()
            == self.runtime.application_creator_chain_id().to_string()
    }

    async fn is_leaderboard_active(&mut self, timestamp: u64) -> &mut Leaderboard {
        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        let start_time = leaderboard.start_time.get();
        let end_time = leaderboard.end_time.get();

        // Basic bounds checking: prevent obviously invalid timestamps
        if timestamp > u64::MAX / 2 {
            panic!("Timestamp too large");
        }

        // Apply timestamp validation to all chains for consistency
        // Keep bypass for system operations (111970) - used for game ending without moves
        if timestamp != 111970
            && (timestamp < *start_time || timestamp > *end_time)
        {
            panic!("Leaderboard is not active");
        }

        leaderboard
    }

    async fn is_shard_active(&mut self, timestamp: u64) -> &mut LeaderboardShard {
        let shard = self.state.shards.load_entry_mut("").await.unwrap();
        let start_time = shard.start_time.get();
        let end_time = shard.end_time.get();

        // Basic bounds checking: prevent obviously invalid timestamps
        if timestamp > u64::MAX / 2 {
            panic!("Timestamp too large");
        }

        // Apply consistent validation to all chains (removed !is_main_chain check)
        // Keep bypass for system operations (111970) - used for game ending without moves
        if timestamp != 111970
            && (timestamp < *start_time || timestamp > *end_time)
        {
            panic!("Shard is not active");
        }

        shard
    }

    async fn update_shard_score(
        &mut self,
        player: &str,
        board_id: String,
        score: u64,
        timestamp: u64,
    ) {
        let shard = self.is_shard_active(timestamp).await;
        let player_shard_score = shard.score.get(player).await.unwrap();

        if player_shard_score.is_none() || player_shard_score < Some(score) {
            shard.score.insert(player, score).unwrap();
            shard.board_ids.insert(player, board_id).unwrap();
            shard.counter.set(*shard.counter.get() + 1);
        }
    }

    fn register_player(&mut self, chain_id: ChainId, player: &str, password_hash: &str) {
        self.runtime
            .prepare_message(Message::RegisterPlayer {
                username: player.to_string(),
                password_hash: password_hash.to_string(),
            })
            .send_to(chain_id);
    }

    async fn upsert_leaderboard(
        &mut self,
        chain_id: ChainId,
        name: &str,
        description: &str,
        host: &str,
        start_time: u64,
        end_time: u64,
        send_to: Option<ChainId>,
    ) {
        self.runtime
            .prepare_message(Message::CreateLeaderboard {
                leaderboard_id: chain_id.to_string(),
                name: name.to_string(),
                description: Some(description.to_string()),
                chain_id: chain_id.to_string(),
                host: host.to_string(),
                start_time,
                end_time,
            })
            .send_to(send_to.unwrap_or(chain_id));
    }

    fn transfer(&mut self, destination: ChainId, amount: Amount) {
        let account = Account {
            chain_id: destination,
            owner: AccountOwner::CHAIN,
        };
        self.runtime.transfer(AccountOwner::CHAIN, account, amount);
    }

    fn auto_faucet(&mut self, faucet_amount: Option<u128>) {
        let current_balance = self.runtime.chain_balance();
        if current_balance.saturating_mul(10) < Amount::from_tokens(5) {
            let app_chain_id = self.runtime.application_creator_chain_id();
            let chain_id = self.runtime.chain_id();

            self.runtime
                .prepare_message(Message::Transfer {
                    chain_id,
                    amount: Amount::from_tokens(faucet_amount.unwrap_or(1)),
                })
                .send_to(app_chain_id);
        }
    }

    fn update_score(
        &mut self,
        chain_id: ChainId,
        player: &str,
        board_id: &str,
        score: u64,
        is_end: bool,
        timestamp: u64,
    ) {
        self.runtime
            .prepare_message(Message::UpdateScore {
                player: player.to_string(),
                board_id: board_id.to_string(),
                score,
                is_end,
                timestamp,
            })
            .send_to(chain_id);
    }

    async fn check_player_registered(
        &mut self,
        player_username: &str,
        check: RegistrationCheck,
    ) -> String {
        let player = self
            .state
            .players
            .load_entry_or_insert(player_username)
            .await
            .unwrap();
        let username = player.username.get();

        let is_registered = !username.trim().is_empty();

        match check {
            RegistrationCheck::EnsureRegistered if !is_registered => {
                panic!("Player not registered");
            }
            RegistrationCheck::EnsureNotRegistered if is_registered => {
                panic!("Player already registered");
            }
            _ => {}
        }

        player.password_hash.get().to_string()
    }

    async fn validate_player_password(&mut self, player_username: &str, provided_password_hash: &str) {
        let stored_password_hash = self.check_player_registered(player_username, RegistrationCheck::EnsureRegistered).await;
        if stored_password_hash != provided_password_hash {
            panic!("Invalid password");
        }
    }
}
