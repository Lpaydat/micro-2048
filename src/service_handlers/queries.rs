use crate::service_handlers::types::*;
use crate::state::Game2048;
use async_graphql::{Enum, Object};
use game2048::Game;
use linera_sdk::ServiceRuntime;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum TournamentFilter {
    /// Show only currently active tournaments (default)
    Active,
    /// Show only past/completed tournaments
    Past,
    /// Show only future/upcoming tournaments  
    Future,
    /// Show all tournaments regardless of time status
    All,
}

pub struct QueryHandler {
    pub state: Arc<Game2048>,
    pub runtime: Arc<ServiceRuntime<crate::Game2048Service>>,
}

#[Object]
impl QueryHandler {
    async fn balance(&self) -> String {
        self.state.balance.get().to_string()
    }

    /// 🚀 NEW: Check if a chain is authorized to trigger aggregation
    async fn is_authorized_triggerer(&self, chain_id: String) -> bool {
        // Check against the main leaderboard (empty string key)
        if let Ok(Some(leaderboard)) = self.state.leaderboards.try_load_entry("").await {
            // Check if primary triggerer
            if leaderboard.primary_triggerer.get() == &chain_id {
                return true;
            }

            // Check backup triggerers
            if let Ok(backups) = leaderboard.backup_triggerers.read_front(5).await {
                return backups.contains(&chain_id);
            }
        }
        false
    }

    /// 🚀 NEW: Get current triggerer pool for transparency
    async fn get_triggerer_pool(&self) -> TriggererPool {
        let mut pool = TriggererPool {
            primary: None,
            backups: Vec::new(),
            last_trigger_time: 0,
            cooldown_until: 0,
        };

        if let Ok(Some(leaderboard)) = self.state.leaderboards.try_load_entry("").await {
            pool.primary = Some(leaderboard.primary_triggerer.get().to_string());
            pool.last_trigger_time = *leaderboard.last_trigger_time.get();
            pool.cooldown_until = *leaderboard.trigger_cooldown_until.get();

            if let Ok(backups) = leaderboard.backup_triggerers.read_front(5).await {
                pool.backups = backups;
            }
        }

        pool
    }

    async fn player(&self, username: String) -> Option<Player> {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&username).await {
            Some(Player {
                username: player.username.get().to_string(),
                chain_id: player.chain_id.get().to_string(),
                is_mod: *player.is_mod.get(),
            })
        } else {
            None
        }
    }

    async fn players(&self, usernames: Option<Vec<String>>) -> Vec<Player> {
        let mut usernames = usernames.unwrap_or_default();
        let mut players: Vec<Player> = Vec::new();

        if usernames.is_empty() {
            usernames = self.state.players.indices().await.unwrap();
        }

        for username in usernames {
            if let Ok(Some(player)) = self.state.players.try_load_entry(&username).await {
                players.push(Player {
                    username,
                    chain_id: player.chain_id.get().to_string(),
                    is_mod: *player.is_mod.get(),
                });
            }
        }

        players
    }

    async fn check_player(&self, username: String, password_hash: String) -> Option<bool> {
        if let Ok(Some(player)) = self.state.players.try_load_entry(&username).await {
            let stored_password_hash = player.password_hash.get().to_string();
            Some(stored_password_hash == password_hash)
        } else {
            None
        }
    }

    async fn board(&self, board_id: Option<String>) -> Option<BoardState> {
        let board_id = board_id.unwrap_or(self.state.latest_board_id.get().to_string());
        if let Ok(Some(game)) = self.state.boards.try_load_entry(&board_id).await {
            let game_state = BoardState {
                board_id: game.board_id.get().to_string(),
                board: Game::convert_to_matrix(*game.board.get()),
                is_ended: *game.is_ended.get(),
                score: *game.score.get(),
                player: game.player.get().to_string(),
                chain_id: game.chain_id.get().to_string(),
                leaderboard_id: game.leaderboard_id.get().to_string(),
                shard_id: game.shard_id.get().to_string(),
                created_at: micros_to_millis(*game.created_at.get()),
                end_time: micros_to_millis(*game.end_time.get()),
            };
            Some(game_state)
        } else {
            None
        }
    }

    async fn boards(&self, board_ids: Option<Vec<String>>) -> Vec<BoardState> {
        let mut board_ids = board_ids.unwrap_or_default();
        let mut boards: Vec<BoardState> = Vec::new();

        if board_ids.is_empty() {
            board_ids = self.state.boards.indices().await.unwrap();
        }

        for board_id in board_ids {
            if let Ok(Some(board)) = self.state.boards.try_load_entry(&board_id).await {
                boards.push(BoardState {
                    board_id,
                    board: Game::convert_to_matrix(*board.board.get()),
                    is_ended: *board.is_ended.get(),
                    score: *board.score.get(),
                    player: board.player.get().to_string(),
                    chain_id: board.chain_id.get().to_string(),
                    leaderboard_id: board.leaderboard_id.get().to_string(),
                    shard_id: board.shard_id.get().to_string(),
                    created_at: micros_to_millis(*board.created_at.get()),
                    end_time: micros_to_millis(*board.end_time.get()),
                });
            }
        }

        boards
    }

    async fn leaderboard(
        &self, 
        leaderboard_id: Option<String>,
        top: Option<u32>,
        offset: Option<u32>,
    ) -> Option<LeaderboardState> {
        let mut players: HashMap<String, Ranker> = HashMap::new();
        let leaderboard_id = leaderboard_id.unwrap_or("".to_string());

        if let Ok(Some(leaderboard)) = self
            .state
            .leaderboards
            .try_load_entry(&leaderboard_id)
            .await
        {
            leaderboard
                .score
                .for_each_index_value(|username, score| {
                    players.insert(
                        username.clone(),
                        Ranker {
                            username,
                            score: *score,
                            board_id: leaderboard_id.clone(),
                        },
                    );
                    Ok(())
                })
                .await
                .unwrap();
            leaderboard
                .board_ids
                .for_each_index_value(|username, board_id| {
                    if let Some(ranker) = players.get_mut(&username) {
                        ranker.board_id = board_id.to_string();
                    }
                    Ok(())
                })
                .await
                .unwrap();

            // 🚀 SORT rankers by score descending (highest first)
            let mut rankers: Vec<Ranker> = players.into_values().collect();
            rankers.sort_by(|a, b| b.score.cmp(&a.score));
            
            // 🚀 PAGINATION: Apply top/offset
            let top = top.unwrap_or(100) as usize; // Default: top 100
            let offset = offset.unwrap_or(0) as usize;
            let rankers: Vec<Ranker> = rankers.into_iter()
                .skip(offset)
                .take(top)
                .collect();

            let shard_ids = leaderboard.shard_ids.read_front(100).await.unwrap();
            let leaderboard_state = LeaderboardState {
                leaderboard_id,
                chain_id: leaderboard.chain_id.get().to_string(),
                name: leaderboard.name.get().to_string(),
                description: Some(leaderboard.description.get().to_string()),
                is_pinned: *leaderboard.is_pinned.get(),
                host: leaderboard.host.get().to_string(),
                start_time: micros_to_millis(*leaderboard.start_time.get()),
                end_time: micros_to_millis(*leaderboard.end_time.get()),
                total_boards: *leaderboard.total_boards.get(),
                total_players: *leaderboard.total_players.get(),
                rankers,
                shard_ids,
            };

            Some(leaderboard_state)
        } else {
            None
        }
    }

    /// Query tournaments with optional filtering by time status (defaults to active)
    async fn leaderboards(&self, filter: Option<TournamentFilter>) -> Vec<LeaderboardState> {
        let filter = filter.unwrap_or(TournamentFilter::Active);
        self.get_tournaments_by_filter(filter).await
    }

    /// Query tournaments filtered by specific status
    async fn tournaments_by_status(&self, filter: TournamentFilter) -> Vec<LeaderboardState> {
        self.get_tournaments_by_filter(filter).await
    }

    async fn shards(&self) -> Shard {
        if let Some(shard) = self.state.shards.try_load_entry("").await.unwrap() {
            let mut scores: HashMap<String, u64> = HashMap::new();
            let mut board_ids: HashMap<String, String> = HashMap::new();
            shard
                .score
                .for_each_index_value(|username, score| {
                    scores.insert(username.clone(), *score);
                    Ok(())
                })
                .await
                .unwrap();
            shard
                .board_ids
                .for_each_index_value(|username, board_id| {
                    board_ids.insert(username.clone(), board_id.to_string());
                    Ok(())
                })
                .await
                .unwrap();
            Shard {
                shard_id: shard.shard_id.get().to_string(),
                leaderboard_id: shard.leaderboard_id.get().to_string(),
                chain_id: shard.chain_id.get().to_string(),
                start_time: shard.start_time.get().to_string(),
                end_time: shard.end_time.get().to_string(),
                counter: *shard.counter.get(),
                scores,
                board_ids,
            }
        } else {
            Shard {
                shard_id: "".to_string(),
                leaderboard_id: "".to_string(),
                chain_id: "".to_string(),
                start_time: "".to_string(),
                end_time: "".to_string(),
                counter: 0,
                scores: HashMap::new(),
                board_ids: HashMap::new(),
            }
        }
    }
}

impl QueryHandler {
    /// Helper method to get tournaments filtered by status
    async fn get_tournaments_by_filter(&self, filter: TournamentFilter) -> Vec<LeaderboardState> {
        let mut leaderboard_ids: Vec<String> = Vec::new();
        self.state
            .leaderboards
            .for_each_index_while(|leaderboard_id| {
                leaderboard_ids.push(leaderboard_id);
                Ok(true)
            })
            .await
            .unwrap();

        let current_time = self.runtime.system_time().micros();

        let mut tournament_games: Vec<LeaderboardState> = Vec::new();
        for leaderboard_id in leaderboard_ids {
            if let Ok(Some(leaderboard)) = self
                .state
                .leaderboards
                .try_load_entry(&leaderboard_id)
                .await
            {
                let start_time_raw = *leaderboard.start_time.get();
                let end_time_raw = *leaderboard.end_time.get();

                let start_time = if start_time_raw == 0 {
                    None
                } else {
                    Some(start_time_raw)
                };
                let end_time = if end_time_raw == 0 {
                    None
                } else {
                    Some(end_time_raw)
                };

                // Determine tournament status
                let is_started = start_time.is_none_or(|start| current_time >= start);
                let is_ended = end_time.is_some_and(|end| current_time >= end);

                let tournament_status = if !is_started {
                    TournamentFilter::Future
                } else if is_ended {
                    TournamentFilter::Past
                } else {
                    TournamentFilter::Active
                };

                // Apply filter
                let include_tournament = match filter {
                    TournamentFilter::All => true,
                    _ => tournament_status == filter,
                };

                if include_tournament {
                    let shard_ids = leaderboard
                        .shard_ids
                        .read_front(100)
                        .await
                        .unwrap_or_default();

                    tournament_games.push(LeaderboardState {
                        leaderboard_id,
                        chain_id: leaderboard.chain_id.get().to_string(),
                        name: leaderboard.name.get().to_string(),
                        description: Some(leaderboard.description.get().to_string()),
                        is_pinned: *leaderboard.is_pinned.get(),
                        host: leaderboard.host.get().to_string(),
                        start_time: micros_to_millis(*leaderboard.start_time.get()),
                        end_time: micros_to_millis(*leaderboard.end_time.get()),
                        total_boards: *leaderboard.total_boards.get(),
                        total_players: *leaderboard.total_players.get(),
                        rankers: Vec::new(),
                        shard_ids,
                    });
                }
            }
        }

        tournament_games
    }
}
