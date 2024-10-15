#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::{Arc, Mutex};

use self::state::Game2048;
use async_graphql::{EmptySubscription, Object, Schema};
use game2048::{gen_range, Direction, Operation};
use linera_sdk::{base::WithServiceAbi, bcs, views::View, Service, ServiceRuntime};

pub struct Game2048Service {
    state: Arc<Game2048>,
    runtime: Arc<Mutex<ServiceRuntime<Self>>>,
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
            runtime: Arc::new(Mutex::new(runtime)),
        }
    }

    async fn handle_query(&self, query: Self::Query) -> Self::QueryResponse {
        let schema = Schema::build(
            QueryRoot {
                state: self.state.clone(),
                // runtime: self.runtime.clone(),
            },
            MutationRoot {
                runtime: self.runtime.clone(),
            },
            EmptySubscription,
        )
        .finish();
        schema.execute(query).await
    }
}

struct QueryRoot {
    state: Arc<Game2048>,
    // runtime: Arc<Mutex<ServiceRuntime<Game2048Service>>>,
}

#[Object]
impl QueryRoot {
    async fn game(&self, game_id: u32) -> Option<u64> {
        if let Ok(Some(game)) = self.state.games.try_load_entry(&game_id).await {
            Some(*game.board.get())
        } else {
            None
        }
    }
}

struct MutationRoot {
    runtime: Arc<Mutex<ServiceRuntime<Game2048Service>>>,
}

#[Object]
impl MutationRoot {
    async fn start_game(&self) -> Vec<u8> {
        let runtime = self.runtime.try_lock().unwrap();
        let seed = runtime.next_block_height().to_string();
        let new_seed = gen_range(&seed, 0, 1);
        bcs::to_bytes(&Operation::StartGame { seed: new_seed }).unwrap()
    }

    async fn make_move(&self, game_id: u32, directions: Vec<Direction>) -> Vec<u8> {
        bcs::to_bytes(&Operation::MakeMove {
            game_id,
            directions,
        })
        .unwrap()
    }
}
