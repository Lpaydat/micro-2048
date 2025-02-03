use async_graphql::{scalar, SimpleObject};
use linera_sdk::views::{
    linera_views, CollectionView, MapView, QueueView, RegisterView, RootView, View,
    ViewStorageContext,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum GameStatus {
    #[default]
    Active,
    Ended,
}
scalar!(GameStatus);

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
    pub leaderboard_id: RegisterView<String>, // client can use to fetch leaderboard
    pub shard_id: RegisterView<String>,
    pub end_time: RegisterView<u64>,
}

#[derive(View, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct LeaderboardShard {
    pub shard_id: RegisterView<String>,
    pub leaderboard_id: RegisterView<String>,
    pub chain_id: RegisterView<String>,
    pub start_time: RegisterView<u64>,
    pub end_time: RegisterView<u64>,

    pub score: MapView<String, u64>,        // username, score
    pub board_ids: MapView<String, String>, // username, board_id
    pub counter: RegisterView<u16>,         // update count
}

#[derive(View, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct Leaderboard {
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

    pub shard_ids: QueueView<String>,           // shard_id
    pub current_shard_id: RegisterView<String>, // current shard_id
}

#[derive(View, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct PlayerRecord {
    pub best_score: MapView<String, u64>, // leaderboard_id, score
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
    pub players: CollectionView<String, Player>,
    pub leaderboards: CollectionView<String, Leaderboard>, // leaderboard_id
    pub shards: CollectionView<String, LeaderboardShard>, // should contain only one shard with empty shard_id
    pub player_records: CollectionView<String, PlayerRecord>, // player_chain_id
    pub onboard_chains: QueueView<String>,                // chain_id
    pub nonce: RegisterView<u64>,
}
