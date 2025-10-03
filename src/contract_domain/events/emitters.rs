//! Event Emitters
//!
//! Utilities for creating and emitting events to streams.

use game2048::GameEvent;

/// Event emission utilities
pub struct EventEmitter;

impl EventEmitter {
    /// Emit a player score update event
    pub async fn emit_player_score_update(
        contract: &mut crate::Game2048Contract,
        player: String,
        board_id: String,
        score: u64,
        chain_id: String,
        timestamp: u64,
        game_status: game2048::GameStatus,
        highest_tile: u64,
        moves_count: u32,
        leaderboard_id: String,
        current_leaderboard_best: u64,
        boards_in_tournament: u32,
    ) {
        let event = GameEvent::PlayerScoreUpdate {
            player,
            board_id,
            score,
            chain_id,
            timestamp,
            game_status,
            highest_tile,
            moves_count,
            leaderboard_id,
            current_leaderboard_best,
            boards_in_tournament,
        };

        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("player_score_update".to_string());
        contract.runtime.emit(stream_name, &event);
    }

    /// Emit a shard score update event
    pub async fn emit_shard_score_update(
        contract: &mut crate::Game2048Contract,
        shard_chain_id: String,
        player_scores: std::collections::HashMap<String, game2048::PlayerScoreSummary>,
        player_activity_scores: std::collections::HashMap<String, u32>,
        player_board_counts: std::collections::HashMap<String, u32>,
        aggregation_timestamp: u64,
        total_players: u32,
        leaderboard_id: String,
    ) {
        let event = GameEvent::ShardScoreUpdate {
            shard_chain_id,
            player_scores,
            player_activity_scores,
            player_board_counts,
            aggregation_timestamp,
            total_players,
            leaderboard_id,
        };

        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("shard_score_update".to_string());
        contract.runtime.emit(stream_name, &event);
    }

    /// Emit active tournaments event
    pub async fn emit_active_tournaments(
        contract: &mut crate::Game2048Contract,
        tournaments: Vec<game2048::TournamentInfo>,
        timestamp: u64,
    ) {
        let event = GameEvent::ActiveTournaments {
            tournaments,
            timestamp,
        };

        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("active_tournaments".to_string());
        contract.runtime.emit(stream_name, &event);
    }

    /// Emit leaderboard update event
    pub async fn emit_leaderboard_update(
        contract: &mut crate::Game2048Contract,
        leaderboard_id: String,
        triggerer_list: Vec<(String, u32)>,
        last_update_timestamp: u64,
        threshold_config: u64,
        total_registered_players: u32,
    ) {
        let event = GameEvent::LeaderboardUpdate {
            leaderboard_id,
            triggerer_list,
            last_update_timestamp,
            threshold_config,
            total_registered_players,
        };

        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("leaderboard_update".to_string());
        contract.runtime.emit(stream_name, &event);
    }

    /// Emit game creation event helper
    pub async fn emit_game_creation_event(
        contract: &mut crate::Game2048Contract,
        board_id: &str,
        player: &str,
        tournament_id: &str,
        timestamp: u64,
        boards_in_tournament: u32,
    ) {
        let score_event = GameEvent::PlayerScoreUpdate {
            player: player.to_string(),
            board_id: board_id.to_string(),
            score: 0,
            chain_id: contract.runtime.chain_id().to_string(),
            timestamp,
            game_status: game2048::GameStatus::Active,
            highest_tile: 2,
            moves_count: 0,
            leaderboard_id: tournament_id.to_string(),
            current_leaderboard_best: 0,
            boards_in_tournament,
        };

        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("player_score_update".to_string());
        contract.runtime.emit(stream_name, &score_event);
    }
}
