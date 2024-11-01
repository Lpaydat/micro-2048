use async_graphql::{scalar, InputObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum MultiplayerGameAction {
    Start,
    End,
    Trigger,
    Join,
    Leave,
    NextRound,
}

scalar!(MultiplayerGameAction);

// Define a struct for elimination game settings
#[derive(Debug, Deserialize, Serialize, InputObject)]
pub struct EliminationGameSettings {
    pub chain_id: String,
    pub game_name: String,
    pub host: String,
    pub total_round: u8,
    pub max_players: u8,
    pub eliminated_per_trigger: u8,
    pub trigger_interval_seconds: u16,
    pub created_time: String,
}
