// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

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
// ANCHOR_END: declare_abi

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
        // Validate that the application parameters were configured correctly.
        self.runtime.application_parameters();

        self.state.value.set(value);
    }

    async fn execute_operation(&mut self, operation: u64) -> u64 {
        // Simple ping operation - just return the operation value
        operation
    }

    async fn execute_message(&mut self, _message: ()) {
        panic!("Game2048 application doesn't support any cross-chain messages");
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

#[cfg(test)]
mod tests {
    use futures::FutureExt as _;
    use linera_sdk::{util::BlockingWait, views::View, Contract, ContractRuntime};

    use super::{Game2048Contract, Game2048State};

    #[test]
    fn ping_operation() {
        let runtime = ContractRuntime::new().with_application_parameters(());
        let state = Game2048State::load(runtime.root_view_storage_context())
            .blocking_wait()
            .expect("Failed to read from mock key value store");
        let mut contract = Game2048Contract { state, runtime };

        let initial_value = 42_u64;
        contract
            .instantiate(initial_value)
            .now_or_never()
            .expect("Initialization should not await anything");

        let ping_value = 123_u64;
        let response = contract
            .execute_operation(ping_value)
            .now_or_never()
            .expect("Execution should not await anything");

        assert_eq!(response, ping_value);
        assert_eq!(*contract.state.value.get(), initial_value);
    }
}


