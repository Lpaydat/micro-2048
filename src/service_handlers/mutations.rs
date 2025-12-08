use crate::service_handlers::types::millis_to_micros;
use crate::state::Game2048;
use crate::Game2048Service;
use async_graphql::Object;
use game2048::{LeaderboardAction, LeaderboardSettings, Operation};
use linera_sdk::ServiceRuntime;
use std::sync::Arc;

pub struct MutationHandler {
    pub state: Arc<Game2048>,
    pub runtime: Arc<ServiceRuntime<Game2048Service>>,
}

#[Object]
impl MutationHandler {
    async fn register_player(&self, username: String, password_hash: String) -> [u8; 0] {
        let operation = Operation::RegisterPlayer {
            username,
            password_hash,
        };
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn new_board(
        &self,
        player: String,
        password_hash: String,
        timestamp: String,
        leaderboard_id: String, // Leaderboard ID parameter
    ) -> [u8; 0] {
        // Validate player exists and password is correct
        self.validate_player_password(&player, &password_hash).await;

        // Convert timestamp from milliseconds to microseconds
        let timestamp_micros = millis_to_micros(&timestamp).expect("Invalid timestamp format");

        let operation = Operation::NewBoard {
            player,
            timestamp: timestamp_micros,
            password_hash,
            leaderboard_id, // Use provided leaderboard ID
        };
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn new_shard(&self) -> [u8; 0] {
        let operation = Operation::NewShard;
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn make_moves(
        &self,
        board_id: String,
        moves: String,
        player: String,
        password_hash: String,
    ) -> [u8; 0] {
        // Validate player exists and password is correct
        self.validate_player_password(&player, &password_hash).await;

        let operation = Operation::MakeMoves {
            board_id,
            moves,
            player,
            password_hash,
        };
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn leaderboard_action(
        &self,
        leaderboard_id: String,
        action: LeaderboardAction,
        settings: LeaderboardSettings,
        player: String,
        password_hash: String,
    ) -> [u8; 0] {
        // Validate player exists and password is correct
        self.validate_player_password(&player, &password_hash).await;

        // Convert timestamps from milliseconds to microseconds
        let converted_settings = LeaderboardSettings {
            name: settings.name,
            description: settings.description,
            start_time: millis_to_micros(&settings.start_time)
                .expect("Invalid start_time")
                .to_string(),
            end_time: millis_to_micros(&settings.end_time)
                .expect("Invalid end_time")
                .to_string(),
            shard_number: settings.shard_number,
            base_triggerer_count: settings.base_triggerer_count,
        };

        let operation = Operation::LeaderboardAction {
            leaderboard_id,
            action,
            settings: converted_settings,
            player,
            password_hash,
        };
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn toggle_mod(&self, player: String, password_hash: String, username: String) -> [u8; 0] {
        // Validate player exists and password is correct
        self.validate_player_password(&player, &password_hash).await;

        // Additional admin check
        if player != "lpaydat" {
            panic!("Only lpaydat can toggle admin");
        }

        let operation = Operation::ToggleAdmin {
            username,
            player,
            password_hash,
        };
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn faucet(&self) -> [u8; 0] {
        let operation = Operation::Faucet;
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn close_chain(&self, chain_id: String) -> [u8; 0] {
        let operation = Operation::CloseChain { chain_id };
        self.runtime.schedule_operation(&operation);
        []
    }

    /// 🚀 NEW: Trigger score aggregation from monitored player chains (for shard chains)
    async fn aggregate_scores(&self) -> [u8; 0] {
        let operation = Operation::AggregateScores;
        self.runtime.schedule_operation(&operation);
        []
    }

    /// 🚀 NEW: Trigger leaderboard update from registered shard chains (for leaderboard chains)
    async fn update_leaderboard(&self) -> [u8; 0] {
        let operation = Operation::UpdateLeaderboard;
        self.runtime.schedule_operation(&operation);
        []
    }

    /// 🚀 NEW: Emit current active tournaments (for leaderboard chains)
    async fn update_active_tournaments(&self) -> [u8; 0] {
        let operation = Operation::UpdateActiveTournaments;
        self.runtime.schedule_operation(&operation);
        []
    }

    /// 🚀 IMPROVED: Request centralized aggregation (with client-side authorization check)
    async fn request_aggregation(&self, requester_chain_id: String) -> [u8; 0] {
        // Check authorization on the client side first
        let is_authorized =
            if let Ok(Some(leaderboard)) = self.state.leaderboards.try_load_entry("").await {
                // Check if primary triggerer
                if leaderboard.primary_triggerer.get() == &requester_chain_id {
                    true
                } else {
                    // Check backup triggerers
                    if let Ok(backups) = leaderboard.backup_triggerers.read_front(5).await {
                        backups.contains(&requester_chain_id)
                    } else {
                        false
                    }
                }
            } else {
                false
            };

        if !is_authorized {
            panic!(
                "Not authorized to trigger aggregation. Chain {} is not in the triggerer pool.",
                requester_chain_id
            );
        }

        // 🚀 IMPROVED: Check cooldown using runtime system time (more reliable)
        if let Ok(Some(leaderboard)) = self.state.leaderboards.try_load_entry("").await {
            // Use runtime's system time for consistency
            let current_time = self.runtime.system_time().micros();

            let cooldown_until = *leaderboard.trigger_cooldown_until.get();
            if current_time < cooldown_until {
                let remaining = (cooldown_until - current_time) / 1_000_000;
                panic!(
                    "Aggregation on cooldown. Please wait {} seconds.",
                    remaining
                );
            }

            // Also check staleness to prevent unnecessary triggers
            let last_trigger = *leaderboard.last_trigger_time.get();
            let time_since_last = current_time.saturating_sub(last_trigger);

            // Require at least 3 seconds between triggers (even if cooldown expired)
            if time_since_last < 3_000_000 {
                panic!("Too soon since last trigger. Please wait a moment.");
            }
        }

        // Proceed with the operation if authorized and not on cooldown
        let operation = Operation::RequestAggregation { requester_chain_id };
        self.runtime.schedule_operation(&operation);
        []
    }

    /// 🚀 ADMIN: Configure base triggerer count
    async fn configure_triggerer_count(
        &self,
        admin_username: String,
        password_hash: String,
        base_triggerer_count: u32,
    ) -> [u8; 0] {
        let operation = Operation::ConfigureTriggererCount {
            admin_username,
            password_hash,
            base_triggerer_count,
        };
        self.runtime.schedule_operation(&operation);
        []
    }

    /// 🚀 NEW: Manual leaderboard refresh - player can trigger update when their score is higher
    /// This sends a TriggerUpdate message directly to the leaderboard chain
    async fn request_leaderboard_refresh(
        &self,
        player: String,
        password_hash: String,
        leaderboard_id: String,
    ) -> [u8; 0] {
        // Validate player exists and password is correct
        self.validate_player_password(&player, &password_hash).await;

        let operation = Operation::RequestLeaderboardRefresh {
            player,
            password_hash,
            leaderboard_id,
        };
        self.runtime.schedule_operation(&operation);
        []
    }

    // ============================================
    // CHAIN POOL MUTATIONS
    // ============================================

    /// 🚀 ADMIN: Refill the chain pool with pre-created player chains
    /// Only callable on main chain by the chain owner (admin)
    async fn refill_chain_pool(&self, count: u32) -> [u8; 0] {
        // No password validation needed - Linera's chain ownership enforces admin access
        // Only the chain owner can submit operations to the main chain

        if count == 0 {
            panic!("Count must be greater than 0");
        }
        if count > 500 {
            panic!("Count must be at most 500 per call");
        }

        let operation = Operation::RefillChainPool { count };
        self.runtime.schedule_operation(&operation);
        []
    }
    
    /// 🚀 MESSAGE-BASED: Claim player chain after registration
    /// This triggers block production which processes the inbox messages
    /// (RegisterPlayer, SubscribeToMainChain)
    async fn claim_chain(&self) -> [u8; 0] {
        let operation = Operation::ClaimChain;
        self.runtime.schedule_operation(&operation);
        []
    }
}

impl MutationHandler {
    async fn validate_player_password(&self, player_username: &str, provided_password_hash: &str) {
        if let Ok(Some(player)) = self.state.players.try_load_entry(player_username).await {
            let stored_password_hash = player.password_hash.get().to_string();
            if stored_password_hash != provided_password_hash {
                panic!("Invalid password");
            }
        } else {
            panic!("Player not found");
        }
    }
}
