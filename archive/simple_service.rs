#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::Arc;

use async_graphql::{EmptySubscription, Object, Request, Response, Schema};
use linera_sdk::{
    linera_base_types::WithServiceAbi,
    views::View,
    Service, ServiceRuntime,
};

use self::state::Game2048State;
use game2048::Game2048Abi;

pub struct Game2048Service {
    state: Arc<Game2048State>,
}

linera_sdk::service!(Game2048Service);

impl WithServiceAbi for Game2048Service {
    type Abi = Game2048Abi;
}

impl Service for Game2048Service {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = Game2048State::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        Game2048Service {
            state: Arc::new(state),
        }
    }

    async fn handle_query(&self, request: Request) -> Response {
        let schema = Schema::build(
            QueryRoot {
                state: self.state.clone(),
            },
            MutationRoot,
            EmptySubscription,
        )
        .finish();
        schema.execute(request).await
    }
}

struct QueryRoot {
    state: Arc<Game2048State>,
}

#[Object]
impl QueryRoot {
    /// Get the total number of participants
    async fn participants_count(&self) -> u64 {
        *self.state.participants_count.get()
    }

    /// Get the total number of game sessions
    async fn sessions_count(&self) -> u64 {
        *self.state.sessions_count.get()
    }

    /// Get service status
    async fn status(&self) -> String {
        format!(
            "Game2048 service is running. Sessions: {}, Participants: {}",
            *self.state.sessions_count.get(),
            *self.state.participants_count.get()
        )
    }
}

struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Register a new participant (placeholder)
    async fn register_participant(&self, username: String) -> String {
        format!("Registration request submitted for username: {}", username)
    }
}