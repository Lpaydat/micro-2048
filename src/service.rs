#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::collections::HashMap;
use std::sync::Arc;

use self::state::Game2048;
use async_graphql::{EmptySubscription, Object, Schema, SimpleObject};
use game2048::{
    Direction, EliminationGameSettings, EventLeaderboardAction, EventLeaderboardSettings, Game,
    MultiplayerGameAction, Operation,
};
use linera_sdk::{base::WithServiceAbi, bcs, views::View, Service, ServiceRuntime};

pub struct Game2048Service {
    state: Arc<Game2048>,
    // runtime: Arc<Mutex<ServiceRuntime<Self>>>,
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
            // runtime: Arc::new(Mutex::new(runtime)),
        }
    }

    async fn handle_query(&self, query: Self::Query) -> Self::QueryResponse {
        let schema = Schema::build(
            QueryRoot {
                state: self.state.clone(),
                // runtime: self.runtime.clone(),
            },
            MutationRoot {
                state: self.state.clone(),
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

#[derive(SimpleObject)]
struct BoardState {
    board_id: String,
    board: [[u16; 4]; 4],
    is_ended: bool,
    score: u64,
    player: String,
    chain_id: String,
    leaderboard_id: Option<String>,
}

#[derive(SimpleObject)]
struct LeaderboardEntry {
    username: String,
    score: u64,
}

#[derive(SimpleObject)]
struct EliminationGameState {
    game_id: String,
    chain_id: String,
    game_name: String,
    host: String,
    players: Vec<String>,
    status: String,
    total_rounds: u8,
    current_round: u8,
    max_players: u8,
    eliminated_per_trigger: u8,
    trigger_interval_seconds: u16,
    round_leaderboard: Vec<EliminationGameRoundLeaderboard>,
    game_leaderboard: Vec<LeaderboardEntry>,
    created_time: String,
    last_updated_time: String,
}

#[derive(SimpleObject)]
struct EliminationGameRoundLeaderboard {
    round: u8,
    players: Vec<LeaderboardEntry>,
    eliminated_players: Vec<LeaderboardEntry>,
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
}

#[derive(SimpleObject)]
struct Player {
    username: String,
    chain_id: String,
    is_admin: bool,
}

#[derive(SimpleObject, serde::Serialize)]
struct Ranker {
    username: String,
    score: u64,
    board_id: String,
}

#[Object]
impl QueryRoot {
    async fn player(&self, username: String) -> Option<Player> {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&username).await {
            Some(Player {
                username: player.username.get().to_string(),
                chain_id: player.chain_id.get().to_string(),
                is_admin: *player.is_admin.get(),
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
                    is_admin: *player.is_admin.get(),
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

    async fn board(&self, board_id: String) -> Option<BoardState> {
        if let Ok(Some(game)) = self.state.boards.try_load_entry(&board_id).await {
            let game_state = BoardState {
                board_id: game.board_id.get().to_string(),
                board: Game::convert_to_matrix(*game.board.get()),
                is_ended: *game.is_ended.get(),
                score: *game.score.get(),
                player: game.player.get().to_string(),
                chain_id: game.chain_id.get().to_string(),
                leaderboard_id: Some(game.leaderboard_id.get().to_string()),
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
                    leaderboard_id: Some(board.leaderboard_id.get().to_string()),
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
                            score,
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
                });
            }
        }

        tournament_games
    }

    async fn elimination_games(&self) -> Vec<EliminationGameState> {
        let mut waiting_rooms_ids: Vec<String> = Vec::new();
        self.state
            .waiting_rooms
            .for_each_index_value(|game_id, _| {
                waiting_rooms_ids.push(game_id);
                Ok(())
            })
            .await
            .unwrap();

        let mut waiting_rooms: Vec<EliminationGameState> = Vec::new();
        for game_id in waiting_rooms_ids {
            if let Ok(Some(game)) = self.state.elimination_games.try_load_entry(&game_id).await {
                let game_state = EliminationGameState {
                    game_id: game.game_id.get().to_string(),
                    chain_id: game.chain_id.get().to_string(),
                    game_name: game.game_name.get().to_string(),
                    host: game.host.get().to_string(),
                    players: game.players.get().to_vec(),
                    status: format!("{:?}", game.status.get()),
                    total_rounds: *game.total_rounds.get(),
                    current_round: *game.current_round.get(),
                    max_players: *game.max_players.get(),
                    eliminated_per_trigger: *game.eliminated_per_trigger.get(),
                    trigger_interval_seconds: *game.trigger_interval_seconds.get(),
                    round_leaderboard: Vec::new(),
                    game_leaderboard: Vec::new(),
                    created_time: game.created_time.get().to_string(),
                    last_updated_time: game.last_updated_time.get().to_string(),
                };
                waiting_rooms.push(game_state);
            }
        }

        waiting_rooms
    }

    async fn elimination_game(
        &self,
        game_id: String,
        round: Option<u8>,
    ) -> Option<EliminationGameState> {
        // let round = round.unwrap_or(0);
        if let Ok(Some(game)) = self.state.elimination_games.try_load_entry(&game_id).await {
            let mut game_leaderboard: Vec<LeaderboardEntry> = Vec::new();
            game.game_leaderboard
                .for_each_index_value(|username, score| {
                    game_leaderboard.push(LeaderboardEntry { username, score });
                    Ok(())
                })
                .await
                .unwrap();

            let round = round.unwrap_or(*game.current_round.get());
            let mut round_leaderboard: Vec<EliminationGameRoundLeaderboard> = Vec::new();
            let mut round_players: Vec<LeaderboardEntry> = Vec::new();
            let mut round_eliminated_players: Vec<LeaderboardEntry> = Vec::new();
            if let Ok(Some(leaderboard)) = game.round_leaderboard.try_load_entry(&round).await {
                leaderboard
                    .players
                    .for_each_index_value(|username, score| {
                        round_players.push(LeaderboardEntry { username, score });
                        Ok(())
                    })
                    .await
                    .unwrap();

                leaderboard
                    .eliminated_players
                    .for_each_index_value(|username, score| {
                        round_eliminated_players.push(LeaderboardEntry { username, score });
                        Ok(())
                    })
                    .await
                    .unwrap();

                round_leaderboard.push(EliminationGameRoundLeaderboard {
                    round,
                    players: round_players,
                    eliminated_players: round_eliminated_players,
                });
            }

            let game_state = EliminationGameState {
                game_id: game.game_id.get().to_string(),
                chain_id: game.chain_id.get().to_string(),
                game_name: game.game_name.get().to_string(),
                host: game.host.get().to_string(),
                players: game.players.get().to_vec(),
                status: format!("{:?}", game.status.get()),
                total_rounds: *game.total_rounds.get(),
                current_round: *game.current_round.get(),
                max_players: *game.max_players.get(),
                eliminated_per_trigger: *game.eliminated_per_trigger.get(),
                trigger_interval_seconds: *game.trigger_interval_seconds.get(),
                round_leaderboard,
                game_leaderboard,
                created_time: game.created_time.get().to_string(),
                last_updated_time: game.last_updated_time.get().to_string(),
            };
            Some(game_state)
        } else {
            None
        }
    }
}

struct MutationRoot {
    state: Arc<Game2048>,
}

#[Object]
impl MutationRoot {
    async fn register_player(&self, username: String, password_hash: String) -> Vec<u8> {
        let operation = Operation::RegisterPlayer {
            username,
            password_hash,
        };
        bcs::to_bytes(&operation).unwrap()
    }

    async fn new_board(
        &self,
        seed: Option<String>,
        player: String,
        password_hash: String,
        timestamp: String,
        leaderboard_id: Option<String>,
    ) -> Vec<u8> {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&player).await {
            if *player.password_hash.get() != password_hash {
                panic!("Invalid password");
            }
        }

        let seed = seed.unwrap_or("".to_string());
        bcs::to_bytes(&Operation::NewBoard {
            seed,
            player,
            timestamp: timestamp.parse::<u64>().unwrap(),
            leaderboard_id,
        })
        .unwrap()
    }

    async fn make_move(
        &self,
        board_id: String,
        direction: Direction,
        player: String,
        timestamp: String,
        password_hash: String,
    ) -> Vec<u8> {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&player).await {
            if *player.password_hash.get() != password_hash {
                panic!("Invalid password");
            }
        }

        let operation = Operation::MakeMove {
            board_id,
            direction,
            player,
            timestamp: timestamp.parse::<u64>().unwrap(),
        };
        bcs::to_bytes(&operation).unwrap()
    }

    async fn create_elimination_game(
        &self,
        player: String,
        password_hash: String,
        settings: EliminationGameSettings,
    ) -> Vec<u8> {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&player).await {
            if *player.password_hash.get() != password_hash {
                panic!("Invalid password");
            }
        }

        let operation = Operation::CreateEliminationGame { player, settings };
        bcs::to_bytes(&operation).unwrap()
    }

    async fn elimination_game_action(
        &self,
        game_id: String,
        action: MultiplayerGameAction,
        player: String,
        password_hash: String,
        timestamp: String,
    ) -> Vec<u8> {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&player).await {
            if *player.password_hash.get() != password_hash {
                panic!("Invalid password");
            }
        }

        let operation = Operation::EliminationGameAction {
            game_id,
            action,
            player,
            timestamp: timestamp.parse::<u64>().unwrap(),
        };
        bcs::to_bytes(&operation).unwrap()
    }

    async fn event_leaderboard_action(
        &self,
        leaderboard_id: String,
        action: EventLeaderboardAction,
        settings: EventLeaderboardSettings,
        player: String,
        password_hash: String,
        timestamp: String,
    ) -> Vec<u8> {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&player).await {
            if *player.password_hash.get() != password_hash {
                panic!("Invalid password");
            }
        }

        let operation = Operation::EventLeaderboardAction {
            leaderboard_id,
            action,
            settings,
            player,
            timestamp: timestamp.parse::<u64>().unwrap(),
        };
        bcs::to_bytes(&operation).unwrap()
    }

    async fn toggle_admin(
        &self,
        player: String,
        password_hash: String,
        username: String,
    ) -> Vec<u8> {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&player).await {
            if *player.password_hash.get() != password_hash {
                panic!("Invalid password");
            }
            if *player.username.get() != "lpaydat" {
                panic!("Only lpaydat can toggle admin");
            }
        }

        let operation = Operation::ToggleAdmin { username };
        bcs::to_bytes(&operation).unwrap()
    }
}
