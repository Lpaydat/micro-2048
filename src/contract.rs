#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::str::FromStr;

use linera_sdk::{
    base::{Amount, ApplicationPermissions, ChainId, WithContractAbi},
    views::{RootView, View},
    Contract, ContractRuntime,
};
use state::{ClassicLeaderboard, EliminationGameStatus};

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

        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        leaderboard.leaderboard_id.set("".to_string());
        leaderboard
            .chain_id
            .set(self.runtime.chain_id().to_string());
        leaderboard.host.set("".to_string());
        leaderboard.start_time.set(0);
        leaderboard.end_time.set(0);
        leaderboard.name.set("".to_string());
        leaderboard.description.set("".to_string());
        leaderboard.total_boards.set(0);
        leaderboard.total_players.set(0);

        // let boards = include_str!("../db/boards.txt");
        // for line in boards.lines() {
        //     let parts: Vec<&str> = line.split_whitespace().collect();
        //     if parts.len() >= 2 {
        //         let board_id = parts[0];
        //         let board_hex = parts[1];
        //         let score = parts[2];
        //         let is_ended = parts[3];

        //         let game = self.state.boards.load_entry_mut(board_id).await.unwrap();
        //         game.board_id.set(board_id.to_string());
        //         game.board
        //             .set(u64::from_str_radix(board_hex.trim_start_matches("0x"), 16).unwrap());
        //         game.player.set("".to_string());
        //         game.score.set(score.parse::<u64>().unwrap());
        //         game.is_ended.set(is_ended.parse::<bool>().unwrap());
        //     }
        // }

        // let rankers = include_str!("../db/rankers.txt");
        // let leaderboard = self
        //     .state
        //     .leaderboards
        //     .load_entry_mut(&"".to_string())
        //     .await
        //     .unwrap();
        // for line in rankers.lines() {
        //     let parts: Vec<&str> = line.split_whitespace().collect();
        //     if parts.len() >= 3 {
        //         let username = parts[0];
        //         let score = parts[1];
        //         let board_id = parts[2];

        //         leaderboard
        //             .score
        //             .insert(username, score.parse::<u64>().unwrap())
        //             .unwrap();
        //         leaderboard
        //             .board_ids
        //             .insert(username, board_id.to_string())
        //             .unwrap();
        //     }
        // }
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            Operation::RegisterPlayer {
                username,
                password_hash,
            } => {
                if username.trim().is_empty() {
                    panic!("Username cannot be empty");
                }
                let is_main_chain = self.is_main_chain().await;
                if !is_main_chain {
                    panic!("Only main chain can register player");
                }

                self.check_player_registered(&username, RegistrationCheck::EnsureNotRegistered)
                    .await;

                let chain_ownership = self.runtime.chain_ownership();
                let application_permissions = ApplicationPermissions::default();
                let amount = Amount::from_tokens(100_000_000);
                let (_, chain_id) =
                    self.runtime
                        .open_chain(chain_ownership, application_permissions, amount);

                let player = self.state.players.load_entry_mut(&username).await.unwrap();
                player.username.set(username.clone());
                player.password_hash.set(password_hash.clone());
                player.chain_id.set(chain_id.to_string());

                self.request_application(chain_id).await;
                self.register_player(chain_id, &username, &password_hash)
                    .await;
            }
            Operation::NewBoard {
                seed,
                player,
                timestamp,
                leaderboard_id,
            } => {
                self.check_player_registered(&player, RegistrationCheck::EnsureRegistered)
                    .await;

                let player_obj = self
                    .state
                    .players
                    .load_entry_or_insert(&player)
                    .await
                    .unwrap();

                let current_chain_id = self.runtime.chain_id().to_string();
                if current_chain_id != *player_obj.chain_id.get() {
                    panic!("You can only create board on your own chain");
                }

                let mut board_id = hash_seed(&seed, &player, timestamp).to_string();
                board_id = format!("{}.{}", player_obj.chain_id.get(), board_id);

                let leaderboard_id = leaderboard_id.unwrap_or_default();
                let new_board = Game::new(&board_id, &player, timestamp).board;
                let game = self.state.boards.load_entry_mut(&board_id).await.unwrap();
                game.board_id.set(board_id.clone());
                game.board.set(new_board);
                game.player.set(player.clone());
                game.leaderboard_id.set(leaderboard_id.clone());
                game.chain_id.set(player_obj.chain_id.get().to_string());

                let leaderboard_chain_id = if !leaderboard_id.is_empty() {
                    ChainId::from_str(&leaderboard_id).unwrap()
                } else {
                    self.runtime.application_creator_chain_id()
                };
                self.runtime
                    .prepare_message(Message::LeaderboardNewGame {
                        player: player.clone(),
                        board_id: board_id.clone(),
                        timestamp,
                    })
                    .send_to(leaderboard_chain_id);
            }
            Operation::EndBoard { board_id } => {
                let board = self.state.boards.load_entry_mut(&board_id).await.unwrap();
                if *board.is_ended.get() {
                    panic!("Game is already ended");
                }
                board.is_ended.set(true);
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

                    let chain_id = board.leaderboard_id.get().clone();
                    let new_board = Game::execute(&mut game, direction);
                    let score = Game::score(new_board);

                    if *board.board.get() == new_board {
                        panic!("No move");
                    }

                    // Get highest tile values for both old and new boards
                    let old_highest = Game::highest_tile(*board.board.get());
                    let new_highest = Game::highest_tile(new_board);

                    board.board.set(new_board);
                    board.score.set(score);

                    let is_ended = Game::is_ended(new_board);
                    if is_ended {
                        board.is_ended.set(true);
                    }

                    // Update score only if highest tile increased or game ended
                    if new_highest > old_highest || is_ended {
                        let chain_id = if !chain_id.is_empty() {
                            ChainId::from_str(&chain_id).unwrap()
                        } else {
                            self.runtime.application_creator_chain_id()
                        };
                        self.update_score(chain_id, &player, &board_id, score, timestamp)
                            .await;
                    }
                } else {
                    panic!("Game is ended");
                }
            }
            Operation::CreateEliminationGame { player, settings } => {
                let is_main_chain = self.is_main_chain().await;
                if !is_main_chain {
                    panic!("Only main chain can create elimination game");
                }

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
                let amount = Amount::from_tokens(10_000);
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
                elimination_game.game_name.set(settings.game_name.clone());
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

                let p = self.state.players.load_entry_mut(&player).await.unwrap();
                let host_chain_id = p.chain_id.get().clone();

                self.ping_player(&player).await;
                self.request_application(chain_id).await;
                self.runtime
                    .prepare_message(Message::CreateEliminationGame {
                        player: player.clone(),
                        host_chain_id,
                        settings: settings.clone(),
                    })
                    .send_to(chain_id);
            }
            Operation::EliminationGameAction {
                action,
                player,
                requester_chain_id,
                timestamp,
            } => {
                // Every action done to elimination game must be done on game chain
                let elimination_game = self
                    .state
                    .elimination_games
                    .load_entry_mut("")
                    .await
                    .unwrap();
                let game_id = elimination_game.chain_id.get().clone();
                let game_chain_id = self.runtime.chain_id().to_string();
                if game_chain_id != *elimination_game.chain_id.get() {
                    panic!("Action allowed only on game chain");
                }

                match action {
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

                        // Check if game is not full
                        if players.len() >= *elimination_game.max_players.get() as usize {
                            panic!("Game is full");
                        }

                        // register player to chain
                        let p = self.state.players.load_entry_mut(&player).await.unwrap();
                        p.username.set(player.clone());
                        p.chain_id.set(requester_chain_id.to_string());

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

                        self.state.players.remove_entry(&player).unwrap();
                    }
                    MultiplayerGameAction::Start => {
                        if elimination_game.status.get() != &EliminationGameStatus::Waiting {
                            panic!("Game is not in waiting state");
                        }
                        if elimination_game.host.get() != &player {
                            panic!("Only host can start the game");
                        }
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
                            round_leaderboard.players.insert(player, 0).unwrap();
                            elimination_game.game_leaderboard.insert(player, 0).unwrap();
                        }

                        for player in players.clone() {
                            let p = self
                                .state
                                .players
                                .load_entry_or_insert(&player)
                                .await
                                .expect("Player not exists");
                            let player_chain_id = p.chain_id.get().clone();
                            self.create_elimination_board(
                                &game_id,
                                &player_chain_id,
                                1,
                                &player,
                                timestamp,
                            )
                            .await;
                        }

                        self.runtime
                            .prepare_message(Message::UpdateEliminationStatus {
                                game_id: game_id.clone(),
                                status: "Active".to_string(),
                            })
                            .send_to(self.runtime.application_creator_chain_id());
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
                        self.close_elimination_chain(&game_id).await;
                        self.runtime
                            .prepare_message(Message::UpdateEliminationStatus {
                                game_id: game_id.clone(),
                                status: "Ended".to_string(),
                            })
                            .send_to(self.runtime.application_creator_chain_id());
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
                                let new_round = *current_round;
                                elimination_game.last_updated_time.set(timestamp);

                                // Initialize new round leaderboard
                                let new_round_leaderboard = elimination_game
                                    .round_leaderboard
                                    .load_entry_mut(&new_round)
                                    .await
                                    .unwrap();

                                // Create boards for next round
                                for player in players.clone() {
                                    new_round_leaderboard.players.insert(&player, 0).unwrap();
                                }

                                for player in players {
                                    let p = self
                                        .state
                                        .players
                                        .load_entry_or_insert(&player)
                                        .await
                                        .expect("Player not exists");
                                    let player_chain_id = p.chain_id.get().clone();
                                    self.create_elimination_board(
                                        &game_id,
                                        &player_chain_id,
                                        new_round,
                                        &player,
                                        timestamp,
                                    )
                                    .await;
                                }
                            } else {
                                elimination_game.status.set(EliminationGameStatus::Ended);
                                elimination_game.last_updated_time.set(timestamp);
                            }
                        } else {
                            panic!("Round is not ended");
                        }
                    }
                    MultiplayerGameAction::Trigger => {
                        let last_updated = *elimination_game.last_updated_time.get();
                        let trigger_interval = *elimination_game.trigger_interval_seconds.get();

                        if timestamp >= last_updated + (trigger_interval as u64) * 1000 {
                            elimination_game.last_updated_time.set(timestamp);

                            // Get current leaderboard
                            let current_round = *elimination_game.current_round.get();
                            let leaderboard = elimination_game
                                .round_leaderboard
                                .load_entry_mut(&current_round)
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
                            for player in eliminated_players.clone() {
                                // Move player to eliminated players
                                leaderboard.players.remove(&player.0).unwrap();
                                leaderboard
                                    .eliminated_players
                                    .insert(&player.0, player.1)
                                    .unwrap();
                            }

                            if is_round_ended {
                                if current_round == *elimination_game.total_rounds.get() {
                                    elimination_game.status.set(EliminationGameStatus::Ended);
                                    self.close_elimination_chain(&game_id).await;
                                } else {
                                    panic!("No player to eliminate");
                                }
                            }

                            for player in eliminated_players {
                                let p = self
                                    .state
                                    .players
                                    .load_entry_or_insert(&player.0)
                                    .await
                                    .expect("Player not exists");
                                let player_chain_id = p.chain_id.get().clone();
                                self.end_elimination_board(
                                    &game_id,
                                    &player_chain_id,
                                    current_round,
                                    &player.0,
                                )
                                .await;
                            }
                        } else {
                            panic!("Trigger too early");
                        }
                    }
                }
            }
            Operation::EventLeaderboardAction {
                leaderboard_id,
                action,
                settings,
                player,
                timestamp,
            } => {
                let is_main_chain = self.is_main_chain().await;
                if !is_main_chain {
                    panic!("Only main chain can perform event leaderboard action");
                }

                self.check_player_registered(&player, RegistrationCheck::EnsureRegistered)
                    .await;

                let chain_id = if action == EventLeaderboardAction::Create {
                    let chain_ownership = self.runtime.chain_ownership();
                    let app_id = self.runtime.application_id().forget_abi();
                    let application_permissions = ApplicationPermissions::new_single(app_id);
                    let amount = Amount::from_tokens(100_000);
                    let (_, chain_id) =
                        self.runtime
                            .open_chain(chain_ownership, application_permissions, amount);

                    chain_id
                } else if !leaderboard_id.is_empty() {
                    ChainId::from_str(&leaderboard_id).unwrap()
                } else {
                    panic!("Leaderboard ID is required");
                };

                let leaderboard = self
                    .state
                    .leaderboards
                    .load_entry_mut(&chain_id.to_string())
                    .await
                    .unwrap();

                let host = leaderboard.host.get().clone();
                if !host.is_empty() && host != player {
                    panic!("Only host can perform this action");
                }

                match action {
                    EventLeaderboardAction::Create | EventLeaderboardAction::Update => {
                        let start_time = settings.start_time.parse::<u64>().unwrap();
                        let end_time = settings.end_time.parse::<u64>().unwrap();

                        if start_time >= end_time {
                            panic!("Start time cannot be after end time");
                        } else if timestamp >= end_time {
                            panic!("Timestamp cannot be after planned end time");
                        };

                        if !settings.name.is_empty() {
                            leaderboard.name.set(settings.name.clone());
                        }

                        if let Some(desc) = settings.description.clone() {
                            leaderboard.description.set(desc);
                        }

                        if start_time != 0 {
                            leaderboard.start_time.set(start_time);
                        }

                        if end_time != 0 {
                            leaderboard.end_time.set(end_time);
                        }

                        if action == EventLeaderboardAction::Create {
                            let chain_id_str = chain_id.to_string();
                            leaderboard.leaderboard_id.set(chain_id_str.clone());
                            leaderboard.chain_id.set(chain_id_str);
                            leaderboard.host.set(player.clone());

                            self.request_application(chain_id).await;
                        }
                        self.upsert_leaderboard(
                            chain_id,
                            &settings.name,
                            &settings.description.unwrap_or_default(),
                            &player,
                            start_time,
                            end_time,
                        )
                        .await;
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
                        self.close_chain(leaderboard_id).await;
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
                let is_main_chain = self.is_main_chain().await;
                if !is_main_chain {
                    panic!("Only main chain can toggle admin");
                }

                self.check_player_registered(&username, RegistrationCheck::EnsureRegistered)
                    .await;

                let player = self.state.players.load_entry_mut(&username).await.unwrap();
                player.is_mod.set(!*player.is_mod.get());
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        match message {
            Message::CloseChain => {
                self.runtime
                    .close_chain()
                    .expect("The application does not have permission to close the chain");
            }
            Message::Ping => {
                log::info!("Ping received");
            }
            Message::RequestApplication { chain_id } => {
                let target_chain_id = self.runtime.application_creator_chain_id().to_string();
                let creation_height = self.runtime.application_id().creation.height.to_string();
                let creation_height_hex =
                    format!("{:02x}", creation_height.parse::<u64>().unwrap()); // Convert to hex and ensure at least two digits
                let padded_height_hex = format!("{:0<24}", creation_height_hex); // Pad with zeros to make it 24 characters long

                let application_id = format!(
                    "{}{}{}{}",
                    self.runtime.application_id().bytecode_id.contract_blob_hash,
                    self.runtime.application_id().bytecode_id.service_blob_hash,
                    self.runtime.application_id().creation.chain_id,
                    padded_height_hex
                );
                log::info!(
                    "REQUEST_APPLICATION - application_id: {}, requester_chain_id: {}, target_chain_id: {}",
                    application_id,
                    chain_id,
                    target_chain_id
                );
            }
            Message::RegisterPlayer {
                username,
                password_hash,
            } => {
                self.check_player_registered(&username, RegistrationCheck::EnsureNotRegistered)
                    .await;

                let player = self.state.players.load_entry_mut(&username).await.unwrap();
                let chain_id = self.runtime.chain_id().to_string();
                player.username.set(username);
                player.password_hash.set(password_hash);
                player.chain_id.set(chain_id);
            }
            Message::EventLeaderboard {
                leaderboard_id,
                name,
                description,
                chain_id,
                host,
                start_time,
                end_time,
            } => {
                // Use default leaderboard id for every chain
                let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();

                if !name.is_empty() {
                    leaderboard.name.set(name);
                }

                if let Some(desc) = description {
                    leaderboard.description.set(desc);
                }

                if !chain_id.is_empty() {
                    leaderboard.chain_id.set(chain_id.to_string());
                }

                if !leaderboard_id.is_empty() {
                    leaderboard.leaderboard_id.set(leaderboard_id);
                }

                if !host.is_empty() {
                    leaderboard.host.set(host);
                }

                if start_time != 0 {
                    leaderboard.start_time.set(start_time);
                }

                if end_time != 0 {
                    leaderboard.end_time.set(end_time);
                }
            }
            Message::LeaderboardNewGame {
                player,
                board_id,
                timestamp,
            } => {
                let leaderboard = self.is_leaderboard_active(timestamp).await;
                let total_boards = leaderboard.total_boards.get_mut();
                *total_boards += 1;

                let participant = leaderboard.score.get(&player).await.unwrap();
                match participant {
                    Some(_) => (),
                    None => {
                        let total_players = leaderboard.total_players.get_mut();
                        *total_players += 1;
                        leaderboard.score.insert(&player, 0).unwrap();
                        leaderboard.board_ids.insert(&player, board_id).unwrap();
                    }
                }
            }
            Message::UpdateScore {
                player,
                board_id,
                score,
                timestamp,
            } => {
                self.update_leaderboard_score(&player, board_id, score, timestamp)
                    .await;
            }
            Message::CreateEliminationGame {
                player,
                host_chain_id,
                settings,
            } => {
                let elimination_game = self
                    .state
                    .elimination_games
                    .load_entry_mut("")
                    .await
                    .unwrap();
                let created_time = settings.created_time.parse::<u64>().unwrap();
                let chain_id = self.runtime.chain_id().to_string();

                elimination_game.chain_id.set(chain_id);
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

                let p = self.state.players.load_entry_mut(&player).await.unwrap();
                p.username.set(player.clone());
                p.chain_id.set(host_chain_id);
            }
            Message::UpdateEliminationStatus { game_id, status } => {
                let elimination_game = self
                    .state
                    .elimination_games
                    .load_entry_mut("")
                    .await
                    .unwrap();
                let s = match status.as_str() {
                    "Active" => EliminationGameStatus::Active,
                    "Ended" => EliminationGameStatus::Ended,
                    _ => panic!("Invalid elimination game status"),
                };
                elimination_game.status.set(s);
                self.state.waiting_rooms.remove(&game_id).unwrap();
            }
            Message::CreateEliminationBoard {
                game_id,
                round,
                player,
                timestamp,
            } => {
                let p = self
                    .state
                    .players
                    .load_entry_or_insert(&player)
                    .await
                    .expect("Invalid message");
                let player_chain_id = p.chain_id.get().clone();
                let board_id = format!("{}-{}-{}-{}", game_id, player_chain_id, player, round);
                let game = self.state.boards.load_entry_mut(&board_id).await.unwrap();
                let new_board = Game::new(&board_id, &player, timestamp).board;

                game.board_id.set(board_id);
                game.board.set(new_board);
                game.player.set(player.clone());
                game.chain_id.set(player_chain_id.clone());
                game.leaderboard_id.set(game_id.clone());
            }
            Message::EndEliminationBoard {
                game_id,
                round,
                player,
            } => {
                let p = self
                    .state
                    .players
                    .load_entry_or_insert(&player)
                    .await
                    .expect("Invalid message");
                let player_chain_id = p.chain_id.get().clone();
                let board_id = format!("{}-{}-{}-{}", game_id, player_chain_id, player, round);
                let board = self.state.boards.load_entry_mut(&board_id).await.unwrap();
                board.is_ended.set(true);
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl Game2048Contract {
    async fn is_main_chain(&mut self) -> bool {
        self.runtime.chain_id().to_string()
            == self.runtime.application_creator_chain_id().to_string()
    }

    async fn is_leaderboard_active(&mut self, timestamp: u64) -> &mut ClassicLeaderboard {
        let is_main_chain = self.is_main_chain().await;
        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        let start_time = leaderboard.start_time.get();
        let end_time = leaderboard.end_time.get();

        if !is_main_chain && (timestamp < *start_time || timestamp > *end_time) {
            panic!("Leaderboard is not active");
        }

        leaderboard
    }

    // Update the leaderboard score, it will always update the chain's main leaderboard
    async fn update_leaderboard_score(
        &mut self,
        player: &str,
        board_id: String,
        score: u64,
        timestamp: u64,
    ) {
        if !board_id.contains("-") {
            // Classic leaderboard
            let leaderboard = self.is_leaderboard_active(timestamp).await;
            let player_leaderboard_score = leaderboard.score.get(player).await.unwrap();

            if player_leaderboard_score.is_none() || player_leaderboard_score < Some(score) {
                leaderboard.score.insert(player, score).unwrap();
                leaderboard.board_ids.insert(player, board_id).unwrap();
            }
        } else {
            // Elimination leaderboard
            let (game_id, _, player, round) =
                self.parse_elimination_game_id(&board_id).await.unwrap();

            if !game_id.is_empty() && round != 0 && !player.is_empty() {
                let elimination_game = self
                    .state
                    .elimination_games
                    .load_entry_mut("")
                    .await
                    .unwrap();

                let leaderboard = elimination_game
                    .round_leaderboard
                    .load_entry_mut(&round)
                    .await
                    .unwrap();
                leaderboard.players.insert(&player, score).unwrap();
            }
        }
    }

    async fn request_application(&mut self, chain_id: ChainId) {
        self.runtime
            .prepare_message(Message::RequestApplication {
                chain_id: chain_id.to_string(),
            })
            .send_to(chain_id);
    }

    async fn register_player(&mut self, chain_id: ChainId, player: &str, password_hash: &str) {
        self.runtime
            .prepare_message(Message::RegisterPlayer {
                username: player.to_string(),
                password_hash: password_hash.to_string(),
            })
            .send_to(chain_id);
    }

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

    fn ping_chain(&mut self, chain_id: String) {
        let chain_id = ChainId::from_str(&chain_id).unwrap();
        self.runtime
            .prepare_message(Message::Ping)
            .send_to(chain_id);
    }

    async fn upsert_leaderboard(
        &mut self,
        chain_id: ChainId,
        name: &str,
        description: &str,
        host: &str,
        start_time: u64,
        end_time: u64,
    ) {
        self.runtime
            .prepare_message(Message::EventLeaderboard {
                leaderboard_id: chain_id.to_string(),
                name: name.to_string(),
                description: Some(description.to_string()),
                chain_id: chain_id.to_string(),
                host: host.to_string(),
                start_time,
                end_time,
            })
            .send_to(chain_id);
    }

    async fn update_score(
        &mut self,
        chain_id: ChainId,
        player: &str,
        board_id: &str,
        score: u64,
        timestamp: u64,
    ) {
        self.runtime
            .prepare_message(Message::UpdateScore {
                player: player.to_string(),
                board_id: board_id.to_string(),
                score,
                timestamp,
            })
            .send_to(chain_id);
    }

    async fn close_elimination_chain(&mut self, game_id: &str) {
        let chain_id = self
            .state
            .elimination_games
            .load_entry_or_insert(game_id)
            .await
            .unwrap()
            .chain_id
            .get()
            .clone();
        self.close_chain(chain_id).await;
    }

    async fn close_chain(&mut self, chain_id: String) {
        let chain_id = ChainId::from_str(&chain_id).unwrap();
        self.runtime
            .prepare_message(Message::CloseChain)
            .send_to(chain_id);
    }

    async fn check_player_registered(
        &mut self,
        player_username: &str,
        check: RegistrationCheck,
    ) -> String {
        let player = self
            .state
            .players
            .load_entry_or_insert(player_username)
            .await
            .unwrap();
        let username = player.username.get();

        let is_registered = !username.trim().is_empty();

        match check {
            RegistrationCheck::EnsureRegistered if !is_registered => {
                panic!("Player not registered");
            }
            RegistrationCheck::EnsureNotRegistered if is_registered => {
                panic!("Player already registered");
            }
            _ => {}
        }

        player.password_hash.get().to_string()
    }

    async fn parse_elimination_game_id(
        &self,
        board_id: &str,
    ) -> Option<(String, String, String, u8)> {
        let parts: Vec<&str> = board_id.split('-').collect();
        if parts.len() == 4 {
            let game_id = parts[0].to_string();
            let player_chain_id = parts[1].to_string();
            let player = parts[2].to_string();
            if let Ok(round_id) = parts[3].parse::<u8>() {
                return Some((game_id, player_chain_id, player, round_id));
            }
        }
        None
    }

    async fn create_elimination_board(
        &mut self,
        game_id: &str,
        player_chain_id: &str,
        round: u8,
        player: &str,
        timestamp: u64,
    ) {
        let chain_id = ChainId::from_str(player_chain_id).unwrap();
        self.runtime
            .prepare_message(Message::CreateEliminationBoard {
                game_id: game_id.to_string(),
                round,
                player: player.to_string(),
                timestamp,
            })
            .send_to(chain_id);
    }

    async fn end_elimination_board(
        &mut self,
        game_id: &str,
        player_chain_id: &str,
        round: u8,
        player: &str,
    ) {
        let chain_id = ChainId::from_str(player_chain_id).unwrap();
        self.runtime
            .prepare_message(Message::EndEliminationBoard {
                game_id: game_id.to_string(),
                round,
                player: player.to_string(),
            })
            .send_to(chain_id);
    }
}
