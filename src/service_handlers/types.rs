use async_graphql::SimpleObject;
use std::collections::HashMap;

#[derive(SimpleObject)]
pub struct BoardState {
    pub board_id: String,
    pub board: [[u16; 4]; 4],
    pub is_ended: bool,
    pub score: u64,
    pub player: String,
    pub chain_id: String,
    pub leaderboard_id: String,
    pub shard_id: String,
    pub created_at: String,
    pub end_time: String,
}

#[derive(SimpleObject)]
pub struct LeaderboardEntry {
    pub username: String,
    pub score: u64,
}

#[derive(SimpleObject)]
pub struct LeaderboardState {
    pub leaderboard_id: String,
    pub chain_id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_pinned: bool,
    pub host: String,
    pub start_time: String,
    pub end_time: String,
    pub total_boards: u32,
    pub total_players: u32,
    pub rankers: Vec<Ranker>,
    pub shard_ids: Vec<String>,
}

#[derive(SimpleObject)]
pub struct Player {
    pub username: String,
    pub chain_id: String,
    pub is_mod: bool,
}

#[derive(SimpleObject, serde::Serialize)]
pub struct Ranker {
    pub username: String,
    pub score: u64,
    pub board_id: String,
}

#[derive(SimpleObject)]
pub struct Shard {
    pub shard_id: String,
    pub leaderboard_id: String,
    pub chain_id: String,
    pub start_time: String,
    pub scores: HashMap<String, u64>,
    pub board_ids: HashMap<String, String>,
    pub end_time: String,
    pub counter: u16,
}

/// 🚀 NEW: Triggerer pool information for clients
#[derive(SimpleObject)]
pub struct TriggererPool {
    pub primary: Option<String>,
    pub backups: Vec<String>,
    pub last_trigger_time: u64,
    pub cooldown_until: u64,
}
