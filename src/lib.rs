mod direction;
mod elimination_game;
mod event_leaderboard;
mod game;
mod moves;
mod random;

pub use crate::direction::Direction;
pub use crate::event_leaderboard::{LeaderboardAction, LeaderboardSettings};
pub use crate::game::Game;
pub use crate::moves::{Moves, COL_MASK, ROW_MASK};
pub use crate::random::{hash_seed, rnd_range};


use linera_sdk::linera_base_types::{Amount, ChainId};
use linera_sdk::{
    abi::{ContractAbi, ServiceAbi},
    graphql::GraphQLMutationRoot,
};
use serde::{Deserialize, Serialize};

pub struct Game2048Abi;

impl ContractAbi for Game2048Abi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for Game2048Abi {
    type Query = async_graphql::Request;
    type QueryResponse = async_graphql::Response;
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
        player: String,
        timestamp: u64,
        password_hash: String,
        leaderboard_id: String, // Must specify leaderboard
    },
    NewShard,
    MakeMoves {
        board_id: String,
        moves: String, // JSON array of MoveEntry
        player: String,
        password_hash: String,
    },
     LeaderboardAction {
         leaderboard_id: String,
         action: LeaderboardAction,
         settings: LeaderboardSettings,
         player: String,
         password_hash: String,
     },
    ToggleAdmin {
        username: String,
        player: String,
        password_hash: String,
    },
    CloseChain {
        chain_id: String,
    },
    Faucet,
    /// 🚀 IMPROVED: Triggers shard chain to aggregate scores from monitored player chains
    AggregateScores,
    /// 🚀 IMPROVED: Triggers leaderboard chain to update from registered shard chains  
    UpdateLeaderboard,
    /// 🚀 NEW: Emit current active tournaments (leaderboard chains)
    UpdateActiveTournaments,
    /// 🚀 NEW: Emit current workload (shard chains)
    UpdateShardWorkload,
    /// 🚀 NEW: Centralized aggregation trigger (for designated triggerers)
    RequestAggregation {
        requester_chain_id: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    RegisterPlayer {
        username: String,
        password_hash: String,
    },
    Transfer {
        chain_id: ChainId,
        amount: Amount,
    },
    // RequestNewBoard {
    //     seed: String,
    // },
    CreateNewBoard {
        seed: String,
        player: String,
        timestamp: u64,
        leaderboard_id: String,
        shard_id: String,
        end_time: u64,
    },
    CreateLeaderboard {
        leaderboard_id: String,
        name: String,
        description: Option<String>,
        chain_id: String,
        host: String,
        start_time: u64,
        end_time: u64,
        shard_ids: Vec<String>,
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
        is_end: bool,
        timestamp: u64,
    },
    Flush {
        board_ids: std::collections::HashMap<String, String>,
        scores: std::collections::HashMap<String, u64>,
    },
    /// 🚀 NEW: Player registers with shard for tournament monitoring
    RegisterPlayerWithShard {
        player_chain_id: String,
        tournament_id: String,
        player_name: String,
    },
    /// 🚀 NEW: Request leaderboard to trigger aggregation (delegated triggerer pattern)
    RequestAggregationTrigger {
        requester_chain_id: String,
        timestamp: u64,
    },
    /// 🚀 NEW: Leaderboard tells shard to aggregate
    TriggerShardAggregation {
        timestamp: u64,
    },
}

/// 🚀 ENHANCED: Four event types for four channels
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum GameEvent {
    /// Channel: "player_score_update" - Emitted on every score change, game creation, game end
    PlayerScoreUpdate {
        player: String,
        board_id: String,
        score: u64,
        chain_id: String,
        timestamp: u64,
        game_status: GameStatus,
        highest_tile: u64,
        moves_count: u32,
        leaderboard_id: String,
        /// Current best score for this player in this leaderboard (for shard filtering)
        current_leaderboard_best: u64,
    },
    
    /// Channel: "shard_score_update" - Emitted by shard chains with aggregated scores
    ShardScoreUpdate {
        shard_chain_id: String,
        player_scores: std::collections::HashMap<String, PlayerScoreSummary>,
        aggregation_timestamp: u64,
        total_players: u32,
        leaderboard_id: String,
    },

    /// Channel: "active_tournaments" - Emitted by leaderboard with current active tournaments
    ActiveTournaments {
        tournaments: Vec<TournamentInfo>,
        timestamp: u64,
    },

    /// Channel: "shard_workload" - Emitted by shards for load balancing
    ShardWorkload {
        shard_chain_id: String,
        tournament_id: String,
        total_players: u32,
        active_players_last_5min: u32,
        timestamp: u64,
    },
}

/// Tournament information for the registry
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TournamentInfo {
    pub tournament_id: String,
    pub name: String,
    pub shard_chain_ids: Vec<String>,
    pub start_time: u64,
    pub end_time: u64,
    pub status: TournamentStatus,
    pub total_players: u32,
}

/// Tournament status
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum TournamentStatus {
    Active,
    Ended,
}

/// Game status for tracking lifecycle
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum GameStatus {
    Created,    // Game just created
    Active,     // Game is being played
    Ended(GameEndReason), // Game finished with reason
}

/// Player score summary for shard aggregation
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlayerScoreSummary {
    pub player: String,
    pub best_score: u64,
    pub board_id: String,
    pub chain_id: String,
    pub highest_tile: u64,
    pub last_update: u64,
    pub game_status: GameStatus,
}



#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum GameEndReason {
    NoMoves, // Board is full, no valid moves available
    TournamentEnded, // Tournament/leaderboard time expired
}

pub enum RegistrationCheck {
    EnsureRegistered,
    EnsureNotRegistered,
}
