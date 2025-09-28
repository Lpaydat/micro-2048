//! Stream Processing Handler
//! 
//! Handles event stream subscriptions and reading operations.

use linera_sdk::linera_base_types::ChainId;
use game2048::GameEvent;

pub struct StreamProcessingHandler;

impl StreamProcessingHandler {
    /// Reads a player score event from another chain
    pub fn read_player_score_event_from_chain(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
        event_index: u32,
    ) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("player_score_update".to_string());
        
        log::info!("📖 STREAM_READ: Reading player_score_update event from chain {} at index {}", chain_id, event_index);
        
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract.runtime.read_event(chain_id, stream_name, event_index)
        })).ok()
    }

    /// Reads a shard score event from another chain
    pub fn read_shard_score_event_from_chain(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
        event_index: u32,
    ) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("shard_score_update".to_string());
        
        log::info!("📖 STREAM_READ: Reading shard_score_update event from chain {} at index {}", chain_id, event_index);
        
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract.runtime.read_event(chain_id, stream_name, event_index)
        })).ok()
    }

    /// Subscribes to player score events from another chain
    pub fn subscribe_to_player_score_events(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
    ) {
        use linera_sdk::linera_base_types::{StreamName, ApplicationId};
        let stream_name = StreamName::from("player_score_update".to_string());
        let application_id = ApplicationId::new(contract.runtime.application_id().application_description_hash);
        
        log::info!("🔔 SUBSCRIBE: Subscribing to player_score_update events from chain {} (app: {})", chain_id, application_id);
        contract.runtime.subscribe_to_events(chain_id, application_id, stream_name);
        log::info!("🔔 SUBSCRIBE: ✅ Successfully subscribed to player_score_update from chain {}", chain_id);
    }

    /// Subscribes to shard score events from another chain
    pub fn subscribe_to_shard_score_events(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
    ) {
        use linera_sdk::linera_base_types::{StreamName, ApplicationId};
        let stream_name = StreamName::from("shard_score_update".to_string());
        let application_id = ApplicationId::new(contract.runtime.application_id().application_description_hash);
        
        log::info!("🔔 SUBSCRIBE: Subscribing to shard_score_update events from chain {} (app: {})", chain_id, application_id);
        contract.runtime.subscribe_to_events(chain_id, application_id, stream_name);
        log::info!("🔔 SUBSCRIBE: ✅ Successfully subscribed to shard_score_update from chain {}", chain_id);
    }

    /// Subscribes to leaderboard update events from leaderboard chain
    pub fn subscribe_to_leaderboard_update_events(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
    ) {
        use linera_sdk::linera_base_types::{StreamName, ApplicationId};
        let stream_name = StreamName::from("leaderboard_update".to_string());
        let application_id = ApplicationId::new(contract.runtime.application_id().application_description_hash);
        
        log::info!("🔔 SUBSCRIBE: Subscribing to leaderboard_update events from chain {} (app: {})", chain_id, application_id);
        contract.runtime.subscribe_to_events(chain_id, application_id, stream_name);
        log::info!("🔔 SUBSCRIBE: ✅ Successfully subscribed to leaderboard_update from chain {}", chain_id);
    }

    /// Read active tournaments event from main chain
    pub fn read_active_tournaments_event_from_chain(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
        event_index: u32,
    ) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("active_tournaments".to_string());
        
        log::info!("📖 STREAM_READ: Reading active_tournaments event from chain {} at index {}", chain_id, event_index);
        
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract.runtime.read_event(chain_id, stream_name, event_index)
        })).ok()
    }

    /// Read leaderboard update event from leaderboard chain
    pub fn read_leaderboard_update_event_from_chain(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
        event_index: u32,
    ) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("leaderboard_update".to_string());
        
        log::info!("📖 STREAM_READ: Reading leaderboard_update event from chain {} at index {}", chain_id, event_index);
        
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract.runtime.read_event(chain_id, stream_name, event_index)
        })).ok()
    }

    /// Emit game creation event helper
    pub async fn emit_game_creation_event(
        contract: &mut crate::Game2048Contract,
        board_id: &str,
        player: &str,
        tournament_id: &str,
        timestamp: u64,
    ) {
        use linera_sdk::linera_base_types::StreamName;
        
        // Get current best score for this player in this tournament
        let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
        let current_best = leaderboard.score.get(player).await.unwrap().unwrap_or(0);
        
        // Get player's current board count for this tournament
        let player_state = contract.state.players.load_entry_mut(player).await.unwrap();
        let current_board_count = player_state.boards_per_tournament
            .get(tournament_id).await.unwrap().unwrap_or(0);
        
        let score_event = GameEvent::PlayerScoreUpdate {
            player: player.to_string(),
            board_id: board_id.to_string(),
            score: 0, // Initial score is 0
            chain_id: contract.runtime.chain_id().to_string(),
            timestamp,
            game_status: game2048::GameStatus::Created,
            highest_tile: 2, // Initial highest tile
            moves_count: 0,
            leaderboard_id: tournament_id.to_string(),
            current_leaderboard_best: current_best,
            boards_in_tournament: current_board_count,
        };
        
        let stream_name = StreamName::from("player_score_update".to_string());
        contract.runtime.emit(stream_name, &score_event);
    }
}