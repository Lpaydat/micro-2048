//! Service binary entry point for the Linera 2048 game.

#![cfg_attr(target_arch = "wasm32", no_main)]

use std::sync::Arc;

use async_graphql::{EmptySubscription, Object, Request, Response, Schema};
use linera_sdk::{
    linera_base_types::WithServiceAbi,
    views::View,
    Service, ServiceRuntime,
};

use game2048::{
    infrastructure::contract::Game2048Abi,
};

linera_sdk::service!(Game2048Service);

pub struct Game2048Service {
    runtime: Arc<ServiceRuntime<Self>>,
}

impl WithServiceAbi for Game2048Service {
    type Abi = Game2048Abi;
}

impl Service for Game2048Service {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        Game2048Service {
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, request: Request) -> Response {
        let schema = Schema::build(
            QueryRoot {
                participants_count: 0,
                active_sessions_count: 0,
            },
            MutationRoot,
            EmptySubscription,
        )
        .finish();
        schema.execute(request).await
    }
}

/// Query root for GraphQL
struct QueryRoot {
    participants_count: usize,
    active_sessions_count: usize,
}

#[Object]
impl QueryRoot {
    /// Get the number of registered participants
    async fn participants_count(&self) -> usize {
        self.participants_count
    }

    /// Get the number of active game sessions
    async fn active_sessions_count(&self) -> usize {
        self.active_sessions_count
    }

    /// Get service status
    async fn status(&self) -> String {
        format!("Game2048 service is running. Active sessions: {}, Total participants: {}", 
            self.active_sessions_count, self.participants_count)
    }
}

/// Mutation root for GraphQL
struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Register a new participant (placeholder)
    async fn register_participant(&self, username: String) -> String {
        format!("Registration request submitted for username: {}", username)
    }
}