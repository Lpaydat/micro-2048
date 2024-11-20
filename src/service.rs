#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::collections::HashMap;
use std::sync::Arc;

use self::state::Game2048;
use async_graphql::{EmptySubscription, Object, Schema, SimpleObject};
use game2048::{Direction, EliminationGameSettings, Game, MultiplayerGameAction, Operation};
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
            MutationRoot,
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
struct Player {
    username: String,
    chain_id: String,
    highest_score: u64,
}

#[derive(SimpleObject)]
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
                highest_score: *player.highest_score.get(),
            })
        } else {
            None
        }
    }

    async fn players(&self, usernames: Vec<String>) -> Vec<Player> {
        let mut players: Vec<Player> = Vec::new();
        for username in usernames {
            if let Ok(Some(player)) = self.state.players.try_load_entry(&username).await {
                players.push(Player {
                    username,
                    chain_id: player.chain_id.get().to_string(),
                    highest_score: *player.highest_score.get(),
                });
            }
        }
        players
    }

    async fn leaderboard(&self) -> Vec<Ranker> {
        let mut players: HashMap<String, Ranker> = HashMap::new();

        if let Ok(Some(leaderboard)) = self.state.singleplayer_leaderboard.try_load_entry(&0).await
        {
            leaderboard
                .rankers
                .for_each_index_value(|username, score| {
                    players.insert(
                        username.clone(),
                        Ranker {
                            username,
                            score,
                            board_id: "".to_string(),
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
        }

        players.into_values().collect()
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
            };
            Some(game_state)
        } else {
            None
        }
    }

    async fn waiting_rooms(&self) -> Vec<EliminationGameState> {
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

struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn register_player(&self, username: String, password_hash: String) -> Vec<u8> {
        let operation = Operation::RegisterPlayer {
            username,
            password_hash,
        };
        bcs::to_bytes(&operation).unwrap()
    }

    async fn new_board(&self, seed: Option<u32>, player: String) -> Vec<u8> {
        let seed = seed.unwrap_or(0);
        bcs::to_bytes(&Operation::NewBoard { seed, player }).unwrap()
    }

    async fn make_move(&self, board_id: String, direction: Direction) -> Vec<u8> {
        let operation = Operation::MakeMove {
            board_id,
            direction,
        };
        bcs::to_bytes(&operation).unwrap()
    }

    async fn create_elimination_game(
        &self,
        player: String,
        settings: EliminationGameSettings,
    ) -> Vec<u8> {
        let operation = Operation::CreateEliminationGame { player, settings };
        bcs::to_bytes(&operation).unwrap()
    }

    async fn join_elimination_game(&self, game_id: String, player: String) -> Vec<u8> {
        let operation = Operation::EliminationGameAction {
            game_id,
            action: MultiplayerGameAction::Join,
            player,
            timestamp: 0,
        };
        bcs::to_bytes(&operation).unwrap()
    }

    async fn leave_elimination_game(&self, game_id: String, player: String) -> Vec<u8> {
        let operation = Operation::EliminationGameAction {
            game_id,
            action: MultiplayerGameAction::Leave,
            player,
            timestamp: 0,
        };
        bcs::to_bytes(&operation).unwrap()
    }

    async fn start_elimination_game(
        &self,
        game_id: String,
        player: String,
        timestamp: String,
    ) -> Vec<u8> {
        let operation = Operation::EliminationGameAction {
            game_id,
            action: MultiplayerGameAction::Start,
            player,
            timestamp: timestamp.parse::<u64>().unwrap(),
        };
        bcs::to_bytes(&operation).unwrap()
    }

    // for early end
    async fn end_elimination_game(
        &self,
        game_id: String,
        player: String,
        timestamp: String,
    ) -> Vec<u8> {
        let operation = Operation::EliminationGameAction {
            game_id,
            action: MultiplayerGameAction::End,
            player,
            timestamp: timestamp.parse::<u64>().unwrap(),
        };
        bcs::to_bytes(&operation).unwrap()
    }

    async fn trigger_elimination_game(
        &self,
        game_id: String,
        player: String,
        timestamp: String,
    ) -> Vec<u8> {
        let operation = Operation::EliminationGameAction {
            game_id,
            action: MultiplayerGameAction::Trigger,
            player,
            timestamp: timestamp.parse::<u64>().unwrap(),
        };
        bcs::to_bytes(&operation).unwrap()
    }

    async fn next_round_elimination_game(
        &self,
        game_id: String,
        player: String,
        timestamp: String,
    ) -> Vec<u8> {
        let operation = Operation::EliminationGameAction {
            game_id,
            action: MultiplayerGameAction::NextRound,
            player,
            timestamp: timestamp.parse::<u64>().unwrap(),
        };
        bcs::to_bytes(&operation).unwrap()
    }
}
