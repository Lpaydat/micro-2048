//! Stream Processors
//!
//! Logic for processing incoming stream updates and events.
//! 
//! 🚀 MESSAGE-BASED ARCHITECTURE: Score updates now use SubmitScore message.
//! Only ActiveTournaments events are processed for tournament discovery.

use crate::contract_domain::events::EventReader;
use linera_sdk::linera_base_types::StreamUpdate;

/// Stream processing utilities
pub struct StreamProcessor;

impl StreamProcessor {
    /// Process all stream updates for the contract
    /// 
    /// 🚀 MESSAGE-BASED: Only processes active_tournaments events.
    /// Score updates (player_score_update, shard_score_update, leaderboard_update)
    /// are no longer used - replaced by direct SubmitScore messages.
    pub async fn process_updates(
        contract: &mut crate::Game2048Contract,
        updates: Vec<StreamUpdate>,
    ) {
        for update in updates.iter() {
            // Determine which stream we're processing based on stream name
            let stream_name_bytes = &update.stream_id.stream_name.0;
            let stream_name = String::from_utf8_lossy(stream_name_bytes);

            // Process all new events in this stream update
            let event_count = update.next_index - update.previous_index;
            if event_count == 0 {
                continue;
            }

            for event_index in update.previous_index..update.next_index {
                match stream_name.as_ref() {
                    // Only active_tournaments events are still used
                    "active_tournaments" => {
                        Self::process_active_tournaments(contract, update, event_index).await;
                    }
                    // All score-related events are deprecated (use SubmitScore message instead)
                    "player_score_update" | "shard_score_update" | "leaderboard_update" => {
                        // DEPRECATED: No-op for backward compatibility
                    }
                    _ => {}
                }
            }
        }
    }

    /// Process active tournaments events
    async fn process_active_tournaments(
        contract: &mut crate::Game2048Contract,
        update: &StreamUpdate,
        event_index: u32,
    ) {
        if let Some(game2048::GameEvent::ActiveTournaments {
            tournaments,
            timestamp,
        }) = EventReader::read_active_tournaments_event_from_chain(
            contract,
            update.chain_id,
            event_index,
        ) {
            // Update local tournament cache with the new data
            Self::update_local_tournament_cache(contract, tournaments, timestamp).await;
        }
    }

    /// Update local tournament cache with latest data from main chain
    async fn update_local_tournament_cache(
        contract: &mut crate::Game2048Contract,
        tournaments: Vec<game2048::TournamentInfo>,
        timestamp: u64,
    ) {
        // Clear existing cache
        let mut keys_to_remove = Vec::new();
        contract
            .state
            .tournaments_cache_json
            .for_each_index_while(|key| {
                keys_to_remove.push(key);
                Ok(true) // Continue iteration
            })
            .await
            .unwrap();

        for key in keys_to_remove {
            contract.state.tournaments_cache_json.remove(&key).unwrap();
        }

        // Add all tournaments to cache as JSON
        for tournament in tournaments {
            let tournament_id = tournament.tournament_id.clone();
            if let Ok(tournament_json) = serde_json::to_string(&tournament) {
                contract
                    .state
                    .tournaments_cache_json
                    .insert(&tournament_id, tournament_json)
                    .unwrap();
            }
        }

        // Update timestamp
        contract.state.last_tournament_update.set(timestamp);
    }
}
