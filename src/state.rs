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
#[view(context = ViewStorageContext)]
pub struct Player {
    pub username: RegisterView<String>,
    pub password_hash: RegisterView<String>,
    pub chain_id: RegisterView<String>,
    pub is_mod: RegisterView<bool>,
}

#[derive(View, SimpleObject)]
#[view(context = ViewStorageContext)]
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
    pub created_at: RegisterView<u64>,
}

#[derive(View, SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct LeaderboardShard {
    pub shard_id: RegisterView<String>,
    pub leaderboard_id: RegisterView<String>,
    pub chain_id: RegisterView<String>,
    pub start_time: RegisterView<u64>,
    pub end_time: RegisterView<u64>,

    pub score: MapView<String, u64>,        // username, score
    pub board_ids: MapView<String, String>, // username, board_id
    pub counter: RegisterView<u16>,         // update count
    
    // 🚀 NEW: Player chain tracking and workload stats
    pub monitored_player_chains: QueueView<String>, // Player chain IDs we're monitoring
    pub active_players_count: RegisterView<u32>,     // Current active players
    pub total_games_count: RegisterView<u32>,        // Total games handled
    pub last_activity: RegisterView<u64>,            // Last activity timestamp
    
    // 🚀 PERFORMANCE: Smart player activity tracking
    pub player_activity_levels: MapView<String, u8>,    // chain_id -> activity_level (0=very_active, 1=active, 2=inactive, 3=very_inactive)
    pub player_last_seen: MapView<String, u64>,         // chain_id -> last_event_timestamp
    pub player_read_intervals: MapView<String, u8>,     // chain_id -> read_interval_multiplier (1, 5, 15)
}

#[derive(View, SimpleObject)]
#[view(context = ViewStorageContext)]
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
    
    // 🚀 NEW: Smart Triggerer Delegation System
    pub primary_triggerer: RegisterView<String>,         // Primary triggerer chain_id
    pub backup_triggerers: QueueView<String>,           // Backup triggerers (up to 4)
    pub last_trigger_time: RegisterView<u64>,           // Last aggregation trigger timestamp
    pub last_trigger_by: RegisterView<String>,          // Who triggered last
    pub trigger_rotation_counter: RegisterView<u32>,    // Rotation counter for fairness
    pub trigger_cooldown_until: RegisterView<u64>,      // Cooldown period (no triggers until this time)
}

#[derive(View, SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct PlayerRecord {
    pub best_score: MapView<String, u64>, // leaderboard_id, score
}

#[derive(RootView, SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct Chain {
    pub chain_id: RegisterView<String>,
}

#[derive(RootView, SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct Game2048 {
    pub balance: RegisterView<String>,
    pub boards: CollectionView<String, BoardState>,
    pub players: CollectionView<String, Player>,
    pub leaderboards: CollectionView<String, Leaderboard>, // leaderboard_id
    pub shards: CollectionView<String, LeaderboardShard>, // should contain only one shard with empty shard_id
    pub player_records: CollectionView<String, PlayerRecord>, // player_chain_id
    pub onboard_chains: QueueView<String>,                // chain_id
    pub nonce: RegisterView<u64>,
    pub latest_board_id: RegisterView<String>,
    
    // 🚀 NEW: Event index tracking for reliable event reading
    pub player_score_event_indices: MapView<String, u64>, // chain_id -> last processed event index
    pub shard_score_event_indices: MapView<String, u64>,  // chain_id -> last processed event index  
    pub active_tournaments_event_index: RegisterView<u64>, // Last processed tournament registry index
    pub shard_workload_event_indices: MapView<String, u64>, // chain_id -> last processed workload index
}