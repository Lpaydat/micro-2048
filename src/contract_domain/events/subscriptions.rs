//! Stream Subscriptions
//!
//! Utilities for managing stream subscriptions to remote chains.

use linera_sdk::linera_base_types::ChainId;

/// Subscription management utilities
pub struct SubscriptionManager;

impl SubscriptionManager {
    /// Subscribes to player score events from another chain
    pub fn subscribe_to_player_score_events(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
    ) {
        use linera_sdk::linera_base_types::{ApplicationId, StreamName};
        let stream_name = StreamName::from("player_score_update".to_string());
        let application_id = ApplicationId::new(
            contract
                .runtime
                .application_id()
                .application_description_hash,
        );

        contract
            .runtime
            .subscribe_to_events(chain_id, application_id, stream_name);
    }

    /// Subscribes to shard score events from another chain
    pub fn subscribe_to_shard_score_events(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
    ) {
        use linera_sdk::linera_base_types::{ApplicationId, StreamName};
        let stream_name = StreamName::from("shard_score_update".to_string());
        let application_id = ApplicationId::new(
            contract
                .runtime
                .application_id()
                .application_description_hash,
        );

        contract
            .runtime
            .subscribe_to_events(chain_id, application_id, stream_name);
    }

    /// Subscribe to leaderboard update events from another chain
    pub fn subscribe_to_leaderboard_update_events(
        contract: &mut crate::Game2048Contract,
        chain_id: ChainId,
    ) {
        use linera_sdk::linera_base_types::{ApplicationId, StreamName};
        let stream_name = StreamName::from("leaderboard_update".to_string());
        let application_id = ApplicationId::new(
            contract
                .runtime
                .application_id()
                .application_description_hash,
        );

        contract
            .runtime
            .subscribe_to_events(chain_id, application_id, stream_name);
    }


}