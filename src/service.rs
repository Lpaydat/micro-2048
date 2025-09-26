#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;
mod service_handlers;

use std::sync::Arc;

use self::state::Game2048;
use self::service_handlers::{QueryHandler, MutationHandler, SubscriptionHandler};
use async_graphql::{Request, Response, Schema};
use linera_sdk::{linera_base_types::WithServiceAbi, views::View, Service, ServiceRuntime};

pub struct Game2048Service {
    state: Arc<Game2048>,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(Game2048Service);

impl WithServiceAbi for Game2048Service {
    type Abi = game2048::Game2048Abi;
}

impl Service for Game2048Service {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = Game2048::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        Game2048Service {
            state: Arc::new(state),
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, request: Request) -> Response {
        let schema = Schema::build(
            QueryHandler {
                state: self.state.clone(),
            },
            MutationHandler {
                state: self.state.clone(),
                runtime: self.runtime.clone(),
            },
            SubscriptionHandler {
                state: self.state.clone(),
                runtime: self.runtime.clone(),
            },
        )
        .finish();
        schema.execute(request).await
    }
}

