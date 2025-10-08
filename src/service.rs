#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};

use self::state::Game2048;
use async_graphql::{EmptySubscription, Object, Schema, SimpleObject};
use game2048::{Game, LeaderboardAction, LeaderboardSettings, Operation};
use linera_sdk::{abi::WithServiceAbi, views::View, Service, ServiceRuntime};

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
                runtime: self.runtime.clone(),
            },
            MutationRoot {
                state: self.state.clone(),
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
    runtime: Arc<Mutex<ServiceRuntime<Game2048Service>>>,
}

#[derive(SimpleObject)]
struct BoardState {
    board_id: String,
    board: [[u16; 4]; 4],
    is_ended: bool,
    score: u64,
    player: String,
    chain_id: String,
    leaderboard_id: String,
    shard_id: String,
    created_at: String,
    end_time: String,
}

#[derive(SimpleObject)]
struct LeaderboardEntry {
    username: String,
    score: u64,
}

#[derive(SimpleObject)]
struct LeaderboardState {
    leaderboard_id: String,
    chain_id: String,
    name: String,
    description: Option<String>,
    is_pinned: bool,
    host: String,
    start_time: String,
    end_time: String,
    total_boards: u32,
    total_players: u32,
    rankers: Vec<Ranker>,
    shard_ids: Vec<String>,
}

#[derive(SimpleObject)]
struct Player {
    username: String,
    chain_id: String,
    is_mod: bool,
}

#[derive(SimpleObject, serde::Serialize)]
struct Ranker {
    username: String,
    score: u64,
    board_id: String,
}

#[derive(SimpleObject)]
struct Shard {
    shard_id: String,
    leaderboard_id: String,
    chain_id: String,
    start_time: String,
    scores: HashMap<String, u64>,
    board_ids: HashMap<String, String>,
    end_time: String,
    counter: u16,
}

#[Object]
impl QueryRoot {
    async fn balance(&self) -> String {
        self.state.balance.get().to_string()
    }

    async fn player(&self, username: String) -> Option<Player> {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&username).await {
            Some(Player {
                username: player.username.get().to_string(),
                chain_id: player.chain_id.get().to_string(),
                is_mod: *player.is_mod.get(),
            })
        } else {
            None
        }
    }

    async fn players(&self, usernames: Option<Vec<String>>) -> Vec<Player> {
        let mut usernames = usernames.unwrap_or_default();
        let mut players: Vec<Player> = Vec::new();

        if usernames.is_empty() {
            usernames = self.state.players.indices().await.unwrap();
        }

        for username in usernames {
            if let Ok(Some(player)) = self.state.players.try_load_entry(&username).await {
                players.push(Player {
                    username,
                    chain_id: player.chain_id.get().to_string(),
                    is_mod: *player.is_mod.get(),
                });
            }
        }

        players
    }

    async fn check_player(&self, username: String, password_hash: String) -> Option<bool> {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&username).await {
            let stored_password_hash = player.password_hash.get().to_string();
            Some(stored_password_hash == password_hash)
        } else {
            return None;
        }
    }

    async fn board(&self, board_id: Option<String>) -> Option<BoardState> {
        let board_id = board_id.unwrap_or(self.state.latest_board_id.get().to_string());
        if let Ok(Some(game)) = self.state.boards.try_load_entry(&board_id).await {
            let game_state = BoardState {
                board_id: game.board_id.get().to_string(),
                board: Game::convert_to_matrix(*game.board.get()),
                is_ended: *game.is_ended.get(),
                score: *game.score.get(),
                player: game.player.get().to_string(),
                chain_id: game.chain_id.get().to_string(),
                leaderboard_id: game.leaderboard_id.get().to_string(),
                shard_id: game.shard_id.get().to_string(),
                created_at: game.created_at.get().to_string(),
                end_time: game.end_time.get().to_string(),
            };
            Some(game_state)
        } else {
            None
        }
    }

    async fn boards(&self, board_ids: Option<Vec<String>>) -> Vec<BoardState> {
        let mut board_ids = board_ids.unwrap_or_default();
        let mut boards: Vec<BoardState> = Vec::new();

        if board_ids.is_empty() {
            board_ids = self.state.boards.indices().await.unwrap();
        }

        for board_id in board_ids {
            if let Ok(Some(board)) = self.state.boards.try_load_entry(&board_id).await {
                boards.push(BoardState {
                    board_id,
                    board: Game::convert_to_matrix(*board.board.get()),
                    is_ended: *board.is_ended.get(),
                    score: *board.score.get(),
                    player: board.player.get().to_string(),
                    chain_id: board.chain_id.get().to_string(),
                    leaderboard_id: board.leaderboard_id.get().to_string(),
                    shard_id: board.shard_id.get().to_string(),
                    created_at: board.created_at.get().to_string(),
                    end_time: board.end_time.get().to_string(),
                });
            }
        }

        boards
    }

    async fn leaderboard(&self, leaderboard_id: Option<String>) -> Option<LeaderboardState> {
        let mut players: HashMap<String, Ranker> = HashMap::new();
        let leaderboard_id = leaderboard_id.unwrap_or("".to_string());

        if let Ok(Some(leaderboard)) = self
            .state
            .leaderboards
            .try_load_entry(&leaderboard_id)
            .await
        {
            leaderboard
                .score
                .for_each_index_value(|username, score| {
                    players.insert(
                        username.clone(),
                        Ranker {
                            username,
                            score: *score,
                            board_id: leaderboard_id.clone(),
                        },
                    );
                    Ok(())
                })
                .await
                .unwrap();
            leaderboard
                .board_ids
                .for_each_index_value(|username, board_id| {
                    if let Some(ranker) = players.get_mut(&username) {
                        ranker.board_id = board_id.to_string();
                    }
                    Ok(())
                })
                .await
                .unwrap();

            let shard_ids = leaderboard.shard_ids.read_front(100).await.unwrap();
            let leaderboard_state = LeaderboardState {
                leaderboard_id,
                chain_id: leaderboard.chain_id.get().to_string(),
                name: leaderboard.name.get().to_string(),
                description: Some(leaderboard.description.get().to_string()),
                is_pinned: *leaderboard.is_pinned.get(),
                host: leaderboard.host.get().to_string(),
                start_time: leaderboard.start_time.get().to_string(),
                end_time: leaderboard.end_time.get().to_string(),
                total_boards: *leaderboard.total_boards.get(),
                total_players: *leaderboard.total_players.get(),
                rankers: players.into_values().collect(),
                shard_ids,
            };

            Some(leaderboard_state)
        } else {
            None
        }
    }

    async fn leaderboards(&self) -> Vec<LeaderboardState> {
        let mut leaderboard_ids: Vec<String> = Vec::new();
        self.state
            .leaderboards
            .for_each_index_while(|leaderboard_id| {
                leaderboard_ids.push(leaderboard_id);
                Ok(true)
            })
            .await
            .unwrap();

        let mut tournament_games: Vec<LeaderboardState> = Vec::new();
        for leaderboard_id in leaderboard_ids {
            if let Ok(Some(leaderboard)) = self
                .state
                .leaderboards
                .try_load_entry(&leaderboard_id)
                .await
            {
                tournament_games.push(LeaderboardState {
                    leaderboard_id,
                    chain_id: leaderboard.chain_id.get().to_string(),
                    name: leaderboard.name.get().to_string(),
                    description: Some(leaderboard.description.get().to_string()),
                    is_pinned: *leaderboard.is_pinned.get(),
                    host: leaderboard.host.get().to_string(),
                    start_time: leaderboard.start_time.get().to_string(),
                    end_time: leaderboard.end_time.get().to_string(),
                    total_boards: *leaderboard.total_boards.get(),
                    total_players: *leaderboard.total_players.get(),
                    rankers: Vec::new(),
                    shard_ids: Vec::new(),
                });
            }
        }

        tournament_games
    }

    async fn shards(&self) -> Shard {
        if let Some(shard) = self.state.shards.try_load_entry("").await.unwrap() {
            let mut scores: HashMap<String, u64> = HashMap::new();
            let mut board_ids: HashMap<String, String> = HashMap::new();
            shard
                .score
                .for_each_index_value(|username, score| {
                    scores.insert(username.clone(), *score);
                    Ok(())
                })
                .await
                .unwrap();
            shard
                .board_ids
                .for_each_index_value(|username, board_id| {
                    board_ids.insert(username.clone(), board_id.to_string());
                    Ok(())
                })
                .await
                .unwrap();
            Shard {
                shard_id: shard.shard_id.get().to_string(),
                leaderboard_id: shard.leaderboard_id.get().to_string(),
                chain_id: shard.chain_id.get().to_string(),
                start_time: shard.start_time.get().to_string(),
                end_time: shard.end_time.get().to_string(),
                counter: *shard.counter.get(),
                scores,
                board_ids,
            }
        } else {
            Shard {
                shard_id: "".to_string(),
                leaderboard_id: "".to_string(),
                chain_id: "".to_string(),
                start_time: "".to_string(),
                end_time: "".to_string(),
                counter: 0,
                scores: HashMap::new(),
                board_ids: HashMap::new(),
            }
        }
    }
}

struct MutationRoot {
    state: Arc<Game2048>,
    runtime: Arc<Mutex<ServiceRuntime<Game2048Service>>>,
}

#[Object]
impl MutationRoot {
    async fn register_player(&self, username: String, password_hash: String) -> [u8; 0] {
        let operation = Operation::RegisterPlayer {
            username,
            password_hash,
        };
        self.runtime.lock().unwrap().schedule_operation(&operation);
        []
    }

    async fn new_board(
        &self,
        player: String,
        password_hash: String,
        player_chain_id: String,
        timestamp: String,
    ) -> [u8; 0] {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&player).await {
            if *player.password_hash.get() != password_hash {
                panic!("Invalid password");
            }
        }

        let operation = Operation::NewBoard {
            player,
            player_chain_id,
            timestamp: timestamp.parse::<u64>().unwrap(),
        };
        self.runtime.lock().unwrap().schedule_operation(&operation);
        []
    }

    async fn new_shard(&self) -> [u8; 0] {
        let operation = Operation::NewShard;
        self.runtime.lock().unwrap().schedule_operation(&operation);
        []
    }

    async fn make_moves(
        &self,
        board_id: String,
        moves: String,
        player: String,
        password_hash: String,
    ) -> [u8; 0] {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&player).await {
            if *player.password_hash.get() != password_hash {
                panic!("Invalid password");
            }
        }

        let operation = Operation::MakeMoves {
            board_id,
            moves,
            player,
        };
        self.runtime.lock().unwrap().schedule_operation(&operation);
        []
    }

    async fn leaderboard_action(
        &self,
        leaderboard_id: String,
        action: LeaderboardAction,
        settings: LeaderboardSettings,
        player: String,
        password_hash: String,
        timestamp: String,
    ) -> [u8; 0] {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&player).await {
            if *player.password_hash.get() != password_hash {
                panic!("Invalid password");
            }
        }

        let operation = Operation::LeaderboardAction {
            leaderboard_id,
            action,
            settings,
            player,
            timestamp: timestamp.parse::<u64>().unwrap(),
        };
        self.runtime.lock().unwrap().schedule_operation(&operation);
        []
    }

    async fn toggle_mod(&self, player: String, password_hash: String, username: String) -> [u8; 0] {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&player).await {
            if *player.password_hash.get() != password_hash {
                panic!("Invalid password");
            }
            if *player.username.get() != "lpaydat" {
                panic!("Only lpaydat can toggle admin");
            }
        }

        let operation = Operation::ToggleAdmin { username };
        self.runtime.lock().unwrap().schedule_operation(&operation);
        []
    }

    async fn faucet(&self) -> [u8; 0] {
        let operation = Operation::Faucet;
        self.runtime.lock().unwrap().schedule_operation(&operation);
        []
    }

    async fn close_chain(&self, chain_id: String) -> [u8; 0] {
        let operation = Operation::CloseChain { chain_id };
        self.runtime.lock().unwrap().schedule_operation(&operation);
        []
    }
}
