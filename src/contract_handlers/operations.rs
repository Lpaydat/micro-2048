use std::str::FromStr;
use linera_sdk::{
    linera_base_types::{Account, AccountOwner, Amount, ApplicationPermissions, ChainId, Timestamp},
};
use game2048::{Direction, Game, LeaderboardAction, Message, RegistrationCheck};
use crate::contract_handlers::game_logic::{GameMoveProcessor, GameMoveResult};

pub struct OperationHandler;

impl OperationHandler {
    pub async fn handle_make_moves(
        contract: &mut crate::Game2048Contract,
        board_id: String,
        moves: String,
        player: String,
        password_hash: String,
    ) {
        // Validate password
        contract.validate_player_password(&player, &password_hash).await;
        let board = contract.state.boards.load_entry_mut(&board_id).await.unwrap();
        let shard_id = board.shard_id.get().clone();

        if player != *board.player.get() {
            panic!("You can only make move on your own board");
        }

        type MoveInput = (Direction, String);
        let moves: Vec<MoveInput> =
            serde_json::from_str(&moves).unwrap_or_else(|_| panic!("Invalid moves format"));

        let is_ended = *board.is_ended.get();
        let end_time = *board.end_time.get();
        
        if !is_ended && !moves.is_empty() {
            let initial_board = *board.board.get();
            
            // Convert string timestamps to u64
            let moves_u64: Vec<(Direction, u64)> = moves
                .into_iter()
                .map(|(dir, ts)| (dir, ts.parse::<u64>().unwrap()))
                .collect();

            match GameMoveProcessor::process_moves(&board_id, &player, &moves_u64, initial_board, end_time) {
                GameMoveResult::Success {
                    final_board,
                    final_score,
                    final_highest_tile,
                    initial_highest_tile,
                    is_ended,
                    latest_timestamp,
                } => {
                    // Update board state
                    board.board.set(final_board);
                    board.score.set(final_score);
                    if is_ended {
                        board.is_ended.set(true);
                    }

                    // Update player record if score improvement is significant
                    let player_record = contract
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
                        contract.update_score(
                            shard_id,
                            &player,
                            &board_id,
                            final_score,
                            is_ended,
                            latest_timestamp,
                        );
                    }
                }
                GameMoveResult::Error(msg) => panic!("{}", msg),
            }
        } else if moves.is_empty() {
            let score = Game::score(*board.board.get());
            if shard_id.is_empty() {
                panic!("Chain id is empty");
            }
            let shard_id = ChainId::from_str(&shard_id).unwrap();
            contract.update_score(shard_id, &player, &board_id, score, true, 111970);
        } else {
            panic!("Game is ended");
        }
    }

    pub async fn handle_leaderboard_action(
        contract: &mut crate::Game2048Contract,
        leaderboard_id: String,
        action: LeaderboardAction,
        settings: game2048::LeaderboardSettings,
        player: String,
        password_hash: String,
    ) {
        // Validate password
        contract.validate_player_password(&player, &password_hash).await;
        let is_main_chain = contract.is_main_chain();
        if !is_main_chain {
            panic!("Only main chain can perform event leaderboard action");
        }

        contract.check_player_registered(&player, RegistrationCheck::EnsureRegistered)
            .await;

        let is_mod = contract
            .state
            .players
            .load_entry_or_insert(&player)
            .await
            .unwrap()
            .is_mod
            .get();

        let chain_id = if action == LeaderboardAction::Create {
            let chain_ownership = contract.runtime.chain_ownership();
            let app_id = contract.runtime.application_id().forget_abi();
            let application_permissions = ApplicationPermissions::new_single(app_id);
            let amount = Amount::from_tokens(if *is_mod { 17 } else { 1 });
            let chain_id = contract.runtime.open_chain(chain_ownership, application_permissions, amount);
            chain_id
        } else if !leaderboard_id.is_empty() {
            ChainId::from_str(&leaderboard_id).unwrap()
        } else {
            panic!("Leaderboard ID is required");
        };

        let leaderboard = contract
            .state
            .leaderboards
            .load_entry_mut(&chain_id.to_string())
            .await
            .unwrap();

        let host = leaderboard.host.get().clone();
        if !host.is_empty() && host != player && !is_mod {
            panic!("Unauthorized: Only the host or moderator can perform this action on the leaderboard");
        }

        match action {
            LeaderboardAction::Create | LeaderboardAction::Update => {
                let start_time = settings.start_time.parse::<u64>().unwrap();
                let end_time = settings.end_time.parse::<u64>().unwrap();

                let current_time = contract.runtime.system_time();
                let end_timestamp = Timestamp::from(end_time);
                if start_time >= end_time {
                    panic!("Start time cannot be after end time");
                } else if current_time >= end_timestamp {
                    panic!("Current time cannot be after planned end time");
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
                }
                
                contract.upsert_leaderboard(
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

                contract.state
                    .leaderboards
                    .remove_entry(&leaderboard_id)
                    .unwrap();
            }
            LeaderboardAction::TogglePin => {
                if !is_mod {
                    panic!("Only admin can pin event");
                }

                leaderboard.is_pinned.set(!*leaderboard.is_pinned.get());
            }
        }
    }

    pub fn handle_faucet(contract: &mut crate::Game2048Contract) {
        let current_balance = contract.runtime.chain_balance();

        if current_balance.saturating_mul(10) > Amount::from_tokens(2) {
            panic!("Faucet is not available");
        }

        let app_chain_id = contract.runtime.application_creator_chain_id();
        let chain_id = contract.runtime.chain_id();

        contract.runtime
            .prepare_message(Message::Transfer {
                chain_id,
                amount: Amount::from_tokens(1),
            })
            .send_to(app_chain_id);
    }

    pub async fn handle_register_player(
        contract: &mut crate::Game2048Contract,
        username: String,
        password_hash: String,
    ) {
        if username.trim().is_empty() {
            panic!("Username cannot be empty");
        }
        let is_main_chain = contract.is_main_chain();
        if !is_main_chain {
            panic!("Only main chain can register player");
        }

        contract.check_player_registered(&username, RegistrationCheck::EnsureNotRegistered)
            .await;

        let chain_ownership = contract.runtime.chain_ownership();
        let application_permissions = ApplicationPermissions::default();
        let amount = Amount::from_tokens(1);
        let chain_id = contract.runtime.open_chain(chain_ownership, application_permissions, amount);

        let player = contract.state.players.load_entry_mut(&username).await.unwrap();
        player.username.set(username.clone());
        player.password_hash.set(password_hash.clone());
        player.chain_id.set(chain_id.to_string());

        contract.register_player(chain_id, &username, &password_hash);
    }

    pub async fn handle_new_board(
        contract: &mut crate::Game2048Contract,
        player: String,
        player_chain_id: String,
        timestamp: u64,
        password_hash: String,
    ) {
        // Validate password
        contract.validate_player_password(&player, &password_hash).await;
        let nonce = contract.state.nonce.get();
        let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
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
            shard_id: contract.runtime.chain_id().to_string(),
            end_time,
        };
        contract.state.nonce.set(nonce + 1);
        let message = contract.runtime.prepare_message(message_payload);
        message.send_to(ChainId::from_str(&player_chain_id).unwrap());

        contract.auto_faucet(Some(1));
    }

    pub async fn handle_new_shard(contract: &mut crate::Game2048Contract) {
        let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();

        let start_time = *leaderboard.start_time.get();
        let end_time = *leaderboard.end_time.get();

        let chain_ownership = contract.runtime.chain_ownership();
        let app_id = contract.runtime.application_id().forget_abi();
        let application_permissions = ApplicationPermissions::new_single(app_id);
        let amount = Amount::from_tokens(1);
        let shard_id = contract.runtime.open_chain(chain_ownership, application_permissions, amount);

        leaderboard.shard_ids.push_back(shard_id.to_string());
        leaderboard.current_shard_id.set(shard_id.to_string());

        let leaderboard_id = leaderboard.chain_id.get().clone();
        contract.upsert_leaderboard(
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

    pub async fn handle_toggle_admin(contract: &mut crate::Game2048Contract, username: String, player: String, password_hash: String) {
        // Validate password
        contract.validate_player_password(&player, &password_hash).await;
        
        // Additional admin check
        if player != "lpaydat" {
            panic!("Only lpaydat can toggle admin");
        }
        let is_main_chain = contract.is_main_chain();
        if !is_main_chain {
            panic!("Only main chain can toggle admin");
        }

        contract.check_player_registered(&username, RegistrationCheck::EnsureRegistered)
            .await;

        let player = contract.state.players.load_entry_mut(&username).await.unwrap();
        player.is_mod.set(!*player.is_mod.get());
    }

    pub fn handle_close_chain(contract: &mut crate::Game2048Contract, chain_id: String) {
        let chain_id = ChainId::from_str(&chain_id).unwrap();
        let account = Account {
            chain_id,
            owner: AccountOwner::CHAIN,
        };
        let amount = contract.runtime.chain_balance().saturating_sub(Amount::from_micros(50));
        contract.runtime.transfer(AccountOwner::CHAIN, account, amount);

        contract.runtime
            .close_chain()
            .expect("The application does not have permission to close the chain");
    }
}