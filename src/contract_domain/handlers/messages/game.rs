use game2048::{hash_seed, Game, RegistrationCheck};
/// Game Messages Handler
///
/// Handles game-related messages including board creation.
/// 
/// 🚀 MESSAGE-BASED ARCHITECTURE: Score updates now use SubmitScore message.
/// No events are emitted on board creation (score=0 boards don't send messages).

pub struct GameMessageHandler;

impl GameMessageHandler {
    pub async fn handle_create_new_board(
        contract: &mut crate::Game2048Contract,
        seed: String,
        player: String,
        timestamp: u64,
        leaderboard_id: String,
        shard_id: String,
        start_time: u64,
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
        game.shard_id.set(shard_id);
        game.chain_id.set(player_obj.chain_id.get().to_string());
        game.start_time.set(start_time);
        game.end_time.set(end_time);
        game.created_at.set(timestamp);

        contract.state.latest_board_id.set(board_id);

        // 🚀 MESSAGE-BASED: No event emission on board creation
        // Score=0 boards don't send SubmitScore messages
        // First SubmitScore is sent when player makes moves and score > 0
    }
}
