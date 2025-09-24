use std::str::FromStr;
use linera_sdk::{
    linera_base_types::{Amount, ChainId},
    views::View,
};
use game2048::{hash_seed, Game, Message, RegistrationCheck};

pub struct MessageHandler;

impl MessageHandler {
    pub fn handle_transfer(contract: &mut crate::Game2048Contract, chain_id: ChainId, amount: Amount) {
        contract.transfer(chain_id, amount);
    }

    pub async fn handle_register_player(
        contract: &mut crate::Game2048Contract,
        username: String,
        password_hash: String,
    ) {
        contract.check_player_registered(&username, RegistrationCheck::EnsureNotRegistered)
            .await;

        let player = contract.state.players.load_entry_mut(&username).await.unwrap();
        let chain_id = contract.runtime.chain_id().to_string();
        player.username.set(username);
        player.password_hash.set(password_hash);
        player.chain_id.set(chain_id);
    }

    pub async fn handle_create_new_board(
        contract: &mut crate::Game2048Contract,
        seed: String,
        player: String,
        timestamp: u64,
        leaderboard_id: String,
        shard_id: String,
        end_time: u64,
    ) {
        contract.check_player_registered(&player, RegistrationCheck::EnsureRegistered)
            .await;

        let player_obj = contract.state.players.load_entry_mut(&player).await.unwrap();

        let current_chain_id = contract.runtime.chain_id().to_string();
        if current_chain_id != *player_obj.chain_id.get() {
            panic!("You can only create board on your own chain");
        }

        let mut board_id = hash_seed(&seed, &player, timestamp).to_string();
        board_id = format!("{}.{}", player_obj.chain_id.get(), board_id);

        let new_board = Game::new(&board_id, &player, timestamp).board;
        let game = contract.state.boards.load_entry_mut(&board_id).await.unwrap();
        game.board_id.set(board_id.clone());
        game.board.set(new_board);
        game.player.set(player.clone());
        game.leaderboard_id.set(leaderboard_id.clone());
        game.shard_id.set(shard_id.clone());
        game.chain_id.set(player_obj.chain_id.get().to_string());
        game.end_time.set(end_time);
        game.created_at.set(timestamp);

        contract.state.latest_board_id.set(board_id.clone());

        // increment player and board count
        let leaderboard_chain_id = ChainId::from_str(&leaderboard_id).unwrap();
        contract.runtime
            .prepare_message(Message::LeaderboardNewGame {
                player: player.clone(),
                board_id: board_id.clone(),
                timestamp,
            })
            .send_to(leaderboard_chain_id);
    }

    pub async fn handle_create_leaderboard(
        contract: &mut crate::Game2048Contract,
        leaderboard_id: String,
        name: String,
        description: Option<String>,
        chain_id: String,
        host: String,
        start_time: u64,
        end_time: u64,
    ) {
        let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();

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

    pub async fn handle_leaderboard_new_game(
        contract: &mut crate::Game2048Contract,
        player: String,
        board_id: String,
        timestamp: u64,
    ) {
        let leaderboard = contract.is_leaderboard_active(timestamp).await;
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

    pub async fn handle_update_score(
        contract: &mut crate::Game2048Contract,
        player: String,
        board_id: String,
        score: u64,
        is_end: bool,
        timestamp: u64,
    ) {
        contract.update_shard_score(&player, board_id, score, timestamp)
            .await;

        let shard = contract.state.shards.load_entry_mut("").await.unwrap();
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
            let shard = contract.state.shards.load_entry_mut("").await.unwrap();
            let leaderboard_id = shard.leaderboard_id.get().clone();

            // Collect all scores and board IDs from shard
            let mut scores = std::collections::HashMap::new();
            let mut board_ids = std::collections::HashMap::new();

            shard
                .score
                .for_each_index_value(|player, score| {
                    scores.insert(player.clone(), *score);
                    Ok(())
                })
                .await
                .unwrap();
            shard
                .board_ids
                .for_each_index_value(|player, board_id| {
                    board_ids.insert(player.clone(), board_id.to_string());
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
                contract.runtime
                    .prepare_message(Message::Flush { board_ids, scores })
                    .send_to(main_chain_id);
            }
        }

        contract.auto_faucet(Some(1));
    }

    pub async fn handle_flush(
        contract: &mut crate::Game2048Contract,
        board_ids: std::collections::HashMap<String, String>,
        scores: std::collections::HashMap<String, u64>,
    ) {
        let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();

        // 1. Only process incoming scores (O(n) complexity)
        for (player, score) in scores.iter() {
            if let Some(board_id) = board_ids.get(player) {
                // 2. Atomic compare-and-swap per entry
                let current_score = leaderboard
                    .score
                    .get(&player.clone())
                    .await
                    .unwrap_or_default()
                    .unwrap_or(0);
                if *score > current_score {
                    // 3. Single insert operation per improvement
                    leaderboard.score.insert(&player.clone(), *score).unwrap();
                    leaderboard
                        .board_ids
                        .insert(&player.clone(), board_id.clone())
                        .unwrap();
                }
            }
        }

        contract.auto_faucet(Some(1));
    }
}