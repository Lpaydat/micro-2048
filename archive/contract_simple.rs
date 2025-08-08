//! Minimal contract for testing deployment

#![cfg_attr(target_arch = "wasm32", no_main)]

use linera_sdk::{
    linera_base_types::WithContractAbi,
    Contract, ContractRuntime,
};

use game2048::infrastructure::contract::Game2048Abi;

linera_sdk::contract!(Game2048Contract);

pub struct Game2048Contract {
    runtime: ContractRuntime<Self>,
}

impl WithContractAbi for Game2048Contract {
    type Abi = Game2048Abi;
}

impl Contract for Game2048Contract {
    type Message = ();
    type InstantiationArgument = ();
    type Parameters = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        Game2048Contract { runtime }
    }

    async fn instantiate(&mut self, _argument: Self::InstantiationArgument) {
        // Minimal initialization
    }

    async fn execute_operation(&mut self, _operation: String) -> String {
        "OK".to_string()
    }

    async fn execute_message(&mut self, _message: ()) {
        // No messages supported
    }

    async fn store(self) {
        // Minimal store
    }
}