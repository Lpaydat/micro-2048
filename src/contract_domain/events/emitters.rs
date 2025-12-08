//! Event Emitters
//!
//! Utilities for creating and emitting events to streams.
//! 
//! 🚀 MESSAGE-BASED ARCHITECTURE: Score-related events are deprecated.
//! Only ActiveTournaments event is still used for tournament discovery.

use game2048::GameEvent;

/// Event emission utilities
pub struct EventEmitter;

impl EventEmitter {
    /// Emit active tournaments event
    /// This is the only event still actively used in the message-based architecture.
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

    // ═══════════════════════════════════════════════════════════════
    // DEPRECATED EMITTERS (kept for reference, may be removed later)
    // ═══════════════════════════════════════════════════════════════

    /// DEPRECATED: Use Message::SubmitScore instead
    #[allow(dead_code)]
    pub async fn emit_player_score_update(
        _contract: &mut crate::Game2048Contract,
        _player: String,
        _board_id: String,
        _score: u64,
        _chain_id: String,
        _timestamp: u64,
        _game_status: game2048::GameStatus,
        _highest_tile: u64,
        _moves_count: u32,
        _leaderboard_id: String,
        _current_leaderboard_best: u64,
        _boards_in_tournament: u32,
    ) {
        // No-op: Use Message::SubmitScore instead
    }

    /// DEPRECATED: No longer using shard aggregation
    #[allow(dead_code)]
    pub async fn emit_shard_score_update(
        _contract: &mut crate::Game2048Contract,
        _shard_chain_id: String,
        _player_scores: std::collections::HashMap<String, game2048::PlayerScoreSummary>,
        _player_activity_scores: std::collections::HashMap<String, u32>,
        _player_board_counts: std::collections::HashMap<String, u32>,
        _aggregation_timestamp: u64,
        _total_players: u32,
        _leaderboard_id: String,
    ) {
        // No-op: No longer using shard aggregation
    }

    /// DEPRECATED: No longer using triggerer system
    #[allow(dead_code)]
    pub async fn emit_leaderboard_update(
        _contract: &mut crate::Game2048Contract,
        _leaderboard_id: String,
        _triggerer_list: Vec<(String, u32)>,
        _last_update_timestamp: u64,
        _threshold_config: u64,
        _total_registered_players: u32,
    ) {
        // No-op: No longer using triggerer system
    }
}
