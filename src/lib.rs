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
        game_id: String,
        action: MultiplayerGameAction,
        player: String,
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
    // the idea is to make the main chain send a message to the newly created chain
    // this message should contain the new chainId and the applicationId
    // when this message displays on "linera net up" panel, it will be catched by the script
    // the script then execute the command to request the application for the new chainId
    // see https://chatgpt.com/c/675ca60a-27ac-8011-89d5-0a51f4839fcf
    RequestApplication {
        chain_id: String,
    },
    RegisterPlayer {
        username: String,
        password_hash: String,
    },
    UpdateClassicLeaderboard {
        player: String,
        board_id: String,
        leaderboard_id: Option<String>,
        score: u64,
        timestamp: u64,
    },
}

pub enum RegistrationCheck {
    EnsureRegistered,
    EnsureNotRegistered,
}
