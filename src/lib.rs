mod direction;
mod elimination_game;
mod game;
mod moves;
mod random;

pub use crate::direction::Direction;
pub use crate::elimination_game::{EliminationGameSettings, MultiplayerGameAction};
pub use crate::game::Game;
pub use crate::moves::{Moves, COL_MASK, ROW_MASK};
pub use crate::random::{hash_seed, rnd_range};
use async_graphql::{Request, Response};
use linera_sdk::{
    base::{ContractAbi, ServiceAbi},
    graphql::GraphQLMutationRoot,
};
use serde::{Deserialize, Serialize};

pub struct Game2048Abi;

impl ContractAbi for Game2048Abi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for Game2048Abi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    ClearMessages,
    RegisterPlayer {
        username: String,
        password_hash: String,
    },
    NewBoard {
        seed: String,
        player: String,
        timestamp: u64,
    },
    EndBoard {
        board_id: String,
    },
    MakeMove {
        board_id: String,
        direction: Direction,
        player: String,
        timestamp: u64,
    },
    // Elimination Game
    CreateEliminationGame {
        player: String,
        settings: EliminationGameSettings,
    },
    EliminationGameAction {
        game_id: String,
        action: MultiplayerGameAction,
        player: String,
        timestamp: u64,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Ping,
    CloseChain,
    // Board {
    //     board_id: String,
    //     board: u64,
    //     score: u64,
    // },
}

pub enum RegistrationCheck {
    EnsureRegistered,
    EnsureNotRegistered,
}
