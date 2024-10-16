mod direction;
mod moves;
mod random;

pub use crate::direction::Direction;
pub use crate::moves::{Moves, COL_MASK, ROW_MASK};
pub use crate::random::gen_range;
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
    StartGame { seed: u32 },
    EndGame { game_id: u32 },
    MakeMove { game_id: u32, direction: Direction },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Game {
        game_id: u32,
        board: u64,
        score: u64,
        is_ended: bool,
    },
}
