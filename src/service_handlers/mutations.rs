use std::sync::Arc;
use async_graphql::Object;
use linera_sdk::ServiceRuntime;
use game2048::{LeaderboardAction, LeaderboardSettings, Operation};
use crate::state::Game2048;
use crate::Game2048Service;

pub struct MutationHandler {
    pub state: Arc<Game2048>,
    pub runtime: Arc<ServiceRuntime<Game2048Service>>,
}

#[Object]
impl MutationHandler {
    async fn register_player(&self, username: String, password_hash: String) -> [u8; 0] {
        let operation = Operation::RegisterPlayer {
            username,
            password_hash,
        };
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn new_board(
        &self,
        player: String,
        password_hash: String,
        player_chain_id: String,
        timestamp: String,
    ) -> [u8; 0] {
        // Validate player exists and password is correct
        self.validate_player_password(&player, &password_hash).await;

        let operation = Operation::NewBoard {
            player,
            player_chain_id,
            timestamp: timestamp.parse::<u64>().unwrap(),
            password_hash,
        };
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn new_shard(&self) -> [u8; 0] {
        let operation = Operation::NewShard;
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn make_moves(
        &self,
        board_id: String,
        moves: String,
        player: String,
        password_hash: String,
    ) -> [u8; 0] {
        // Validate player exists and password is correct
        self.validate_player_password(&player, &password_hash).await;

        let operation = Operation::MakeMoves {
            board_id,
            moves,
            player,
            password_hash,
        };
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn leaderboard_action(
        &self,
        leaderboard_id: String,
        action: LeaderboardAction,
        settings: LeaderboardSettings,
        player: String,
        password_hash: String,
    ) -> [u8; 0] {
        // Validate player exists and password is correct
        self.validate_player_password(&player, &password_hash).await;

        let operation = Operation::LeaderboardAction {
            leaderboard_id,
            action,
            settings,
            player,
            password_hash,
        };
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn toggle_mod(&self, player: String, password_hash: String, username: String) -> [u8; 0] {
        // Validate player exists and password is correct
        self.validate_player_password(&player, &password_hash).await;

        // Additional admin check
        if player != "lpaydat" {
            panic!("Only lpaydat can toggle admin");
        }

        let operation = Operation::ToggleAdmin { username, player, password_hash };
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn faucet(&self) -> [u8; 0] {
        let operation = Operation::Faucet;
        self.runtime.schedule_operation(&operation);
        []
    }

    async fn close_chain(&self, chain_id: String) -> [u8; 0] {
        let operation = Operation::CloseChain { chain_id };
        self.runtime.schedule_operation(&operation);
        []
    }
}

impl MutationHandler {
    async fn validate_player_password(&self, player_username: &str, provided_password_hash: &str) {
        if let Ok(Some(player)) = self.state.players.try_load_entry(player_username).await {
            let stored_password_hash = player.password_hash.get().to_string();
            if stored_password_hash != provided_password_hash {
                panic!("Invalid password");
            }
        } else {
            panic!("Player not found");
        }
    }
}