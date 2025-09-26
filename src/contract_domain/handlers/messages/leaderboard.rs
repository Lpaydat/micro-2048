use std::str::FromStr;
use linera_sdk::views::View;
/// Leaderboard Messages Handler
///
/// Handles leaderboard-related messages including creation, game notifications, score updates, and flushing.
use linera_sdk::linera_base_types::ChainId;
use game2048::Message;

pub struct LeaderboardMessageHandler;

impl LeaderboardMessageHandler {
    pub async fn handle_create_leaderboard(
        contract: &mut crate::Game2048Contract,
        leaderboard_id: String,
        name: String,
        description: Option<String>,
        chain_id: String,
        host: String,
        start_time: u64,
        end_time: u64,
        shard_ids: Vec<String>,
    ) {
        let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();

        if !name.is_empty() {
            leaderboard.name.set(name.clone());
        }

        if let Some(ref desc) = description {
            leaderboard.description.set(desc.clone());
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

        // Add provided shard IDs to leaderboard (from main chain)
        
        for shard_id in &shard_ids {
            leaderboard.shard_ids.push_back(shard_id.clone());
            if leaderboard.current_shard_id.get().is_empty() {
                leaderboard.current_shard_id.set(shard_id.clone());
            }
        }

        // Emit ActiveTournaments event so clients know the leaderboard is ready with real shard IDs
        contract.emit_active_tournaments().await;
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
