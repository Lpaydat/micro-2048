#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::str::FromStr;

use linera_sdk::{
    base::{Amount, ApplicationPermissions, ChainId, WithContractAbi},
    views::{RootView, View},
    Contract, ContractRuntime,
};
use state::EliminationGameStatus;

use self::state::Game2048;
use game2048::{
    hash_seed, EventLeaderboardAction, Game, Message, MultiplayerGameAction, Operation,
    RegistrationCheck,
};

pub struct Game2048Contract {
    state: Game2048,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(Game2048Contract);

impl WithContractAbi for Game2048Contract {
    type Abi = game2048::Game2048Abi;
}

impl Contract for Game2048Contract {
    type Message = Message;
    type Parameters = ();
    type InstantiationArgument = u32;

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = Game2048::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        Game2048Contract { state, runtime }
    }

    async fn instantiate(&mut self, _seed: Self::InstantiationArgument) {
        self.runtime.application_parameters();

        let boards = include_str!("../db/boards.txt");
        for line in boards.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let board_id = parts[0];
                let board_hex = parts[1];
                let score = parts[2];
                let is_ended = parts[3];

                let game = self.state.boards.load_entry_mut(board_id).await.unwrap();
                game.board_id.set(board_id.to_string());
                game.board
                    .set(u64::from_str_radix(board_hex.trim_start_matches("0x"), 16).unwrap());
                game.player.set("".to_string());
                game.score.set(score.parse::<u64>().unwrap());
                game.is_ended.set(is_ended.parse::<bool>().unwrap());
            }
        }

        let rankers = include_str!("../db/rankers.txt");
        let leaderboard = self
            .state
            .leaderboards
            .load_entry_mut(&"".to_string())
            .await
            .unwrap();
        for line in rankers.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let username = parts[0];
                let score = parts[1];
                let board_id = parts[2];

                leaderboard
                    .score
                    .insert(username, score.parse::<u64>().unwrap())
                    .unwrap();
                leaderboard
                    .board_ids
                    .insert(username, board_id.to_string())
                    .unwrap();
            }
        }
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            Operation::RegisterPlayer {
                username,
                password_hash,
            } => {
                if username.is_empty() {
                    panic!("Username cannot be empty");
                }
                self.check_player_registered(&username, RegistrationCheck::EnsureNotRegistered)
                    .await;

                let player = self.state.players.load_entry_mut(&username).await.unwrap();

                let chain_ownership = self.runtime.chain_ownership();
                let application_permissions = ApplicationPermissions::default();
                let amount = Amount::from_tokens(0);
                let (_, chain_id) =
                    self.runtime
                        .open_chain(chain_ownership, application_permissions, amount);

                player.username.set(username);
                player.password_hash.set(password_hash);
                player.chain_id.set(chain_id.to_string());
            }
            Operation::NewBoard {
                seed,
                player,
                timestamp,
                leaderboard_id,
            } => {
                self.check_player_registered(&player, RegistrationCheck::EnsureRegistered)
                    .await;

                let board_id = hash_seed(&seed, &player, timestamp).to_string();
                let new_board = Game::new(&board_id, &player, timestamp).board;
                let game = self.state.boards.load_entry_mut(&board_id).await.unwrap();
                let leaderboard_id = leaderboard_id.unwrap_or("".to_string());
                let player_obj = self
                    .state
                    .players
                    .load_entry_or_insert(&player)
                    .await
                    .unwrap();

                game.board_id.set(board_id.clone());
                game.board.set(new_board);
                game.player.set(player.clone());
                game.leaderboard_id.set(leaderboard_id.clone());
                game.chain_id.set(player_obj.chain_id.get().clone());

                let leaderboard = self
                    .state
                    .leaderboards
                    .load_entry_mut(&leaderboard_id.clone())
                    .await
                    .unwrap();

                let total_boards = leaderboard.total_boards.get_mut();
                *total_boards += 1;

                let participant = leaderboard.score.get(&player).await.unwrap();
                match participant {
                    Some(_) => (),
                    None => {
                        let total_players = leaderboard.total_players.get_mut();
                        *total_players += 1;
                        leaderboard.score.insert(&player, 0).unwrap();
                        leaderboard
                            .board_ids
                            .insert(&player, board_id.clone())
                            .unwrap();
                    }
                }

                self.ping_player(&player).await;
            }
            Operation::EndBoard { board_id } => {
                let board = self.state.boards.load_entry_mut(&board_id).await.unwrap();
                if *board.is_ended.get() {
                    panic!("Game is already ended");
                }
                board.is_ended.set(true);

                let player = board.player.get().clone();
                self.ping_player(&player).await;
            }
            Operation::MakeMove {
                board_id,
                direction,
                player,
                timestamp,
            } => {
                let board = self.state.boards.load_entry_mut(&board_id).await.unwrap();

                let is_ended = board.is_ended.get();
                if !is_ended {
                    let mut game = Game {
                        board: *board.board.get(),
                        board_id: board_id.clone(),
                        username: player.clone(),
                        timestamp,
                    };

                    let leaderboard_id = board.leaderboard_id.get();
                    let leaderboard = self
                        .state
                        .leaderboards
                        .load_entry_mut(&leaderboard_id.to_string())
                        .await
                        .unwrap();

                    if !leaderboard_id.is_empty() {
                        let start_time = leaderboard.start_time.get();
                        let end_time = leaderboard.end_time.get();
                        // TODO: need to implement the better check
                        if timestamp < *start_time || timestamp > *end_time {
                            panic!("Tournament is not active");
                        }
                    }

                    let new_board = Game::execute(&mut game, direction);
                    let score = Game::score(new_board);
                    let player = board.player.get().clone();

                    if *board.board.get() == new_board {
                        panic!("No move");
                    }

                    board.board.set(new_board);
                    board.score.set(score);

                    if !board_id.contains("-") {
                        let is_ended = Game::is_ended(new_board);
                        if is_ended {
                            board.is_ended.set(true);
                        }

                        let username = board.player.get();
                        let player_leaderboard_score =
                            leaderboard.score.get(username.as_str()).await.unwrap();

                        if player_leaderboard_score.is_none()
                            || player_leaderboard_score < Some(score)
                        {
                            leaderboard.score.insert(&username.clone(), score).unwrap();
                            leaderboard
                                .board_ids
                                .insert(&username.clone(), board_id)
                                .unwrap();
                        }
                    } else {
                        let (game_id, round, player) =
                            self.parse_elimination_game_id(&board_id).await.unwrap();

                        if !game_id.is_empty() && round != 0 && !player.is_empty() {
                            let elimination_game = self
                                .state
                                .elimination_games
                                .load_entry_mut(&game_id)
                                .await
                                .unwrap();

                            let leaderboard = elimination_game
                                .round_leaderboard
                                .load_entry_mut(&round)
                                .await
                                .unwrap();
                            leaderboard.players.insert(&player, score).unwrap();

                            self.ping_game(&game_id).await;
                        }
                    }

                    self.ping_player(&player).await;
                } else {
                    panic!("Game is ended");
                }
            }
            Operation::CreateEliminationGame { player, settings } => {
                self.check_player_registered(&player, RegistrationCheck::EnsureRegistered)
                    .await;

                if settings.total_round < 1 {
                    panic!("Total round must be greater than 0");
                }
                if settings.max_players < 2 {
                    panic!("Max players must be greater than 1");
                }
                if settings.eliminated_per_trigger < 1 {
                    panic!("Eliminated per trigger must be greater than 0");
                }
                if settings.trigger_interval_seconds < 1 {
                    panic!("Trigger interval must be greater than 0 seconds");
                }

                let chain_ownership = self.runtime.chain_ownership();
                let app_id = self.runtime.application_id().forget_abi();
                let application_permissions = ApplicationPermissions::new_single(app_id);
                let amount = Amount::from_tokens(0);
                let (_, chain_id) =
                    self.runtime
                        .open_chain(chain_ownership, application_permissions, amount);
                let game_id = chain_id.to_string();

                let elimination_game = self
                    .state
                    .elimination_games
                    .load_entry_mut(&game_id)
                    .await
                    .unwrap();
                let created_time = settings.created_time.parse::<u64>().unwrap();
                self.state.waiting_rooms.insert(&game_id, true).unwrap();

                elimination_game.game_id.set(game_id);
                elimination_game.chain_id.set(chain_id.to_string());
                elimination_game.game_name.set(settings.game_name);
                elimination_game.players.set(vec![player.clone()]);
                elimination_game.host.set(player.clone());
                elimination_game.status.set(EliminationGameStatus::Waiting);
                elimination_game.total_rounds.set(settings.total_round);
                elimination_game.current_round.set(0);
                elimination_game.max_players.set(settings.max_players);
                elimination_game
                    .eliminated_per_trigger
                    .set(settings.eliminated_per_trigger);
                elimination_game
                    .trigger_interval_seconds
                    .set(settings.trigger_interval_seconds);
                elimination_game.created_time.set(created_time);
                elimination_game.last_updated_time.set(created_time);

                self.ping_player(&player).await;
            }
            Operation::EliminationGameAction {
                game_id,
                action,
                player,
                timestamp,
            } => {
                self.check_player_registered(&player, RegistrationCheck::EnsureRegistered)
                    .await;

                let elimination_game = self
                    .state
                    .elimination_games
                    .load_entry_mut(&game_id)
                    .await
                    .unwrap();

                match action {
                    MultiplayerGameAction::Start => {
                        if elimination_game.status.get() != &EliminationGameStatus::Waiting {
                            panic!("Game is not in waiting state");
                        }
                        if elimination_game.host.get() != &player {
                            panic!("Only host can start the game");
                        }
                        self.state.waiting_rooms.remove(&game_id).unwrap();
                        elimination_game.status.set(EliminationGameStatus::Active);
                        elimination_game.current_round.set(1);
                        elimination_game.last_updated_time.set(timestamp);

                        let players = elimination_game.players.get();
                        let round_leaderboard = elimination_game
                            .round_leaderboard
                            .load_entry_mut(&1)
                            .await
                            .unwrap();
                        for player in players {
                            let player_chain_id = self
                                .state
                                .players
                                .load_entry_or_insert(player)
                                .await
                                .unwrap()
                                .chain_id
                                .get();
                            let board_id = format!("{}-{}-{}", game_id, 1, player);
                            let game = self.state.boards.load_entry_mut(&board_id).await.unwrap();
                            let new_board = Game::new(&board_id, player, timestamp).board;

                            game.board_id.set(board_id);
                            game.board.set(new_board);
                            game.player.set(player.clone());
                            game.chain_id.set(player_chain_id.clone());
                            round_leaderboard.players.insert(player, 0).unwrap();
                            elimination_game.game_leaderboard.insert(player, 0).unwrap();
                        }
                    }
                    MultiplayerGameAction::End => {
                        if elimination_game.status.get() == &EliminationGameStatus::Ended {
                            panic!("Game is already ended");
                        }
                        if elimination_game.host.get() != &player {
                            panic!("Only host can end the game");
                        }
                        elimination_game.status.set(EliminationGameStatus::Ended);
                        elimination_game.last_updated_time.set(timestamp);

                        // Remove the game from the waiting_rooms list
                        self.state.waiting_rooms.remove(&game_id).unwrap();

                        self.close_chain(&game_id).await;
                    }
                    MultiplayerGameAction::Join => {
                        // Check if game hasn't started yet
                        if elimination_game.status.get() != &EliminationGameStatus::Waiting {
                            panic!("You cannot join the game after it started");
                        }

                        let players = elimination_game.players.get_mut();

                        // Check if player is already in the game
                        if players.contains(&player) {
                            panic!("Player is already in the game");
                        }

                        if self
                            .state
                            .players
                            .load_entry_or_insert(&player)
                            .await
                            .unwrap()
                            .username
                            .get()
                            == ""
                        {
                            panic!("Player is not registered");
                        }

                        // Check if game is not full
                        if players.len() >= *elimination_game.max_players.get() as usize {
                            panic!("Game is full");
                        }

                        players.append(&mut vec![player.clone()]);
                        elimination_game.last_updated_time.set(timestamp);

                        self.ping_player(&player).await;
                    }
                    MultiplayerGameAction::Leave => {
                        // Check if game is in waiting state
                        if elimination_game.status.get() != &EliminationGameStatus::Waiting {
                            panic!("Can only leave game in waiting state");
                        }

                        if elimination_game.host.get() == &player {
                            panic!("Host cannot leave the game");
                        }

                        let players = elimination_game.players.get_mut();
                        if !players.contains(&player) {
                            panic!("Player is not in the game");
                        }

                        players.retain(|p| p != &player);
                        elimination_game.last_updated_time.set(timestamp);
                    }
                    MultiplayerGameAction::NextRound => {
                        if elimination_game.status.get() != &EliminationGameStatus::Active {
                            panic!("Game is not in active state");
                        }

                        if elimination_game.host.get() != &player
                            && elimination_game.last_updated_time.get() + 5000 > timestamp
                        {
                            panic!("Only host can early start next round");
                        }

                        let current_round = elimination_game.current_round.get_mut();
                        let leaderboard = elimination_game
                            .round_leaderboard
                            .load_entry_mut(current_round)
                            .await
                            .unwrap();

                        let mut is_round_ended = true;
                        leaderboard
                            .players
                            .for_each_index(|_key| {
                                is_round_ended = false;
                                Ok(())
                            })
                            .await
                            .unwrap();

                        if is_round_ended {
                            let mut players: Vec<String> = Vec::new();
                            leaderboard
                                .eliminated_players
                                .for_each_index_value(|username, _score| {
                                    players.push(username);
                                    Ok(())
                                })
                                .await
                                .unwrap();
                            let total_round = elimination_game.total_rounds.get();

                            // Update round scores to game leaderboard
                            let mut player_round_scores = std::collections::HashMap::new();
                            leaderboard
                                .eliminated_players
                                .for_each_index_value(|username, score| {
                                    player_round_scores.insert(username, score);
                                    Ok(())
                                })
                                .await
                                .unwrap();

                            // Add round scores to total scores
                            for player in player_round_scores {
                                let prev_score = elimination_game
                                    .game_leaderboard
                                    .get(&player.0)
                                    .await
                                    .unwrap();
                                elimination_game
                                    .game_leaderboard
                                    .insert(&player.0, player.1 + prev_score.unwrap_or(0))
                                    .unwrap();
                            }

                            if *current_round < *total_round {
                                *current_round += 1;
                                elimination_game.last_updated_time.set(timestamp);

                                // Initialize new round leaderboard
                                let new_round_leaderboard = elimination_game
                                    .round_leaderboard
                                    .load_entry_mut(current_round)
                                    .await
                                    .unwrap();

                                // Create boards for next round
                                for player in players {
                                    let player_chain_id = self
                                        .state
                                        .players
                                        .load_entry_or_insert(&player)
                                        .await
                                        .unwrap()
                                        .chain_id
                                        .get();
                                    let board_id =
                                        format!("{}-{}-{}", game_id, current_round, player);
                                    let game =
                                        self.state.boards.load_entry_mut(&board_id).await.unwrap();
                                    let new_board = Game::new(&board_id, &player, timestamp).board;

                                    game.board_id.set(board_id);
                                    game.board.set(new_board);
                                    game.player.set(player.clone());
                                    game.chain_id.set(player_chain_id.clone());
                                    new_round_leaderboard.players.insert(&player, 0).unwrap();
                                }
                            } else {
                                elimination_game.status.set(EliminationGameStatus::Ended);
                                elimination_game.last_updated_time.set(timestamp);
                            }
                        } else {
                            panic!("Round is not ended");
                        }

                        // Update timestamp
                        elimination_game.last_updated_time.set(timestamp);
                    }
                    MultiplayerGameAction::Trigger => {
                        let last_updated = elimination_game.last_updated_time.get();
                        let trigger_interval = elimination_game.trigger_interval_seconds.get();

                        if timestamp >= last_updated + (*trigger_interval as u64) * 1000 {
                            elimination_game.last_updated_time.set(timestamp);

                            // Get current leaderboard
                            let current_round = elimination_game.current_round.get();
                            let leaderboard = elimination_game
                                .round_leaderboard
                                .load_entry_mut(current_round)
                                .await
                                .unwrap();

                            // Sort players by score and get lowest scoring players
                            let mut player_scores: Vec<(String, u64)> = Vec::new();
                            let mut zero_score_players: u8 = 0;

                            leaderboard
                                .players
                                .for_each_index_value(|username, score| {
                                    player_scores.push((username, score));
                                    if score == 0 {
                                        zero_score_players += 1;
                                    }
                                    Ok(())
                                })
                                .await
                                .unwrap();
                            player_scores.sort_by_key(|k| k.1);

                            // Eliminate lowest scoring players
                            let base_eliminate_count =
                                *elimination_game.eliminated_per_trigger.get() as usize;
                            let eliminate_count =
                                base_eliminate_count + zero_score_players as usize;

                            // If only one more player than elimination count, eliminate all
                            let eliminated_players: Vec<(String, u64)> =
                                if player_scores.len() <= eliminate_count + 1 {
                                    player_scores.clone() // Eliminate all remaining players
                                } else {
                                    // Find the score threshold
                                    let threshold_score = player_scores[eliminate_count - 1].1;

                                    // Take all players with scores less than or equal to the threshold
                                    player_scores
                                        .iter()
                                        .take_while(|(_, score)| *score <= threshold_score)
                                        .cloned()
                                        .collect()
                                };

                            let is_round_ended = eliminated_players.is_empty();

                            // End game for eliminated players
                            for player in eliminated_players {
                                let board_id =
                                    format!("{}-{}-{}", game_id, current_round, player.0);
                                let board =
                                    self.state.boards.load_entry_mut(&board_id).await.unwrap();
                                board.is_ended.set(true);

                                // Move player to eliminated players
                                leaderboard.players.remove(&player.0).unwrap();
                                leaderboard
                                    .eliminated_players
                                    .insert(&player.0, player.1)
                                    .unwrap();
                            }

                            if is_round_ended {
                                if current_round == elimination_game.total_rounds.get() {
                                    elimination_game.status.set(EliminationGameStatus::Ended);
                                    self.close_chain(&game_id).await;
                                } else {
                                    panic!("No player to eliminate");
                                }
                            }
                        } else {
                            panic!("Trigger too early");
                        }
                    }
                }

                self.ping_game(&game_id).await;
            }
            Operation::EventLeaderboardAction {
                leaderboard_id,
                action,
                settings,
                player,
                timestamp,
            } => {
                self.check_player_registered(&player, RegistrationCheck::EnsureRegistered)
                    .await;

                let leaderboard = self
                    .state
                    .leaderboards
                    .load_entry_mut(&leaderboard_id)
                    .await
                    .unwrap();

                let host = leaderboard.host.get();
                if !host.is_empty() && host != &player {
                    panic!("Only host can perform this action");
                }

                match action {
                    EventLeaderboardAction::Create => {
                        let start_time = settings.start_time.parse::<u64>().unwrap();
                        let end_time = settings.end_time.parse::<u64>().unwrap();

                        if start_time >= end_time {
                            panic!("Start time cannot be after end time");
                        } else if timestamp >= end_time {
                            panic!("Timestamp cannot be after planned end time");
                        };

                        let chain_ownership = self.runtime.chain_ownership();
                        let app_id = self.runtime.application_id().forget_abi();
                        let application_permissions = ApplicationPermissions::new_single(app_id);
                        let amount = Amount::from_tokens(0);
                        let (_, chain_id) = self.runtime.open_chain(
                            chain_ownership,
                            application_permissions,
                            amount,
                        );
                        let leaderboard_id = chain_id.to_string();

                        let leaderboard = self
                            .state
                            .leaderboards
                            .load_entry_mut(&leaderboard_id)
                            .await
                            .unwrap();

                        leaderboard.name.set(settings.name);
                        leaderboard
                            .description
                            .set(settings.description.unwrap_or("".to_string()));
                        leaderboard.chain_id.set(chain_id.to_string());
                        leaderboard.leaderboard_id.set(leaderboard_id);
                        leaderboard.host.set(player);
                        leaderboard.start_time.set(start_time);
                        leaderboard.end_time.set(end_time);
                    }
                    EventLeaderboardAction::Update => {
                        let name = settings.name;
                        let description = settings.description.unwrap_or("".to_string());
                        let start_time = settings.start_time.parse::<u64>().unwrap();
                        let end_time = settings.end_time.parse::<u64>().unwrap();

                        if start_time >= end_time {
                            panic!("Start time cannot be after end time");
                        }
                        if timestamp >= end_time {
                            panic!("Timestamp cannot be after planned end time");
                        }

                        if !name.is_empty() {
                            leaderboard.name.set(name);
                        }

                        if !description.is_empty() {
                            leaderboard.description.set(description);
                        }

                        if *leaderboard.start_time.get() != 0 {
                            leaderboard.start_time.set(start_time);
                        }
                        if *leaderboard.end_time.get() != 0 {
                            leaderboard.end_time.set(end_time);
                        }
                    }
                    EventLeaderboardAction::Delete => {
                        let is_mod = self
                            .state
                            .players
                            .load_entry_or_insert(&player)
                            .await
                            .unwrap()
                            .is_mod
                            .get();

                        if *leaderboard.host.get() != player && !is_mod {
                            panic!("Only host or admin can delete event");
                        }
                        if leaderboard.leaderboard_id.get().is_empty() {
                            panic!("Cannot delete the main leaderboard");
                        }

                        self.state
                            .leaderboards
                            .remove_entry(&leaderboard_id)
                            .unwrap();
                        // TODO: this cause error when trying to close empty chain
                        // self.close_chain(&leaderboard_id).await;
                    }
                    EventLeaderboardAction::TogglePin => {
                        let is_mod = self
                            .state
                            .players
                            .load_entry_or_insert(&player)
                            .await
                            .unwrap()
                            .is_mod
                            .get();

                        if !is_mod {
                            panic!("Only admin can pin event");
                        }

                        leaderboard.is_pinned.set(!*leaderboard.is_pinned.get());
                    }
                }
            }
            Operation::ToggleAdmin { username } => {
                self.check_player_registered(&username, RegistrationCheck::EnsureRegistered)
                    .await;

                let player = self.state.players.load_entry_mut(&username).await.unwrap();
                player.is_mod.set(!*player.is_mod.get());
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        match message {
            Message::CloseChain => self.runtime.close_chain().unwrap(),
            Message::Ping => {
                log::info!("Ping received");
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl Game2048Contract {
    async fn ping_player(&mut self, player: &str) {
        let chain_id = self
            .state
            .players
            .load_entry_or_insert(player)
            .await
            .unwrap()
            .chain_id
            .get()
            .clone();
        self.ping_chain(chain_id);
    }

    async fn ping_game(&mut self, game_id: &str) {
        let chain_id = self
            .state
            .elimination_games
            .load_entry_or_insert(game_id)
            .await
            .unwrap()
            .chain_id
            .get()
            .clone();
        self.ping_chain(chain_id);
    }

    fn ping_chain(&mut self, chain_id: String) {
        let chain_id = ChainId::from_str(&chain_id).unwrap();
        self.runtime
            .prepare_message(Message::Ping)
            .send_to(chain_id);
    }

    async fn close_chain(&mut self, game_id: &str) {
        let chain_id = self
            .state
            .elimination_games
            .load_entry_or_insert(game_id)
            .await
            .unwrap()
            .chain_id
            .get()
            .clone();
        let chain_id = ChainId::from_str(&chain_id).unwrap();
        self.runtime
            .prepare_message(Message::CloseChain)
            .send_to(chain_id);
    }

    async fn check_player_registered(&mut self, player: &str, check: RegistrationCheck) {
        let username = self
            .state
            .players
            .load_entry_or_insert(player)
            .await
            .unwrap()
            .username
            .get();

        let is_registered = !username.is_empty();

        match check {
            RegistrationCheck::EnsureRegistered if !is_registered => {
                panic!("Player not registered");
            }
            RegistrationCheck::EnsureNotRegistered if is_registered => {
                panic!("Player already registered");
            }
            _ => {}
        }
    }

    async fn parse_elimination_game_id(&self, board_id: &str) -> Option<(String, u8, String)> {
        let parts: Vec<&str> = board_id.split('-').collect();
        if parts.len() == 3 {
            let game_id = parts[0].to_string();
            if let Ok(round_id) = parts[1].parse::<u8>() {
                let player_id = parts[2].to_string();
                return Some((game_id, round_id, player_id));
            }
        }
        None
    }
}
