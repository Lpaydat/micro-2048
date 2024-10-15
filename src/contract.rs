#![cfg_attr(target_arch = "wasm32", no_main)]

mod game;
mod state;

use crate::game::Game;
use linera_sdk::{
    base::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};

use self::state::Game2048;
use game2048::{gen_range, Operation};

pub struct Game2048Contract {
    state: Game2048,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(Game2048Contract);

impl WithContractAbi for Game2048Contract {
    type Abi = game2048::Game2048Abi;
}

impl Contract for Game2048Contract {
    type Message = ();
    type Parameters = ();
    type InstantiationArgument = u32;

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        log::info!("Hello World!");
        let state = Game2048::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        Game2048Contract { state, runtime }
    }

    async fn instantiate(&mut self, seed: Self::InstantiationArgument) {
        self.runtime.application_parameters();

        log::info!("Instantiating game");

        // Initialize a default game entry if it doesn't exist
        let game_id = seed; // Example game ID
        if self
            .state
            .games
            .load_entry_or_insert(&game_id)
            .await
            .is_err()
        {
            let game = self.state.games.load_entry_mut(&game_id).await.unwrap();
            game.game_id.set(game_id);
            game.board.set(0); // Set a default board value, e.g., an empty board
        }
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            Operation::StartGame { init_seed } => {
                let seed = if init_seed != 0 {
                    init_seed
                } else {
                    let block_height = self.runtime.block_height().to_string();
                    gen_range(&block_height, 0, u32::MAX)
                };
                log::info!("Game ID++: {:?}", seed);
                let new_board = Game::new(seed).board;
                let game = self.state.games.load_entry_mut(&seed).await.unwrap();

                game.game_id.set(seed);
                game.board.set(new_board);
            }
            Operation::MakeMove {
                game_id,
                directions,
            } => {
                let block_height = self.runtime.block_height().to_string();
                let seed = gen_range(&block_height, 0, u32::MAX);
                let board = self.state.games.load_entry_mut(&game_id).await.unwrap();
                let mut game = Game {
                    board: *board.board.get(),
                    seed,
                };
                log::info!("Game board: {:016x}", game.board);
                log::info!("Game ID: {:?}", game_id);
                log::info!("Directions: {:?}", directions);
                let new_board = Game::execute(&mut game, &directions);
                board.board.set(new_board);
            }
        }
    }

    async fn execute_message(&mut self, _message: Self::Message) {}

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

// impl Game2048Contract {
//     fn start_game(&mut self, game_id: u32) {
//         let new_board = Game::new().board;
//         let game = self.state.games.load_entry_mut(&game_id).await.unwrap();

//         game.game_id.set(game_id);
//         game.board.set(new_board);
//     }
// }
