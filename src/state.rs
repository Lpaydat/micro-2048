use async_graphql::{scalar, SimpleObject};
use linera_sdk::views::{
    linera_views, CollectionView, MapView, QueueView, RegisterView, RootView, View,
    ViewStorageContext,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
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
pub struct MoveRecord {
    pub direction: RegisterView<u8>,
    pub timestamp: RegisterView<u64>,
    pub board_after: RegisterView<u64>,
    pub score_after: RegisterView<u64>,
}

#[derive(View, SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct Player {
    pub username: RegisterView<String>,
    pub password_hash: RegisterView<String>,
    pub chain_id: RegisterView<String>,
    pub is_mod: RegisterView<bool>,
    pub boards_per_tournament: MapView<String, u32>, // tournament_id -> board_count
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
    pub move_history: CollectionView<u32, MoveRecord>, // move_index -> MoveRecord
    pub move_count: RegisterView<u32>,                 // Total number of moves made
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ActiveBoardInfo {
    pub player: String,
    pub score: u64,
    pub is_ended: bool,
}

#[derive(View, SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct LeaderboardShard {
    pub shard_id: RegisterView<String>,
    pub leaderboard_id: RegisterView<String>,
    pub chain_id: RegisterView<String>,
    pub start_time: RegisterView<u64>,
    pub end_time: RegisterView<u64>,

    pub score: MapView<String, u64>,               // username, score
    pub board_ids: MapView<String, String>,        // username, board_id
    pub is_ended: MapView<String, bool>,           // username, is_ended (for best board)
    pub player_chain_ids: MapView<String, String>, // username -> chain_id mapping
    pub highest_tiles: MapView<String, u64>,       // username -> highest_tile
    #[graphql(skip)]
    pub game_statuses: MapView<String, game2048::GameStatus>, // username -> game_status
    pub counter: RegisterView<u16>,                // update count

    #[graphql(skip)]
    pub active_boards: MapView<String, ActiveBoardInfo>, // board_id -> board summary

    // 🚀 NEW: Board counting per tournament (flattened key: "tournament_id:player_chain_id")
    pub tournament_player_board_counts: MapView<String, u32>, // "tournament_id:player_chain_id" -> board_count

    // 🚀 NEW: Player chain tracking and workload stats
    pub monitored_player_chains: QueueView<String>, // Player chain IDs we're monitoring
    pub active_players_count: RegisterView<u32>,    // Current active players
    pub total_games_count: RegisterView<u32>,       // Total games handled
    pub last_activity: RegisterView<u64>,           // Last activity timestamp

    // 🚀 PERFORMANCE: Smart player activity tracking
    pub player_activity_levels: MapView<String, u8>, // chain_id -> activity_level (0=very_active, 1=active, 2=inactive, 3=very_inactive)
    pub player_last_seen: MapView<String, u64>,      // chain_id -> last_event_timestamp
    pub player_read_intervals: MapView<String, u8>, // chain_id -> read_interval_multiplier (1, 5, 15)

    // 🚀 NEW: Activity-based triggerer tracking (rolling window)
    pub current_round_updates: MapView<String, u32>, // player_chain_id -> update_count_this_round

    // 🚀 NEW: Tournament configuration for dynamic triggerer calculation
    pub base_triggerer_count: RegisterView<u32>, // Tournament's configured triggerer count
    pub total_shard_count: RegisterView<u32>,    // Total shards in this tournament
    pub round_history: QueueView<String>,        // JSON of past round data (last N rounds)
    pub round_counter: RegisterView<u32>,        // Current aggregation round number
    pub round_start_time: RegisterView<u64>,     // When current round started
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
    pub is_ended: MapView<String, bool>,    // username, is_ended (for best board)
    #[graphql(skip)]
    pub active_boards: MapView<String, ActiveBoardInfo>, // board_id -> board summary

    // 🚀 NEW: Distributed board counting (player_chain_id -> total_boards_in_tournament)
    pub player_board_counts: MapView<String, u32>, // Merged from all shards

    pub shard_ids: QueueView<String>,           // shard_id
    pub current_shard_id: RegisterView<String>, // current shard_id

    // 🚀 NEW: Smart Triggerer Delegation System
    pub primary_triggerer: RegisterView<String>, // Primary triggerer chain_id
    pub backup_triggerers: QueueView<String>,    // Backup triggerers (up to 4)
    pub last_trigger_time: RegisterView<u64>,    // Last aggregation trigger timestamp
    pub last_trigger_by: RegisterView<String>,   // Who triggered last
    pub trigger_rotation_counter: RegisterView<u32>, // Rotation counter for fairness
    pub trigger_cooldown_until: RegisterView<u64>, // Global cooldown: no triggers until this time

    // 🚀 NEW: Activity-based triggerer ranking
    pub player_activity_scores: MapView<String, u32>, // player_chain_id -> weighted_activity_score
    pub last_successful_update: RegisterView<u64>, // Last time leaderboard was successfully updated

    pub admin_base_triggerer_count: RegisterView<u32>, // Admin-configurable base triggerer count
}

#[derive(View, SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct PlayerRecord {
    pub best_score: MapView<String, u64>, // tournament_id (leaderboard_id) -> best_score
}

#[derive(RootView, SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct Chain {
    pub chain_id: RegisterView<String>,
}

#[derive(RootView)]
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

    // 🚀 NEW: Tournament cache for player chains (from streaming system)
    // Note: Using String storage for TournamentInfo to avoid GraphQL OutputType issues
    pub tournaments_cache_json: MapView<String, String>, // tournament_id -> JSON-serialized tournament info
    pub last_tournament_update: RegisterView<u64>,       // Last tournament update timestamp

    // 🚀 NEW: Triggerer system for player chains
    pub triggerer_list: QueueView<String>, // Current triggerer list (sorted by activity)
    pub triggerer_activity_scores: QueueView<u32>, // Activity scores corresponding to triggerer_list
    pub triggerer_list_timestamp: RegisterView<u64>, // Last triggerer list update
    pub trigger_threshold_config: RegisterView<u64>, // Minimum time between triggers (microseconds)
    pub last_trigger_sent: RegisterView<u64>,      // Last time this player sent a trigger
    pub total_registered_players: RegisterView<u32>, // Total number of registered players
    pub admin_base_triggerer_count: RegisterView<u32>, // Admin-configurable base triggerer count

    // 🚀 TIER 6 EMERGENCY MODE: Operation counting for inactive leaderboard recovery
    pub operations_since_tier6: RegisterView<u32>, // Count of operations since tier 6 started
    pub tier6_start_time: RegisterView<u64>,       // When tier 6 began (last_update + 5*threshold)
    pub is_in_tier6: RegisterView<bool>,           // Flag to track if we're in tier 6 mode

    // 🚀 CHAIN POOL: Pre-created chains for fast registration
    pub unclaimed_chains: QueueView<String>,         // Pre-created chain IDs available for claiming
    pub chain_pool_target_size: RegisterView<u32>,   // Target pool size (e.g., 100)
    pub chain_pool_low_threshold: RegisterView<u32>, // Trigger replenish when below this (e.g., 20)
}
