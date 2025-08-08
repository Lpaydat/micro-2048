#![cfg_attr(target_arch = "wasm32", no_main)]

#[path = "minimal_state.rs"]
mod state;

use game2048::Game2048Abi;
use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};

use self::state::Game2048State;

linera_sdk::contract!(Game2048Contract);

pub struct Game2048Contract {
    state: Game2048State,
    runtime: ContractRuntime<Self>,
}

impl WithContractAbi for Game2048Contract {
    type Abi = Game2048Abi;
}

impl Contract for Game2048Contract {
    type Message = ();
    type InstantiationArgument = u64;
    type Parameters = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = Game2048State::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        Game2048Contract { state, runtime }
    }

    async fn instantiate(&mut self, value: u64) {
        self.runtime.application_parameters();
        self.state.participants_count.set(value);
    }

    async fn execute_operation(&mut self, operation: u64) -> u64 {
        let new_value = self.state.participants_count.get() + operation;
        self.state.participants_count.set(new_value);
        new_value
    }

    async fn execute_message(&mut self, _message: ()) {
        panic!("Game2048 application doesn't support any cross-chain messages");
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}