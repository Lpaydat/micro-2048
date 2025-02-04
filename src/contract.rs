#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::str::FromStr;

use linera_sdk::{
    base::{Amount, ApplicationPermissions, ChainId, WithContractAbi},
    serde_json,
    views::{RootView, View},
    Contract, ContractRuntime,
};
use state::{Leaderboard, LeaderboardShard};

use self::state::Game2048;
use game2048::{
    hash_seed, Direction, Game, LeaderboardAction, Message, Operation, RegistrationCheck,
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
                let amount = Amount::from_tokens(10_000);
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
                player,
                player_chain_id,
                timestamp,
            } => {
                let nonce = self.state.nonce.get();
                let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
                let leaderboard_id = leaderboard.leaderboard_id.get();

                if leaderboard_id.is_empty() {
                    panic!("No leaderboard found");
                }

                let start_time = *leaderboard.start_time.get();
                let end_time = *leaderboard.end_time.get();

                if timestamp < start_time {
                    panic!("Timestamp cannot be before planned start time");
                }

                if timestamp > end_time {
                    panic!("Timestamp cannot be after planned end time");
                }

                let message_payload = Message::CreateNewBoard {
                    seed: nonce.to_string(),
                    player: player.clone(),
                    timestamp,
                    leaderboard_id: leaderboard_id.clone(),
                    shard_id: self.runtime.chain_id().to_string(), // this will be leaderboard chain_id or shard chain_id
                    end_time,
                };
                self.state.nonce.set(nonce + 1);
                let message = self.runtime.prepare_message(message_payload);
                message.send_to(ChainId::from_str(&player_chain_id).unwrap());
            }
            Operation::NewShard => {
                let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();

                let start_time = *leaderboard.start_time.get();
                let end_time = *leaderboard.end_time.get();

                let chain_ownership = self.runtime.chain_ownership();
                let app_id = self.runtime.application_id().forget_abi();
                let application_permissions = ApplicationPermissions::new_single(app_id);
                let amount = Amount::from_tokens(1_000);
                let (_, shard_id) =
                    self.runtime
                        .open_chain(chain_ownership, application_permissions, amount);

                leaderboard.shard_ids.push_back(shard_id.to_string());
                leaderboard.current_shard_id.set(shard_id.to_string());

                let leaderboard_id = leaderboard.chain_id.get().clone();
                self.request_application(shard_id).await;
                self.upsert_leaderboard(
                    ChainId::from_str(&leaderboard_id).unwrap(),
                    "",
                    "",
                    "",
                    start_time,
                    end_time,
                    Some(shard_id),
                )
                .await;
            }
            Operation::MakeMoves {
                board_id,
                moves,
                player,
            } => {
                let board = self.state.boards.load_entry_mut(&board_id).await.unwrap();
                // let chain_id = board.leaderboard_id.get().clone();
                let shard_id = board.shard_id.get().clone();

                if player != *board.player.get() {
                    panic!("You can only make move on your own board");
                }

                type MoveInput = (Direction, String);
                let moves: Vec<MoveInput> =
                    serde_json::from_str(&moves).unwrap_or_else(|_| panic!("Invalid moves format"));

                let mut is_ended = *board.is_ended.get();
                let end_time = *board.end_time.get();
                if !is_ended && !moves.is_empty() {
                    let initial_board = *board.board.get();
                    let initial_highest_tile = Game::highest_tile(initial_board);
                    let mut current_board = initial_board;
                    let mut any_change = false;
                    let mut latest_timestamp = 0;

                    for (direction, timestamp) in moves {
                        if is_ended {
                            break;
                        }

                        let timestamp = timestamp.parse::<u64>().unwrap();
                        if timestamp > end_time {
                            board.is_ended.set(true);
                            break;
                        }
                        if timestamp < latest_timestamp {
                            panic!("Timestamp must be after latest timestamp");
                        }
                        latest_timestamp = timestamp;

                        let mut game = Game {
                            board: current_board,
                            board_id: board_id.clone(),
                            username: player.clone(),
                            timestamp,
                        };

                        let new_board = Game::execute(&mut game, direction);
                        let new_score = Game::score(new_board);

                        if current_board == new_board {
                            continue;
                        }

                        any_change = true;
                        current_board = new_board;
                        board.board.set(current_board);
                        board.score.set(new_score);

                        is_ended = Game::is_ended(current_board);
                        if is_ended {
                            board.is_ended.set(true);
                            break;
                        }
                    }

                    if !any_change {
                        panic!("No valid moves in the sequence");
                    }

                    let final_score = *board.score.get();
                    let final_highest_tile = Game::highest_tile(current_board);

                    let player_record = self
                        .state
                        .player_records
                        .load_entry_mut(&player)
                        .await
                        .unwrap();
                    let prev_score = player_record
                        .best_score
                        .get(&shard_id)
                        .await
                        .unwrap()
                        .unwrap_or(0);

                    let score_threshold = prev_score + 1000;
                    if final_score > score_threshold
                        || final_highest_tile > initial_highest_tile
                        || is_ended
                    {
                        player_record
                            .best_score
                            .insert(&shard_id, final_score)
                            .unwrap();
                        let shard_id = ChainId::from_str(&shard_id).unwrap();
                        self.update_score(
                            shard_id,
                            &player,
                            &board_id,
                            final_score,
                            is_ended,
                            latest_timestamp,
                        )
                        .await;
                    }
                } else if moves.is_empty() {
                    let score = Game::score(*board.board.get());
                    if shard_id.is_empty() {
                        panic!("Chain id is empty");
                    }
                    let shard_id = ChainId::from_str(&shard_id).unwrap();
                    self.update_score(shard_id, &player, &board_id, score, true, 111970)
                        .await;
                } else {
                    panic!("Game is ended");
                }
            }
            Operation::LeaderboardAction {
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

                let chain_id = if action == LeaderboardAction::Create {
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

                let is_mod = self
                    .state
                    .players
                    .load_entry_or_insert(&player)
                    .await
                    .unwrap()
                    .is_mod
                    .get();

                let host = leaderboard.host.get().clone();
                if !host.is_empty() && host != player && !is_mod {
                    panic!("Unauthorized: Only the host or moderator can perform this action on the leaderboard");
                }

                match action {
                    LeaderboardAction::Create | LeaderboardAction::Update => {
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

                        if action == LeaderboardAction::Create {
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
                            None,
                        )
                        .await;
                    }
                    LeaderboardAction::Delete => {
                        if leaderboard.leaderboard_id.get().is_empty() {
                            panic!("Cannot delete the main leaderboard");
                        }

                        self.state
                            .leaderboards
                            .remove_entry(&leaderboard_id)
                            .unwrap();
                        self.close_chain(leaderboard_id).await;
                    }
                    LeaderboardAction::TogglePin => {
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
                    format!("{:02x}", creation_height.parse::<u64>().unwrap());
                let padded_height_hex = format!("{:0<24}", creation_height_hex);

                let application_id = format!(
                    "{}{}{}{}",
                    self.runtime.application_id().bytecode_id.contract_blob_hash,
                    self.runtime.application_id().bytecode_id.service_blob_hash,
                    self.runtime.application_id().creation.chain_id,
                    padded_height_hex
                );

                // IMPORTANT
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
            Message::CreateNewBoard {
                seed,
                player,
                timestamp,
                leaderboard_id,
                shard_id,
                end_time,
            } => {
                self.check_player_registered(&player, RegistrationCheck::EnsureRegistered)
                    .await;

                let player_obj = self.state.players.load_entry_mut(&player).await.unwrap();

                let current_chain_id = self.runtime.chain_id().to_string();
                if current_chain_id != *player_obj.chain_id.get() {
                    panic!("You can only create board on your own chain");
                }

                let mut board_id = hash_seed(&seed, &player, timestamp).to_string();
                board_id = format!("{}.{}", player_obj.chain_id.get(), board_id);

                let new_board = Game::new(&board_id, &player, timestamp).board;
                let game = self.state.boards.load_entry_mut(&board_id).await.unwrap();
                game.board_id.set(board_id.clone());
                game.board.set(new_board);
                game.player.set(player.clone());
                game.leaderboard_id.set(leaderboard_id.clone());
                game.shard_id.set(shard_id.clone());
                game.chain_id.set(player_obj.chain_id.get().to_string());
                game.end_time.set(end_time);
                game.created_at.set(timestamp);

                self.state.latest_board_id.set(board_id.clone());

                // increment player and board count
                let leaderboard_chain_id = ChainId::from_str(&leaderboard_id).unwrap();
                self.runtime
                    .prepare_message(Message::LeaderboardNewGame {
                        player: player.clone(),
                        board_id: board_id.clone(),
                        timestamp,
                    })
                    .send_to(leaderboard_chain_id);
            }
            Message::CreateLeaderboard {
                leaderboard_id,
                name,
                description,
                chain_id,
                host,
                start_time,
                end_time,
            } => {
                let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
                let shard = self.state.shards.load_entry_mut("").await.unwrap();

                if !name.is_empty() {
                    leaderboard.name.set(name.clone());
                }

                if let Some(desc) = description {
                    leaderboard.description.set(desc);
                }

                if !chain_id.is_empty() {
                    leaderboard.chain_id.set(chain_id.to_string());
                    shard.chain_id.set(chain_id.to_string());
                }

                if !leaderboard_id.is_empty() {
                    leaderboard.leaderboard_id.set(leaderboard_id.clone());
                    shard.leaderboard_id.set(leaderboard_id.clone());
                }

                if !host.is_empty() {
                    leaderboard.host.set(host.clone());
                }

                if start_time != 0 {
                    leaderboard.start_time.set(start_time);
                    shard.start_time.set(start_time);
                }

                if end_time != 0 {
                    leaderboard.end_time.set(end_time);
                    shard.end_time.set(end_time);
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
                is_end,
                timestamp,
            } => {
                self.update_shard_score(&player, board_id, score, timestamp)
                    .await;

                let shard = self.state.shards.load_entry_mut("").await.unwrap();
                let count = *shard.counter.get();

                let mut len = 0;
                shard
                    .board_ids
                    .for_each_index(|_| {
                        len += 1;
                        Ok(())
                    })
                    .await
                    .unwrap();

                // Check flush condition (game ended or shard size threshold)
                if is_end || count >= len * 10 {
                    let shard = self.state.shards.load_entry_mut("").await.unwrap();
                    let leaderboard_id = shard.leaderboard_id.get().clone();

                    // Collect all scores and board IDs from shard
                    let mut scores = std::collections::HashMap::new();
                    let mut board_ids = std::collections::HashMap::new();

                    shard
                        .score
                        .for_each_index_value(|player, score| {
                            scores.insert(player.clone(), score);
                            Ok(())
                        })
                        .await
                        .unwrap();
                    shard
                        .board_ids
                        .for_each_index_value(|player, board_id| {
                            board_ids.insert(player.clone(), board_id.clone());
                            Ok(())
                        })
                        .await
                        .unwrap();

                    // Send flush to main leaderboard chain
                    if !leaderboard_id.is_empty() {
                        shard.board_ids.clear();
                        shard.score.clear();
                        shard.counter.set(0);

                        let main_chain_id = ChainId::from_str(&leaderboard_id).unwrap();
                        self.runtime
                            .prepare_message(Message::Flush { board_ids, scores })
                            .send_to(main_chain_id);
                    }
                }
            }
            Message::Flush { board_ids, scores } => {
                let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();

                let mut entries = Vec::new();
                for (player, score) in scores.iter() {
                    if let Some(board_id) = board_ids.get(player) {
                        if let Some(current_score) =
                            leaderboard.score.get(player).await.unwrap_or_default()
                        {
                            if *score > current_score {
                                entries.push((player.clone(), *score, board_id.clone()));
                            } else if let Some(current_board_id) =
                                leaderboard.board_ids.get(player).await.unwrap_or_default()
                            {
                                entries.push((
                                    player.clone(),
                                    current_score,
                                    current_board_id.clone(),
                                ));
                            }
                        }
                    }
                }

                // 2. Validate sorting logic (descending score, ascending name for ties)
                entries.sort_unstable_by(|a, b| {
                    b.1.cmp(&a.1) // Primary sort: highest score first
                        .then_with(|| a.0.cmp(&b.0)) // Secondary sort: alphabetical for ties
                });

                // 3. Strict truncation to top 200
                entries.truncate(200);

                // 4. Atomic update of leaderboard state
                leaderboard.score.clear();
                leaderboard.board_ids.clear();

                // 5. Bulk insert of validated top entries
                for (player, score, board_id) in entries {
                    leaderboard.score.insert(&player, score).unwrap();
                    leaderboard.board_ids.insert(&player, board_id).unwrap();
                }
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

    async fn is_leaderboard_active(&mut self, timestamp: u64) -> &mut Leaderboard {
        let is_main_chain = self.is_main_chain().await;
        let leaderboard = self.state.leaderboards.load_entry_mut("").await.unwrap();
        let start_time = leaderboard.start_time.get();
        let end_time = leaderboard.end_time.get();

        if !is_main_chain
            && timestamp != 111970
            && (timestamp < *start_time || timestamp > *end_time)
        {
            panic!("Leaderboard is not active");
        }

        leaderboard
    }

    async fn is_shard_active(&mut self, timestamp: u64) -> &mut LeaderboardShard {
        let is_main_chain = self.is_main_chain().await;
        let shard = self.state.shards.load_entry_mut("").await.unwrap();
        let start_time = shard.start_time.get();
        let end_time = shard.end_time.get();

        if !is_main_chain
            && timestamp != 111970
            && (timestamp < *start_time || timestamp > *end_time)
        {
            panic!("Leaderboard is not active");
        }

        shard
    }

    async fn update_shard_score(
        &mut self,
        player: &str,
        board_id: String,
        score: u64,
        timestamp: u64,
    ) {
        let shard = self.is_shard_active(timestamp).await;
        let player_shard_score = shard.score.get(player).await.unwrap();

        if player_shard_score.is_none() || player_shard_score < Some(score) {
            shard.score.insert(player, score).unwrap();
            shard.board_ids.insert(player, board_id).unwrap();
            shard.counter.set(*shard.counter.get() + 1);
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

    async fn upsert_leaderboard(
        &mut self,
        chain_id: ChainId,
        name: &str,
        description: &str,
        host: &str,
        start_time: u64,
        end_time: u64,
        send_to: Option<ChainId>,
    ) {
        self.runtime
            .prepare_message(Message::CreateLeaderboard {
                leaderboard_id: chain_id.to_string(),
                name: name.to_string(),
                description: Some(description.to_string()),
                chain_id: chain_id.to_string(),
                host: host.to_string(),
                start_time,
                end_time,
            })
            .send_to(send_to.unwrap_or(chain_id));
    }

    async fn update_score(
        &mut self,
        chain_id: ChainId,
        player: &str,
        board_id: &str,
        score: u64,
        is_end: bool,
        timestamp: u64,
    ) {
        self.runtime
            .prepare_message(Message::UpdateScore {
                player: player.to_string(),
                board_id: board_id.to_string(),
                score,
                is_end,
                timestamp,
            })
            .send_to(chain_id);
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
}
