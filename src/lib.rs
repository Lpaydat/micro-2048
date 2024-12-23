mod direction;
mod elimination_game;
mod event_leaderboard;
mod game;
mod moves;
mod random;

pub use crate::direction::Direction;
pub use crate::elimination_game::{EliminationGameSettings, MultiplayerGameAction};
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
        action: MultiplayerGameAction,
        player: String,
        requester_chain_id: String,
        timestamp: u64,
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
    CreateEliminationGame {
        player: String,
        host_chain_id: String,
        settings: EliminationGameSettings,
    },
    UpdateEliminationStatus {
        game_id: String,
        status: String,
    },
    CreateEliminationBoard {
        player: String,
        game_id: String,
        round: u8,
        timestamp: u64,
    },
    EndEliminationBoard {
        player: String,
        game_id: String,
        round: u8,
    },
}

pub enum RegistrationCheck {
    EnsureRegistered,
    EnsureNotRegistered,
}
