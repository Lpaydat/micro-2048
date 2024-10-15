mod direction;
mod random;

// pub extern crate rand;

pub use crate::direction::Direction;
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
    StartGame {
        init_seed: u32,
    },
    // ForfeitGame {
    //     game_id: u32,
    // },
    MakeMove {
        game_id: u32,
        directions: Vec<Direction>,
    },
}
