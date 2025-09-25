//! Game Operations Handler
//! 
//! Handles game-related operations including moves and board creation.

use std::str::FromStr;
use linera_sdk::{
    linera_base_types::ChainId,
};
use game2048::{Direction, Game, Message};
use crate::contract_domain::game_logic::{GameMoveProcessor, GameMoveResult};

pub struct GameOperationHandler;

impl GameOperationHandler {
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
}
