//! Leaderboard Operations Handler
//!
//! Handles leaderboard-related operations including creation, updates, management, score aggregation, and triggerer coordination.

use crate::state::Leaderboard;
use game2048::{
    GameEvent, LeaderboardAction, LeaderboardSettings, Message, PlayerScoreSummary,
    RegistrationCheck, TournamentInfo,
};
use linera_sdk::linera_base_types::{Amount, ApplicationPermissions, ChainId};
use std::str::FromStr;

pub struct LeaderboardOperationHandler;

impl LeaderboardOperationHandler {
    pub async fn handle_leaderboard_action(
        contract: &mut crate::Game2048Contract,
        leaderboard_id: String,
        action: LeaderboardAction,
        settings: LeaderboardSettings,
        player: String,
        password_hash: String,
    ) {
        // Validate password
        contract
            .validate_player_password(&player, &password_hash)
            .await;
        let is_main_chain = contract.is_main_chain();
        if !is_main_chain {
            panic!("Only main chain can perform event leaderboard action");
        }

        contract
            .check_player_registered(&player, RegistrationCheck::EnsureRegistered)
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
            contract
                .runtime
                .open_chain(chain_ownership, application_permissions, amount)
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
                // Parse optional start/end times: "0" or empty = None (unlimited)
                let start_time = if settings.start_time.is_empty() || settings.start_time == "0" {
                    None
                } else {
                    Some(settings.start_time.parse::<u64>().unwrap())
                };

                let end_time = if settings.end_time.is_empty() || settings.end_time == "0" {
                    None
                } else {
                    Some(settings.end_time.parse::<u64>().unwrap())
                };

                // Only validate times if both are set
                if let (Some(start), Some(end)) = (start_time, end_time) {
                    // Temporarily skip current time validation to avoid system_time() WASM panic
                    // let current_time = contract.runtime.system_time().micros();
                    if start >= end {
                        panic!("Start time cannot be after end time");
                    }
                    // Commented out to avoid system_time() issues:
                    // else if current_time >= end {
                    //     panic!("Current time cannot be after planned end time");
                    // }
                }

                if !settings.name.is_empty() {
                    leaderboard.name.set(settings.name.clone());
                }

                if let Some(desc) = settings.description.clone() {
                    leaderboard.description.set(desc);
                }

                // Store times: None -> 0 (unlimited), Some(value) -> value
                leaderboard.start_time.set(start_time.unwrap_or(0));
                leaderboard.end_time.set(end_time.unwrap_or(0));

                if action == LeaderboardAction::Create {
                    let chain_id_str = chain_id.to_string();
                    leaderboard.leaderboard_id.set(chain_id_str.clone());
                    leaderboard.chain_id.set(chain_id_str);
                    leaderboard.host.set(player.clone());

                    // Create shard chains from main chain (ONLY on creation)
                    let shard_number = settings.shard_number.unwrap_or(1);
                    let mut created_shard_ids = Vec::new();

                    for _ in 0..shard_number {
                        let shard_chain_ownership = contract.runtime.chain_ownership();
                        let shard_app_id = contract.runtime.application_id().forget_abi();
                        let shard_application_permissions =
                            ApplicationPermissions::new_single(shard_app_id);
                        let shard_amount = Amount::from_tokens(1);
                        let shard_id = contract.runtime.open_chain(
                            shard_chain_ownership,
                            shard_application_permissions,
                            shard_amount,
                        );

                        created_shard_ids.push(shard_id.to_string());

                        // Send CreateLeaderboard message to each shard
                        contract
                            .runtime
                            .prepare_message(Message::CreateLeaderboard {
                                leaderboard_id: chain_id.to_string(),
                                name: settings.name.clone(),
                                description: settings.description.clone(),
                                chain_id: chain_id.to_string(),
                                host: player.clone(),
                                start_time: start_time.unwrap_or(0),
                                end_time: end_time.unwrap_or(0),
                                shard_ids: vec![], // Shards don't need shard IDs
                            })
                            .send_to(shard_id);
                    }

                    // Update main chain leaderboard list with shard info
                    let main_leaderboard = contract
                        .state
                        .leaderboards
                        .load_entry_mut(&chain_id.to_string())
                        .await
                        .unwrap();

                    for shard_id in &created_shard_ids {
                        main_leaderboard.shard_ids.push_back(shard_id.clone());
                    }
                    main_leaderboard
                        .current_shard_id
                        .set(created_shard_ids.first().cloned().unwrap_or_default());

                    // Send CreateLeaderboard message to new leaderboard chain with shard IDs
                    contract
                        .runtime
                        .prepare_message(Message::CreateLeaderboard {
                            leaderboard_id: chain_id.to_string(),
                            name: settings.name.clone(),
                            description: settings.description.clone(),
                            chain_id: chain_id.to_string(),
                            host: player.clone(),
                            start_time: start_time.unwrap_or(0),
                            end_time: end_time.unwrap_or(0),
                            shard_ids: created_shard_ids.clone(),
                        })
                        .send_to(chain_id);

                    // Main chain: emit updated active tournaments registry
                    if is_main_chain {
                        contract.emit_active_tournaments().await;
                    }
                } else if action == LeaderboardAction::Update {
                    // For updates, just send message to existing leaderboard chain (no shard creation)
                    contract
                        .runtime
                        .prepare_message(Message::CreateLeaderboard {
                            leaderboard_id: chain_id.to_string(),
                            name: settings.name.clone(),
                            description: settings.description.clone(),
                            chain_id: chain_id.to_string(),
                            host: player.clone(),
                            start_time: start_time.unwrap_or(0),
                            end_time: end_time.unwrap_or(0),
                            shard_ids: vec![], // No shard changes on update
                        })
                        .send_to(chain_id);

                    // Main chain: emit updated active tournaments registry
                    if is_main_chain {
                        contract.emit_active_tournaments().await;
                    }
                }
            }
            LeaderboardAction::Delete => {
                if leaderboard.leaderboard_id.get().is_empty() {
                    panic!("Cannot delete the main leaderboard");
                }

                contract
                    .state
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

    /// Check if leaderboard is active for the given timestamp
    pub async fn is_leaderboard_active(
        contract: &mut crate::Game2048Contract,
        timestamp: u64,
    ) -> &mut Leaderboard {
        let leaderboard = contract
            .state
            .leaderboards
            .load_entry_mut("")
            .await
            .unwrap();
        let start_time = leaderboard.start_time.get();
        let end_time = leaderboard.end_time.get();

        // Basic bounds checking: prevent obviously invalid timestamps
        if timestamp > u64::MAX / 2 {
            panic!("Timestamp too large");
        }

        // Apply timestamp validation to all chains for consistency with optional time limits
        // Keep bypass for system operations (111970) - used for game ending without moves
        if timestamp != 111970 {
            let start_time_raw = *start_time;
            let end_time_raw = *end_time;

            // Only validate if times are set (non-zero)
            let start_limit = if start_time_raw == 0 {
                None
            } else {
                Some(start_time_raw)
            };
            let end_limit = if end_time_raw == 0 {
                None
            } else {
                Some(end_time_raw)
            };

            let mut invalid = false;
            if let Some(start) = start_limit {
                if timestamp < start {
                    invalid = true;
                }
            }
            if let Some(end) = end_limit {
                if timestamp > end {
                    invalid = true;
                }
            }

            if invalid {
                panic!("Leaderboard is not active for timestamp {}", timestamp);
            }
        }

        leaderboard
    }

    /// Update leaderboard from shard chains with proper index tracking
    pub async fn update_leaderboard_from_shard_chains(
        contract: &mut crate::Game2048Contract,
        shard_chain_ids: Vec<ChainId>,
    ) {
        use std::collections::HashMap;

        let mut all_player_summaries: HashMap<String, PlayerScoreSummary> = HashMap::new();

        // Process each shard chain with index tracking
        for chain_id in shard_chain_ids.iter() {
            let chain_id_str = chain_id.to_string();

            // Get last processed index for this shard chain
            let last_processed_index = contract
                .state
                .shard_score_event_indices
                .get(&chain_id_str)
                .await
                .unwrap()
                .unwrap_or(0);

            // Read ascending from last index until error (blockchain-style)
            let mut current_index = last_processed_index;

            // Read until we hit error (no more events)
            // Read until we hit error (no more events)
            #[allow(clippy::while_let_loop)]
            loop {
                // if let Some(event) = contract.read_shard_score_event_from_chain(*chain_id, current_index as u32) {
                if let Some(event) = None::<game2048::GameEvent> {
                    // Commented out manual event reading
                    match event {
                        GameEvent::ShardScoreUpdate { player_scores, .. } => {
                            // Smart merge player summaries from this shard
                            for (player, summary) in player_scores.iter() {
                                let should_update =
                                    if let Some(existing) = all_player_summaries.get(player) {
                                        // Update if better score OR more recent timestamp
                                        summary.best_score > existing.best_score
                                            || (summary.best_score == existing.best_score
                                                && summary.last_update > existing.last_update)
                                    } else {
                                        true // New player
                                    };

                                if should_update {
                                    // Merge with the BEST data from any shard
                                    let merged_summary = if let Some(existing) =
                                        all_player_summaries.get(player)
                                    {
                                        PlayerScoreSummary {
                                            player: player.clone(),
                                            best_score: summary.best_score.max(existing.best_score),
                                            board_id: if summary.best_score >= existing.best_score {
                                                summary.board_id.clone()
                                            } else {
                                                existing.board_id.clone()
                                            },
                                            chain_id: if summary.best_score >= existing.best_score {
                                                summary.chain_id.clone()
                                            } else {
                                                existing.chain_id.clone()
                                            },
                                            highest_tile: summary
                                                .highest_tile
                                                .max(existing.highest_tile),
                                            last_update: summary
                                                .last_update
                                                .max(existing.last_update),
                                            game_status: if summary.last_update
                                                >= existing.last_update
                                            {
                                                summary.game_status.clone()
                                            } else {
                                                existing.game_status.clone()
                                            },
                                            boards_in_tournament: summary
                                                .boards_in_tournament
                                                .max(existing.boards_in_tournament),
                                        }
                                    } else {
                                        summary.clone()
                                    };

                                    all_player_summaries.insert(player.clone(), merged_summary);
                                }
                            }
                        }
                        _ => {
                            // Ignore other event types
                        }
                    }

                    current_index += 1;
                } else {
                    // Hit error - no more events available
                    break;
                }
            }

            // Update index tracking: save our progress
            if current_index > last_processed_index {
                contract
                    .state
                    .shard_score_event_indices
                    .insert(&chain_id_str, current_index)
                    .unwrap();
            }
        }

        // Update leaderboard state with comprehensive tracking
        if !all_player_summaries.is_empty() {
            let leaderboard = contract
                .state
                .leaderboards
                .load_entry_mut("")
                .await
                .unwrap();
            let mut _players_updated = 0u32;
            let mut total_unique_players = 0u32;

            // Update leaderboard state with all player data
            for (player, summary) in all_player_summaries.iter() {
                let current_score = leaderboard.score.get(player).await.unwrap().unwrap_or(0);
                total_unique_players += 1;

                // Always update if we have better score OR if this is a new player
                if summary.best_score >= current_score {
                    leaderboard
                        .score
                        .insert(player, summary.best_score)
                        .unwrap();
                    leaderboard
                        .board_ids
                        .insert(player, summary.board_id.clone())
                        .unwrap();

                    if summary.best_score > current_score {
                        _players_updated += 1;
                    }
                }
            }

            // Update leaderboard metadata
            leaderboard.total_players.set(total_unique_players);

            // Subscribe to all shard chains for real-time updates
            for chain_id in shard_chain_ids.iter() {
                contract.subscribe_to_shard_score_events(*chain_id);
            }

            // Update triggerer pool based on latest scores
            Self::update_triggerer_pool(contract).await;
        }
    }

    /// Emit current active tournaments (for leaderboard chains)
    pub async fn emit_active_tournaments(contract: &mut crate::Game2048Contract) {


        let is_main_chain = contract.is_main_chain();
        let mut tournaments = Vec::new();
        let current_time = contract.runtime.system_time().micros();

        if is_main_chain {
            // Main chain: emit ALL active leaderboards as central registry
            // Iterate through all leaderboard entries
            let mut leaderboard_ids = Vec::new();
            contract
                .state
                .leaderboards
                .for_each_index_while(|key| {
                    if !key.is_empty() {
                        leaderboard_ids.push(key);
                    }
                    Ok(true) // Continue iteration
                })
                .await
                .unwrap();

            for leaderboard_id in leaderboard_ids {
                if let Ok(Some(leaderboard)) = contract
                    .state
                    .leaderboards
                    .try_load_entry(&leaderboard_id)
                    .await
                {
                    let tournament_id = leaderboard.leaderboard_id.get().clone();
                    if !tournament_id.is_empty() {
                        let start_time_raw = *leaderboard.start_time.get();
                        let end_time_raw = *leaderboard.end_time.get();

                        // 🚀 NEW: Time-based filtering - only include active tournaments
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

                        // Check if tournament is currently active (not expired and started)
                        let is_active = {
                            let started = start_time.is_none_or(|start| current_time >= start);
                            let not_ended = end_time.is_none_or(|end| current_time < end);
                            started && not_ended
                        };

                        if is_active {
                            let tournament_info = TournamentInfo {
                                tournament_id: tournament_id.clone(),
                                name: leaderboard.name.get().clone(),
                                shard_chain_ids: leaderboard
                                    .shard_ids
                                    .read_front(100)
                                    .await
                                    .unwrap_or_default(),
                                start_time,
                                end_time,
                                total_players: *leaderboard.total_players.get(),
                            };
                            tournaments.push(tournament_info);
                        }
                    }
                }
            }
        } else {
            // Non-main chain: emit current chain's leaderboard only
            let leaderboard = contract
                .state
                .leaderboards
                .load_entry_mut("")
                .await
                .unwrap();
            let tournament_id = leaderboard.leaderboard_id.get().clone();

            if !tournament_id.is_empty() {
                let start_time_raw = *leaderboard.start_time.get();
                let end_time_raw = *leaderboard.end_time.get();

                // 🚀 NEW: Time-based filtering for non-main chain too
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

                // Check if tournament is currently active (not expired and started)
                let is_active = {
                    let started = start_time.is_none_or(|start| current_time >= start);
                    let not_ended = end_time.is_none_or(|end| current_time < end);
                    started && not_ended
                };

                if is_active {
                    let tournament_info = TournamentInfo {
                        tournament_id: tournament_id.clone(),
                        name: leaderboard.name.get().clone(),
                        shard_chain_ids: leaderboard
                            .shard_ids
                            .read_front(100)
                            .await
                            .unwrap_or_default(),
                        start_time,
                        end_time,
                        total_players: *leaderboard.total_players.get(),
                    };
                    tournaments.push(tournament_info);
                }
            }
        }

        if !tournaments.is_empty() {
            use crate::contract_domain::events::emitters::EventEmitter;
            let timestamp = contract.runtime.system_time().micros();
            EventEmitter::emit_active_tournaments(
                contract,
                tournaments.clone(),
                timestamp,
            ).await;

            // Log tournament details
            for _tournament in &tournaments {}
        }
    }

    /// Dynamic Triggerer Management - Updates based on actual scores
    pub async fn update_triggerer_pool(contract: &mut crate::Game2048Contract) {
        let leaderboard = contract
            .state
            .leaderboards
            .load_entry_mut("")
            .await
            .unwrap();

        // Collect top 5 players by score
        let mut top_players: Vec<(String, u64)> = Vec::new();

        // Iterate through all scores to find top 5
        leaderboard
            .score
            .for_each_index_value(|player, score| {
                top_players.push((player.clone(), *score));
                Ok(())
            })
            .await
            .unwrap();

        // Sort by score (descending)
        top_players.sort_by(|a, b| b.1.cmp(&a.1));

        // Take top 5 (or fewer if less players)
        top_players.truncate(5);

        // Update triggerer pool
        if !top_players.is_empty() {
            // First player is primary
            if let Some((top_player, _)) = top_players.first() {
                // Get board_id for the top player to get their chain_id
                if let Some(board_id) = leaderboard.board_ids.get(top_player).await.unwrap() {
                    // Extract chain_id from board_id (format: "chain_id.hash")
                    let chain_id = board_id.split('.').next().unwrap_or(top_player).to_string();
                    leaderboard.primary_triggerer.set(chain_id);
                }
            }

            // Clear and rebuild backup pool with players 2-5
            while leaderboard.backup_triggerers.count() > 0 {
                leaderboard.backup_triggerers.delete_front();
            }

            // Add new backups (positions 2-5)
            for i in 1..top_players.len().min(5) {
                if let Some((player, _)) = top_players.get(i) {
                    if let Some(board_id) = leaderboard.board_ids.get(player).await.unwrap() {
                        let chain_id = board_id.split('.').next().unwrap_or(player).to_string();
                        leaderboard.backup_triggerers.push_back(chain_id);
                    }
                }
            }
        }

        // Update rotation counter for tracking
        let counter = *leaderboard.trigger_rotation_counter.get();
        leaderboard.trigger_rotation_counter.set(counter + 1);
    }

    /// Check if a chain is authorized to trigger
    pub async fn is_authorized_triggerer(
        contract: &mut crate::Game2048Contract,
        requester_chain_id: &str,
    ) -> bool {
        let leaderboard = contract
            .state
            .leaderboards
            .load_entry_mut("")
            .await
            .unwrap();

        // Check if primary triggerer
        if leaderboard.primary_triggerer.get() == requester_chain_id {
            return true;
        }

        // Check if in backup pool
        let backup_triggerers = leaderboard
            .backup_triggerers
            .read_front(5)
            .await
            .unwrap_or_default();
        backup_triggerers.contains(&requester_chain_id.to_string())
    }

    /// Handle aggregation trigger request with robust cooldown
    pub async fn handle_aggregation_trigger_request(
        contract: &mut crate::Game2048Contract,
        requester_chain_id: &str,
        _timestamp: u64,
    ) -> Result<(), String> {
        // Check if requester is authorized
        if !Self::is_authorized_triggerer(contract, requester_chain_id).await {
            return Err(format!(
                "Chain {} is not authorized to trigger aggregation",
                requester_chain_id
            ));
        }

        let leaderboard = contract
            .state
            .leaderboards
            .load_entry_mut("")
            .await
            .unwrap();
        let current_time = contract.runtime.system_time().micros();

        // Multi-layer cooldown checks
        let cooldown_until = *leaderboard.trigger_cooldown_until.get();
        let last_trigger_time = *leaderboard.last_trigger_time.get();

        // Check explicit cooldown
        if current_time < cooldown_until {
            // Don't error, just silently ignore (prevents error spam)
            return Ok(());
        }

        // Check minimum time between triggers (3 seconds hard minimum)
        let time_since_last = current_time.saturating_sub(last_trigger_time);
        if time_since_last < 3_000_000 {
            // Too frequent - silently ignore
            return Ok(());
        }

        // Set cooldown IMMEDIATELY before doing any work
        leaderboard
            .trigger_cooldown_until
            .set(current_time + 5_000_000); // 5 second cooldown
        leaderboard.last_trigger_time.set(current_time);
        leaderboard
            .last_trigger_by
            .set(requester_chain_id.to_string());

        // Send trigger messages to all shards
        let shard_ids = leaderboard
            .shard_ids
            .read_front(100)
            .await
            .unwrap_or_default();
        if shard_ids.is_empty() {
            return Err("No shards registered for this leaderboard".to_string());
        }

        for shard_id_str in shard_ids {
            if let Ok(shard_chain_id) = ChainId::from_str(&shard_id_str) {
                contract
                    .runtime
                    .prepare_message(Message::TriggerShardAggregation {
                        timestamp: current_time, // Use current time
                    })
                    .send_to(shard_chain_id);
            }
        }

        // Update aggregation counter for monitoring
        let rotation_counter = *leaderboard.trigger_rotation_counter.get();
        leaderboard
            .trigger_rotation_counter
            .set(rotation_counter + 1);

        Ok(())
    }
}
