// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

use std::sync::Arc;

use async_graphql::{EmptySubscription, Request, Response, Schema, MergedObject};
use linera_sdk::{linera_base_types::WithServiceAbi, Service, ServiceRuntime, views::View};

use gamehub::{
    infrastructure::{state::GameHubState, operations::Operation},
    GameHubAbi, ScheduleOperation,
    api::{
        queries::{
            player_queries::PlayerQueries,
            leaderboard_queries::LeaderboardQueries,
            game_queries::GameQueries,
            event_queries::EventQueries,
            admin_queries::AdminQueries,
            analytics_queries::AnalyticsQueries,
            config_queries::ConfigQueries,
        },
        mutations::{
            player_mutations::PlayerMutations,
            moderation_mutations::ModerationMutations,
            game_mutations::GameMutations,
            admin_mutations::AdminMutations,
            config_mutations::ConfigMutations,
        },
    },
};

// ANCHOR: service_struct
linera_sdk::service!(GameHubService);

pub struct GameHubService {
    state: Arc<GameHubState>,
    runtime: Arc<ServiceRuntime<Self>>,
}
// ANCHOR_END: service_struct

// ANCHOR: declare_abi
impl WithServiceAbi for GameHubService {
    type Abi = GameHubAbi;
}
// ANCHOR_END: declare_abi

impl Service for GameHubService {
    type Parameters = ();

    // ANCHOR: new
    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = GameHubState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        GameHubService {
            state: Arc::new(state),
            runtime: Arc::new(runtime),
        }
    }
    // ANCHOR_END: new

    // ANCHOR: handle_query  
    async fn handle_query(&self, request: Request) -> Response {
        let schema = Schema::build(
            QueryRoot::default_with_state(Arc::clone(&self.state)),
            MutationRoot::default_with_runtime(Arc::clone(&self.runtime)),
            EmptySubscription,
        )
        .data(self.runtime.clone())
        .finish();
        schema.execute(request).await
    }
    // ANCHOR_END: handle_query
}

impl GameHubService {
    /// Get a reference to the application state (used to avoid dead code warning)
    #[allow(dead_code)]
    pub fn state(&self) -> &Arc<GameHubState> {
        &self.state
    }
}

// ANCHOR: query_root
#[derive(MergedObject)]
struct QueryRoot(
    PlayerQueries,
    LeaderboardQueries,
    GameQueries,
    EventQueries,
    AdminQueries,
    AnalyticsQueries,
    ConfigQueries,
);

impl QueryRoot {
    fn default_with_state(state: Arc<GameHubState>) -> Self {
        QueryRoot(
            PlayerQueries { state: Arc::clone(&state) },
            LeaderboardQueries { state: Arc::clone(&state) },
            GameQueries { state: Arc::clone(&state) },
            EventQueries { state: Arc::clone(&state) },
            AdminQueries { state: Arc::clone(&state) },
            AnalyticsQueries { state: Arc::clone(&state) },
            ConfigQueries { state: Arc::clone(&state) },
        )
    }
}
// ANCHOR_END: query_root

// ANCHOR: mutation_root
#[derive(MergedObject)]
struct MutationRoot(
    PlayerMutations,
    ModerationMutations,
    GameMutations,
    AdminMutations,
    ConfigMutations,
);

// Wrapper struct to implement the ScheduleOperation trait
struct RuntimeWrapper {
    runtime: Arc<ServiceRuntime<GameHubService>>,
}

impl ScheduleOperation for RuntimeWrapper {
    fn schedule_operation(&self, operation: &Operation) {
        self.runtime.schedule_operation(operation);
    }
}

impl MutationRoot {
    fn default_with_runtime(runtime: Arc<ServiceRuntime<GameHubService>>) -> Self {
        let wrapper: Arc<dyn ScheduleOperation> = Arc::new(RuntimeWrapper { runtime });
        MutationRoot(
            PlayerMutations { runtime: Arc::clone(&wrapper) },
            ModerationMutations { runtime: Arc::clone(&wrapper) },
            GameMutations { runtime: Arc::clone(&wrapper) },
            AdminMutations { runtime: Arc::clone(&wrapper) },
            ConfigMutations { runtime: Arc::clone(&wrapper) },
        )
    }
}
// ANCHOR_END: mutation_root

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_introspection() {
        // Basic test to ensure schema builds without panics
        let query_root = QueryRoot::default_with_state(Arc::new(unsafe { std::mem::zeroed() }));
        let mutation_root = MutationRoot::default_with_runtime(Arc::new(unsafe { std::mem::zeroed() }));

        // This should not panic if the GraphQL schema is valid
        let _schema = Schema::build(query_root, mutation_root, EmptySubscription).finish();
    }
}