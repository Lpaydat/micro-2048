/// Leaderboard Messages Handler
///
/// Handles leaderboard-related messages including creation and score submissions.
use linera_sdk::views::View;

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
        base_triggerer_count: u32,
        total_shard_count: u32,
    ) {
        let leaderboard = contract
            .state
            .leaderboards
            .load_entry_mut("")
            .await
            .unwrap();
        let shard = contract.state.shards.load_entry_mut("").await.unwrap();

        // 🔒 FIX: Clear old tournament data when creating/updating a tournament
        // Check if this is a NEW tournament (different leaderboard_id) or fresh chain
        let current_tournament_id = leaderboard.leaderboard_id.get();
        let is_new_tournament = current_tournament_id.is_empty() || current_tournament_id != &leaderboard_id;
        
        if is_new_tournament && !leaderboard_id.is_empty() {
            // Clear all old score data from previous tournament
            leaderboard.score.clear();
            leaderboard.board_ids.clear();
            leaderboard.is_ended.clear();
            leaderboard.active_boards.clear();
            leaderboard.player_activity_scores.clear();
            leaderboard.player_board_counts.clear();
            leaderboard.total_boards.set(0);
            leaderboard.total_players.set(0);
            
            // Clear shard data too
            shard.score.clear();
            shard.board_ids.clear();
            shard.is_ended.clear();
            shard.player_chain_ids.clear();
            shard.highest_tiles.clear();
            shard.game_statuses.clear();
            shard.counter.set(0);
            shard.active_boards.clear();
            shard.tournament_player_board_counts.clear();
            shard.monitored_player_chains.clear();
            shard.active_players_count.set(0);
            shard.total_games_count.set(0);
            shard.player_activity_levels.clear();
            shard.player_last_seen.clear();
            shard.player_read_intervals.clear();
        }

        leaderboard.leaderboard_id.set(leaderboard_id.clone());
        leaderboard.name.set(name);
        leaderboard.description.set(description.unwrap_or_default());
        leaderboard.chain_id.set(chain_id.clone());
        leaderboard.host.set(host);
        leaderboard.start_time.set(start_time);
        leaderboard.end_time.set(end_time);
        leaderboard.admin_base_triggerer_count.set(base_triggerer_count);
        // total_shard_count is used by shards, not leaderboard
        let _ = total_shard_count;

        // Clear and repopulate shard_ids
        loop {
            match leaderboard.shard_ids.read_front(1).await {
                Ok(items) if !items.is_empty() => {
                    leaderboard.shard_ids.delete_front();
                }
                _ => break,
            }
        }

        for shard_id in shard_ids.iter() {
            leaderboard.shard_ids.push_back(shard_id.clone());
        }

        // Also update shard with the same info
        shard.shard_id.set(chain_id.clone());
        shard.leaderboard_id.set(leaderboard_id);
        shard.chain_id.set(chain_id);
        shard.start_time.set(start_time);
        shard.end_time.set(end_time);
    }

    /// 🚀 PRIMARY: Handle direct score submission from player chain
    /// 
    /// This is the main handler for the message-based architecture.
    /// Player chains send SubmitScore directly to leaderboard chain.
    /// 
    /// 🔒 VALIDATION: Validates that tournament times in message match leaderboard times
    pub async fn handle_submit_score(
        contract: &mut crate::Game2048Contract,
        player: String,
        player_chain_id: String,
        board_id: String,
        score: u64,
        highest_tile: u64,
        game_status: game2048::GameStatus,
        timestamp: u64,
        boards_in_tournament: u32,
        tournament_start_time: u64,
        tournament_end_time: u64,
    ) {
        let leaderboard = contract
            .state
            .leaderboards
            .load_entry_mut("")
            .await
            .unwrap();

        // 🔒 VALIDATION: Verify tournament times match leaderboard's stored times
        // This prevents tampered submissions from player chains with modified times
        let lb_start_time = *leaderboard.start_time.get();
        let lb_end_time = *leaderboard.end_time.get();
        
        if tournament_start_time != lb_start_time {
            // Silently reject - times don't match (possible tampering or stale board)
            return;
        }
        if tournament_end_time != lb_end_time {
            // Silently reject - times don't match (possible tampering or stale board)
            return;
        }

        // Get current best score for this player
        let current_best = leaderboard.score.get(&player).await.unwrap().unwrap_or(0);
        let is_new_player = current_best == 0 && leaderboard.board_ids.get(&player).await.unwrap().is_none();

        // Only update if better score (or equal score from ended game)
        let is_ended = matches!(game_status, game2048::GameStatus::Ended(_));
        let should_update = score > current_best || (score == current_best && is_ended);

        if should_update {
            leaderboard.score.insert(&player, score).unwrap();
            leaderboard.board_ids.insert(&player, board_id.clone()).unwrap();
            leaderboard.highest_tiles.insert(&player, highest_tile).unwrap();
            leaderboard.last_update.insert(&player, timestamp).unwrap();
            
            // Update global leaderboard timestamp for staleness check
            leaderboard.leaderboard_last_update.set(timestamp);
        }

        // Track game ended status
        if is_ended {
            leaderboard.is_ended.insert(&player, true).unwrap();
            // Remove from active boards if it was there
            let _ = leaderboard.active_boards.remove(&board_id);
        } else {
            // Track as active board
            leaderboard
                .active_boards
                .insert(
                    &board_id,
                    crate::state::ActiveBoardInfo {
                        player: player.clone(),
                        score,
                        is_ended: false,
                    },
                )
                .unwrap();
        }

        // Track new player
        if is_new_player {
            let count = *leaderboard.total_players.get();
            leaderboard.total_players.set(count + 1);
        }

        // Update board count for this player (take max seen)
        let current_board_count = leaderboard
            .player_board_counts
            .get(&player_chain_id)
            .await
            .unwrap()
            .unwrap_or(0);
        if boards_in_tournament > current_board_count {
            leaderboard
                .player_board_counts
                .insert(&player_chain_id, boards_in_tournament)
                .unwrap();

            // Recalculate total boards
            let mut total = 0u32;
            leaderboard
                .player_board_counts
                .for_each_index_value(|_, count| {
                    total += *count;
                    Ok(())
                })
                .await
                .unwrap();
            leaderboard.total_boards.set(total);
        }
    }
}
