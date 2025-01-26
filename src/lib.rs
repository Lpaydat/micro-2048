mod direction;
mod elimination_game;
mod event_leaderboard;
mod game;
mod moves;
mod random;

pub use crate::direction::Direction;
// pub use crate::elimination_game::EliminationGameSettings;
pub use crate::event_leaderboard::{EventLeaderboardAction, EventLeaderboardSettings};
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

#[derive(async_graphql::SimpleObject, Debug, Deserialize, Serialize)]
#[graphql(input_name = "MoveEntryInput")]
struct MoveEntry {
    direction: Direction,
    timestamp: u64,
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    RegisterPlayer {
        username: String,
        password_hash: String,
    },
    NewBoard {
        seed: String,
        player: String,
        timestamp: u64,
        leaderboard_id: Option<String>,
    },
    MakeMoves {
        board_id: String,
        moves: String, // JSON array of MoveEntry
        player: String,
    },
    EventLeaderboardAction {
        leaderboard_id: String,
        action: EventLeaderboardAction,
        settings: EventLeaderboardSettings,
        player: String,
        timestamp: u64,
    },
    ToggleAdmin {
        username: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Ping,
    CloseChain,
    RequestApplication {
        chain_id: String,
    },
    RegisterPlayer {
        username: String,
        password_hash: String,
    },
    EventLeaderboard {
        leaderboard_id: String,
        name: String,
        description: Option<String>,
        chain_id: String,
        host: String,
        start_time: u64,
        end_time: u64,
    },
    LeaderboardNewGame {
        player: String,
        board_id: String,
        timestamp: u64,
    },
    UpdateScore {
        player: String,
        board_id: String,
        score: u64,
        timestamp: u64,
    },
}

pub enum RegistrationCheck {
    EnsureRegistered,
    EnsureNotRegistered,
}
