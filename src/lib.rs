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

    /// 🚀 NEW: Centralized aggregation trigger (for designated triggerers)
    RequestAggregation {
        requester_chain_id: String,
    },
    /// 🚀 ADMIN: Configure base triggerer count
    ConfigureTriggererCount {
        admin_username: String,
        password_hash: String,
        base_triggerer_count: u32,
    },
    /// 🚀 NEW: Manual leaderboard refresh - player can trigger update when their score is higher
    RequestLeaderboardRefresh {
        player: String,
        password_hash: String,
        leaderboard_id: String,
    },

    // 🚀 CHAIN POOL: Pre-create chains for fast registration
    /// ADMIN: Refill the chain pool with pre-created player chains
    RefillChainPool {
        count: u32,
    },
    
    /// 🚀 MESSAGE-BASED: Claim/initialize player chain after registration
    /// This processes the inbox to receive RegisterPlayer message
    /// No authentication needed - just triggers block production
    ClaimChain,
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
        base_triggerer_count: u32,
        total_shard_count: u32,
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
    /// 🚀 NEW: Player chain subscribes to main chain's active tournaments
    SubscribeToMainChain {
        main_chain_id: String,
    },
    /// 🚀 NEW: Shard registers first player with leaderboard for triggerer system
    RegisterFirstPlayer {
        shard_chain_id: String,
        player_chain_id: String,
        tournament_id: String,
    },
    /// 🚀 IMPROVED: Shard sends multiple trigger candidates to leaderboard
    UpdateShardTriggerCandidates {
        shard_chain_id: String,
        player_chain_ids: Vec<String>,
        tournament_id: String,
    },
    /// 🚀 NEW: Player chain sends trigger update request to leaderboard
    TriggerUpdate {
        triggerer_chain_id: String,
        tournament_id: String,
        timestamp: u64,
    },
    
    /// 🚀 NEW (Message-based architecture): Player submits score directly to leaderboard
    /// Sent when: score > 0 AND score > current_best (new personal best)
    /// Replaces: PlayerScoreUpdate event + shard aggregation
    SubmitScore {
        player: String,
        player_chain_id: String,
        board_id: String,
        score: u64,
        highest_tile: u64,
        game_status: GameStatus,
        timestamp: u64,
        boards_in_tournament: u32,
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
        /// 🚀 NEW: Player's total board count in this tournament (for distributed counting)
        boards_in_tournament: u32,
    },

    /// Channel: "shard_score_update" - Emitted by shard chains with aggregated scores
    ShardScoreUpdate {
        shard_chain_id: String,
        player_scores: std::collections::HashMap<String, PlayerScoreSummary>,
        player_activity_scores: std::collections::HashMap<String, u32>, // NEW: player -> activity_score
        /// 🚀 NEW: Player board counts for distributed board counting (player_chain_id -> board_count)
        player_board_counts: std::collections::HashMap<String, u32>,
        aggregation_timestamp: u64,
        total_players: u32,
        leaderboard_id: String,
    },

    /// Channel: "active_tournaments" - Emitted by leaderboard with current active tournaments
    ActiveTournaments {
        tournaments: Vec<TournamentInfo>,
        timestamp: u64,
    },

    /// Channel: "leaderboard_update" - Emitted by leaderboard with triggerer list updates
    LeaderboardUpdate {
        leaderboard_id: String,
        triggerer_list: Vec<(String, u32)>, // (player_chain_id, activity_score) sorted by activity (most active first)
        last_update_timestamp: u64,
        threshold_config: u64, // Minimum time between triggers (microseconds)
        total_registered_players: u32,
    },
}

/// Tournament information for the registry
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TournamentInfo {
    pub tournament_id: String,
    pub name: String,
    pub shard_chain_ids: Vec<String>,
    pub start_time: Option<u64>, // None = unlimited start time
    pub end_time: Option<u64>,   // None = unlimited end time
    pub total_players: u32,
}

impl TournamentInfo {
    /// Check if tournament is currently active based on time
    pub fn is_active(&self, current_time: u64) -> bool {
        let started = self.start_time.is_none_or(|start| current_time >= start);
        let not_ended = self.end_time.is_none_or(|end| current_time < end);
        started && not_ended
    }

    /// Check if tournament is in the future
    pub fn is_future(&self, current_time: u64) -> bool {
        self.start_time.is_some_and(|start| current_time < start)
    }

    /// Check if tournament has ended
    pub fn is_ended(&self, current_time: u64) -> bool {
        self.end_time.is_some_and(|end| current_time >= end)
    }
}

/// Game status for tracking lifecycle
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum GameStatus {
    Created,              // Game just created
    Active,               // Game is being played
    Ended(GameEndReason), // Game finished with reason
}

/// Summary of an active board for leaderboard display
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ActiveBoardSummary {
    pub board_id: String,
    pub player: String,
    pub score: u64,
    pub is_ended: bool,
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
    /// 🚀 NEW: Player's total board count in this tournament
    pub boards_in_tournament: u32,
    /// 🚀 ACTIVE BOARDS: Current active boards tracked for this player
    #[serde(default)]
    pub active_boards: Vec<ActiveBoardSummary>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum GameEndReason {
    NoMoves,         // Board is full, no valid moves available
    TournamentEnded, // Tournament/leaderboard time expired
}

pub enum RegistrationCheck {
    EnsureRegistered,
    EnsureNotRegistered,
}

/// Instantiation argument for the contract
/// Simple u32 for backward compatibility:
/// - 0 = use default (300 chains)
/// - N = create N chains in pool
pub type InstantiationArgument = u32;
