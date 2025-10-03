use game2048::{hash_seed, Game, GameStatus, Message, RegistrationCheck};
/// Game Messages Handler
///
/// Handles game-related messages including board creation.
use linera_sdk::linera_base_types::ChainId;
use std::str::FromStr;

pub struct GameMessageHandler;

impl GameMessageHandler {
    pub async fn handle_create_new_board(
        contract: &mut crate::Game2048Contract,
        seed: String,
        player: String,
        timestamp: u64,
        leaderboard_id: String,
        shard_id: String,
        end_time: u64,
    ) {
        contract
            .check_player_registered(&player, RegistrationCheck::EnsureRegistered)
            .await;

        let player_obj = contract
            .state
            .players
            .load_entry_mut(&player)
            .await
            .unwrap();

        let current_chain_id = contract.runtime.chain_id().to_string();
        if current_chain_id != *player_obj.chain_id.get() {
            panic!("You can only create board on your own chain");
        }

        let mut board_id = hash_seed(&seed, &player, timestamp).to_string();
        board_id = format!("{}.{}", player_obj.chain_id.get(), board_id);

        let new_board = Game::new(&board_id, &player, timestamp).board;
        let game = contract
            .state
            .boards
            .load_entry_mut(&board_id)
            .await
            .unwrap();
        game.board_id.set(board_id.clone());
        game.board.set(new_board);
        game.player.set(player.clone());
        game.leaderboard_id.set(leaderboard_id.clone());
        game.shard_id.set(shard_id.clone());
        game.chain_id.set(player_obj.chain_id.get().to_string());
        game.end_time.set(end_time);
        game.created_at.set(timestamp);

        contract.state.latest_board_id.set(board_id.clone());

        // 🚀 NEW: Emit player score update for game creation (score = 0, status = Created)
        // Get current best score for this player (likely 0 for new players)
        let leaderboard_obj = contract
            .state
            .leaderboards
            .load_entry_mut("")
            .await
            .unwrap();
        let current_best = leaderboard_obj
            .score
            .get(&player)
            .await
            .unwrap()
            .unwrap_or(0);

        // Get player's current board count for this tournament
        let player_state = contract
            .state
            .players
            .load_entry_mut(&player)
            .await
            .unwrap();
        let current_board_count = player_state
            .boards_per_tournament
            .get(&leaderboard_id)
            .await
            .unwrap()
            .unwrap_or(0);

        use crate::contract_domain::events::emitters::EventEmitter;
        let chain_id = contract.runtime.chain_id().to_string();
        EventEmitter::emit_player_score_update(
            contract,
            player.clone(),
            board_id.clone(),
            0, // Initial score is 0
            chain_id,
            timestamp,
            GameStatus::Created,
            2, // Initial highest tile
            0,
            leaderboard_id.clone(),
            current_best,
            current_board_count,
        )
        .await;

        // increment player and board count
        let leaderboard_chain_id = ChainId::from_str(&leaderboard_id).unwrap();
        contract
            .runtime
            .prepare_message(Message::LeaderboardNewGame {
                player: player.clone(),
                board_id: board_id.clone(),
                timestamp,
            })
            .send_to(leaderboard_chain_id);
    }
}
