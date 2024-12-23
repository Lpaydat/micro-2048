use async_graphql::{scalar, SimpleObject};
use linera_sdk::views::{
    linera_views, CollectionView, MapView, RegisterView, RootView, View, ViewStorageContext,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum GameStatus {
    #[default]
    Active,
    Ended,
}
scalar!(GameStatus);

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum EliminationGameStatus {
    #[default]
    Waiting,
    Active,
    Ended,
}
scalar!(EliminationGameStatus);

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum GameMode {
    #[default]
    Classic,
    Elimination,
}
scalar!(GameMode);

#[derive(View, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct Player {
    pub username: RegisterView<String>,
    pub password_hash: RegisterView<String>,
    pub chain_id: RegisterView<String>,
    pub is_mod: RegisterView<bool>,
}

#[derive(View, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct BoardState {
    pub board_id: RegisterView<String>,
    pub board: RegisterView<u64>,
    pub score: RegisterView<u64>,
    pub is_ended: RegisterView<bool>,
    pub player: RegisterView<String>,
    pub chain_id: RegisterView<String>,
    pub leaderboard_id: RegisterView<String>,
}

#[derive(View, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct MultiplayerLeaderboard {
    pub players: MapView<String, u64>,            // username, score
    pub eliminated_players: MapView<String, u64>, // username
}

#[derive(View, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct ClassicLeaderboard {
    pub leaderboard_id: RegisterView<String>,
    pub chain_id: RegisterView<String>,
    pub name: RegisterView<String>,
    pub description: RegisterView<String>,
    pub host: RegisterView<String>,
    pub start_time: RegisterView<u64>,
    pub end_time: RegisterView<u64>,
    pub total_boards: RegisterView<u32>,
    pub total_players: RegisterView<u32>,
    pub is_pinned: RegisterView<bool>,

    pub score: MapView<String, u64>,        // username, score
    pub board_ids: MapView<String, String>, // username, board_id
}

#[derive(View, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct PlayerRecord {
    pub best_score: MapView<String, u64>, // leaderboard_id, score
}

#[derive(View, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct EliminationGame {
    pub game_id: RegisterView<String>,
    pub chain_id: RegisterView<String>,
    pub game_name: RegisterView<String>,
    pub host: RegisterView<String>,
    pub players: RegisterView<Vec<String>>, // board_id = game_id-player_chain_id-username-round
    pub status: RegisterView<EliminationGameStatus>,
    pub total_rounds: RegisterView<u8>,
    pub current_round: RegisterView<u8>,
    pub max_players: RegisterView<u8>,
    pub eliminated_per_trigger: RegisterView<u8>,
    pub trigger_interval_seconds: RegisterView<u16>,
    pub round_leaderboard: CollectionView<u8, MultiplayerLeaderboard>,
    pub game_leaderboard: MapView<String, u64>,
    pub created_time: RegisterView<u64>,
    pub last_updated_time: RegisterView<u64>,
}

#[derive(RootView, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct Chain {
    pub chain_id: RegisterView<String>,
}

#[derive(RootView, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct Game2048 {
    pub boards: CollectionView<String, BoardState>,
    pub elimination_games: CollectionView<String, EliminationGame>, // game_id
    pub waiting_rooms: MapView<String, bool>,
    pub players: CollectionView<String, Player>,
    pub leaderboards: CollectionView<String, ClassicLeaderboard>, // leaderboard_id
    pub player_records: CollectionView<String, PlayerRecord>,     // player_chain_id
}
