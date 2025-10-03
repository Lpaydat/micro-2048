//! Event Readers
//!
//! Utilities for reading events from remote chains in the streaming system.

use game2048::GameEvent;
use linera_sdk::linera_base_types::ChainId;

/// Event reading utilities
pub struct EventReader;

impl EventReader {
    /// Reads a player score event from another chain
    pub fn read_player_score_event_from_chain(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
        event_index: u32,
    ) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("player_score_update".to_string());

        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract
                .runtime
                .read_event(chain_id, stream_name, event_index)
        }))
        .ok()
    }

    /// Reads a shard score event from another chain
    pub fn read_shard_score_event_from_chain(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
        event_index: u32,
    ) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("shard_score_update".to_string());

        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract
                .runtime
                .read_event(chain_id, stream_name, event_index)
        }))
        .ok()
    }

    /// Read active tournaments event from chain
    pub fn read_active_tournaments_event_from_chain(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
        event_index: u32,
    ) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("active_tournaments".to_string());

        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract
                .runtime
                .read_event(chain_id, stream_name, event_index)
        }))
        .ok()
    }

    /// Read leaderboard update event from chain
    pub fn read_leaderboard_update_event_from_chain(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
        event_index: u32,
    ) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let stream_name = StreamName::from("leaderboard_update".to_string());

        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract
                .runtime
                .read_event(chain_id, stream_name, event_index)
        }))
        .ok()
    }
}
