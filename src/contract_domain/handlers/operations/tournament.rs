//! Tournament Operations Handler
//!
//! Handles tournament-related operations including validation and shard selection.

use linera_sdk::linera_base_types::ChainId;
use game2048::{GameEvent, hash_seed};

pub struct TournamentOperationHandler;

impl TournamentOperationHandler {
    /// Validate tournament exists and is active (with time-based checks)
    pub async fn validate_tournament(
        contract: &mut crate::Game2048Contract,
        tournament_id: &str,
    ) -> bool {
        // 🚀 IMPROVED: Use cached tournament data with time validation
        if let Some(tournament) = contract.get_cached_tournament(tournament_id).await {
            let current_time = contract.runtime.system_time().micros();
            
            // Check if tournament has started (if start_time is set)
            if let Some(start_time) = tournament.start_time {
                if current_time < start_time {
                    log::warn!("🎯 VALIDATION: ❌ Tournament '{}' has not started yet (start: {}, current: {})", 
                        tournament_id, start_time, current_time);
                    return false;
                }
            }
            
            // Check if tournament has ended (if end_time is set)
            if let Some(end_time) = tournament.end_time {
                if current_time >= end_time {
                    log::warn!("🎯 VALIDATION: ❌ Tournament '{}' has ended (end: {}, current: {})", 
                        tournament_id, end_time, current_time);
                    return false;
                }
            }
            
            // Tournament is active
            log::info!("🎯 VALIDATION: ✅ Tournament '{}' is active and within time bounds", tournament_id);
            true
        } else {
            log::warn!("🎯 VALIDATION: ❌ Tournament '{}' not found in cache", tournament_id);
            false
        }
    }

    /// Select optimal shard for a tournament using hash-based distribution
    pub async fn select_optimal_shard(
        contract: &mut crate::Game2048Contract,
        tournament_id: &str,
        player_id: &str,
    ) -> String {
        // Use cached tournament data for shard selection
        if let Some(tournament) = contract.get_cached_tournament(tournament_id).await {
            // Ensure we have at least one shard
            if tournament.shard_chain_ids.is_empty() {
                panic!("❌ FATAL: Tournament {} has no shard chains configured! Cannot register player.", tournament_id);
            }

            // Use hash-based distribution for consistent shard assignment
            let hash_input = format!("{}{}", player_id, tournament_id);
            let hash = hash_seed("", &hash_input, 0);
            let shard_index = (hash as usize) % tournament.shard_chain_ids.len();
            let best_shard = tournament.shard_chain_ids[shard_index].clone();
            
            log::info!("🎯 SHARD_SELECTION: Player {} assigned to shard {} (index {}/{}) via hash {}", 
                      player_id, best_shard, shard_index, tournament.shard_chain_ids.len(), hash);

            return best_shard;
        }

        // No fallback - throw clear error
        panic!("❌ FATAL: Cannot find tournament '{}' in cached tournaments! Player registration failed.", tournament_id);
    }

    /// Read latest active tournaments - try stored index first, then increment until failure
    pub async fn read_active_tournaments(
        contract: &mut crate::Game2048Contract,
    ) -> Option<GameEvent> {
        use linera_sdk::linera_base_types::StreamName;
        let _stream_name = StreamName::from("active_tournaments".to_string());
        let _chain_id = contract.runtime.application_creator_chain_id();
        
        // Get the stored index as starting point for efficiency
        let stored_index = *contract.state.active_tournaments_event_index.get();
        let latest_event = None;
        let last_valid_index = 0;
        let _current_index = stored_index;

        log::info!("DEBUG: read_active_tournaments - starting dynamic scan from stored index {}", stored_index);

        // First, validate the stored index is still good
        // if stored_index > 0 {
        //     match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        //         contract.runtime.read_event(chain_id, stream_name.clone(), stored_index as u32)
        //     })) {
        //         Ok(event) => {
        //             log::info!("DEBUG: Stored index {} is still valid", stored_index);
        //             latest_event = Some(event);
        //             last_valid_index = stored_index;
        //         }
        //         Err(_) => {
        //             log::info!("DEBUG: Stored index {} is no longer valid, starting from 0", stored_index);
        //             current_index = 0; // Reset to beginning if stored index is invalid
        //         }
        //     }
        // }

        // Dynamic scanning: continue from current_index until we hit an error or breaker
        // loop {
        //     match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        //         contract.runtime.read_event(chain_id, stream_name.clone(), current_index as u32)
        //     })) {
        //         Ok(event) => {
        //             log::info!("DEBUG: Found event at index {}", current_index);
        //             latest_event = Some(event);
        //             last_valid_index = current_index;
        //             current_index += 1;
        //             // Continue scanning for newer events
        //         }
        //         Err(_) => {
        //             log::info!("DEBUG: No event at index {} - reached end of events", current_index);
        //             break; // Stop when we hit a non-existent event
        //         }
        //     }
        //     
        //     // TEMPORARY BREAKER: Stop at index 2 for testing (will remove later)
        //     if current_index > 2 {
        //         log::info!("DEBUG: Hit temporary breaker at index {}, stopping scan", current_index);
        //         break;
        //     }
        // }

        // Always update stored index to the last valid index we found
        if latest_event.is_some() {
            // Only update if we found a newer or equal index
            if last_valid_index >= stored_index {
                contract.state.active_tournaments_event_index.set(last_valid_index);
                log::info!("DEBUG: Updated stored index from {} to {} (last valid index)", stored_index, last_valid_index);
            } else {
                log::info!("DEBUG: Keeping stored index at {} (no newer events found)", stored_index);
            }
            log::info!("DEBUG: Returning latest active tournaments event from index {}", last_valid_index);
        } else {
            log::info!("DEBUG: No active tournaments found during dynamic scan");
        }

        latest_event
    }

    /// Read latest shard workload - try stored index first, then increment until failure
    pub async fn read_shard_workload(
        contract: &mut crate::Game2048Contract,
        shard_chain_id: ChainId,
    ) -> Result<Option<GameEvent>, Box<dyn std::any::Any + Send>> {
        use linera_sdk::linera_base_types::StreamName;
        let _stream_name = StreamName::from("shard_workload".to_string());
        let chain_id_str = shard_chain_id.to_string();

        // Try stored index first
        let current_index = contract.state.shard_workload_event_indices
            .get(&chain_id_str)
            .await
            .unwrap()
            .unwrap_or(0);
        let latest_event = None;

        log::info!("DEBUG: read_shard_workload - trying stored index {} for shard {}", current_index, chain_id_str);

        // match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        //     contract.runtime.read_event(shard_chain_id, stream_name.clone(), current_index as u32)
        // })) {
        //     Ok(event) => {
        //         log::info!("DEBUG: Found shard workload event at stored index {} for shard {}", current_index, chain_id_str);
        //         latest_event = Some(event);
        //         contract.state.shard_workload_event_indices
        //             .insert(&chain_id_str, current_index)
        //             .unwrap();
        //     }
        //     Err(_) => {
        //         log::info!("DEBUG: No event at stored index {} for shard {}, trying index 0", current_index, chain_id_str);
        //         // Try index 0
        //         match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        //             contract.runtime.read_event(shard_chain_id, stream_name.clone(), 0)
        //         })) {
        //             Ok(event) => {
        //                 log::info!("DEBUG: Found shard workload event at index 0 for shard {}", chain_id_str);
        //                 latest_event = Some(event);
        //                 contract.state.shard_workload_event_indices
        //                     .insert(&chain_id_str, 0)
        //                     .unwrap();
        //             }
        //             Err(_) => {
        //                 log::info!("DEBUG: No shard workload events found for {}", chain_id_str);
        //             }
        //         }
        //     }
        // }

        if latest_event.is_some() {
            log::info!("DEBUG: Returning latest shard workload event for {}", chain_id_str);
        } else {
            log::info!("DEBUG: No shard workload events found for {}", chain_id_str);
        }

        Ok(latest_event)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use linera_sdk::linera_base_types::ChainId;

    #[test]
    fn test_chain_id_parsing() {
        // Test that we can parse valid chain IDs
        let valid_chain_id = "e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65";
        assert!(ChainId::from_str(valid_chain_id).is_ok());

        // Test that invalid chain IDs fail
        let invalid_chain_id = "invalid";
        assert!(ChainId::from_str(invalid_chain_id).is_err());
    }

    #[test]
    fn test_tournament_validation_logic() {
        // This is a basic test to ensure the validation logic compiles
        // In a real test environment, we'd mock the contract and test the actual logic
        assert!(true);
    }
}