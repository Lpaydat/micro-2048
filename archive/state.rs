use async_graphql::SimpleObject;
use linera_sdk::{
    linera_base_types::Timestamp,
    views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext},
};
use serde::{Deserialize, Serialize};

use game2048::{GameVariant, Direction};

/// The application state for the 2048 game
#[derive(RootView, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct Game2048State {
    /// Total number of registered participants
    pub participants_count: RegisterView<u64>,
    /// Total number of game sessions created
    pub sessions_count: RegisterView<u64>,
    /// Map of participant usernames to their IDs
    pub participants: MapView<String, ParticipantInfo>,
    /// Map of session IDs to game sessions
    pub game_sessions: MapView<u64, GameSession>,
}

#[derive(Clone, Serialize, Deserialize, SimpleObject)]
pub struct ParticipantInfo {
    pub participant_id: u64,
    pub username: String,
    pub chain_id: String,
    pub registration_time: Timestamp,
    pub total_sessions: u32,
    pub best_score: u64,
}

#[derive(Clone, Serialize, Deserialize, SimpleObject)]
pub struct GameSession {
    pub session_id: u64,
    pub participant_id: u64,
    pub game_variant: GameVariant,
    pub board_state: u64, // Bit-packed 4x4 grid
    pub score: u64,
    pub move_count: u32,
    pub highest_tile: u32,
    pub is_ended: bool,
    pub created_at: Timestamp,
    pub last_move_at: Option<Timestamp>,
}