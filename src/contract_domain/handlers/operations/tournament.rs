//! Tournament Operations Handler
//!
//! Handles tournament-related operations including validation and shard selection.

use game2048::hash_seed;

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
                    return false;
                }
            }

            // Check if tournament has ended (if end_time is set)
            if let Some(end_time) = tournament.end_time {
                if current_time >= end_time {
                    return false;
                }
            }

            // Tournament is active
            true
        } else {
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

            return best_shard;
        }

        // No fallback - throw clear error
        panic!("❌ FATAL: Cannot find tournament '{}' in cached tournaments! Player registration failed.", tournament_id);
    }
}

#[cfg(test)]
mod tests {

    use linera_sdk::linera_base_types::ChainId;
    use std::str::FromStr;

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
    }
}
