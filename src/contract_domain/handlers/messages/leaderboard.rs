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
        let mut shard_chain_ids = Vec::new();
        
        for shard_id in &shard_ids {
            leaderboard.shard_ids.push_back(shard_id.clone());
            if leaderboard.current_shard_id.get().is_empty() {
                leaderboard.current_shard_id.set(shard_id.clone());
            }
            
            // Collect chain IDs for subscription
            if let Ok(chain_id) = std::str::FromStr::from_str(&shard_id) {
                shard_chain_ids.push((shard_id.clone(), chain_id));
            } else {
                log::error!("🔔 LEADERBOARD_SETUP: ❌ Failed to parse shard_id: {}", shard_id);
            }
        }
        
        // End the borrow scope before subscribing
        let _ = leaderboard;
        let _ = shard;
        
        // 🚀 NEW: Subscribe to shard score events for streaming
        for (shard_id, chain_id) in shard_chain_ids {
            log::info!("🔔 LEADERBOARD_SETUP: Subscribing to shard_score_update events from shard {}", shard_id);
            contract.subscribe_to_shard_score_events(chain_id);
            log::info!("🔔 LEADERBOARD_SETUP: ✅ Successfully subscribed to shard {}", shard_id);
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
        log::info!("🏆 LB_BOARD_REGISTER: Received LeaderboardNewGame message for player {} board {}", player, board_id);
        let leaderboard = contract.is_leaderboard_active(timestamp).await;
        let total_boards = leaderboard.total_boards.get_mut();
        *total_boards += 1;
        log::info!("🏆 LB_BOARD_REGISTER: ✅ Board registered on leaderboard, total boards now: {}", *total_boards);

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
        contract.update_shard_score(&player, board_id, score, timestamp, format!("legacy_{}", player), 1, "legacy_tournament".to_string(), game2048::GameStatus::Active, 2)
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

    /// Handle first player registration from shard for triggerer system
    pub async fn handle_register_first_player(
        contract: &mut crate::Game2048Contract,
        shard_chain_id: String,
        player_chain_id: String,
        tournament_id: String,
    ) {
        log::info!("🎯 TRIGGERER_REGISTER: Registering first player {} from shard {} for tournament {}", 
                   player_chain_id, shard_chain_id, tournament_id);
        
        let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
        
        // Verify this is the correct tournament
        if leaderboard.leaderboard_id.get() == &tournament_id {
            // Check if we need to set primary triggerer
            if leaderboard.primary_triggerer.get().is_empty() {
                log::info!("🎯 TRIGGERER_REGISTER: Setting {} as primary triggerer", player_chain_id);
                leaderboard.primary_triggerer.set(player_chain_id.clone());
            } else {
                // Add to backup triggerers (max 4 total) - check by reading existing
                match leaderboard.backup_triggerers.read_front(4).await {
                    Ok(backups) => {
                        if backups.len() < 4 {
                            log::info!("🎯 TRIGGERER_REGISTER: Adding {} as backup triggerer #{}", player_chain_id, backups.len() + 1);
                            leaderboard.backup_triggerers.push_back(player_chain_id.clone());
                        } else {
                            log::info!("🎯 TRIGGERER_REGISTER: Max triggerers reached, skipping {}", player_chain_id);
                            return;
                        }
                    }
                    Err(_) => {
                        log::info!("🎯 TRIGGERER_REGISTER: Adding {} as first backup triggerer", player_chain_id);
                        leaderboard.backup_triggerers.push_back(player_chain_id.clone());
                    }
                }
            }
            
            // Get all player activity scores for triggerer list
            let mut all_players_activity = Vec::new();
            
            // Add this newly registered player with initial activity score of 1
            all_players_activity.push((player_chain_id.clone(), 1u32));
            
            // Add any existing players from activity scores
            leaderboard.player_activity_scores.for_each_index_value_while(|player_id, activity_score| {
                if player_id != player_chain_id {
                    all_players_activity.push((player_id, *activity_score));
                }
                Ok(true)
            }).await.unwrap();
            
            // Sort by activity score (highest first) - same as activity-based ranking
            all_players_activity.sort_by(|(_, a), (_, b)| b.cmp(a));
            
            // Emit updated activity-based triggerer list to all player chains
            Self::emit_activity_based_triggerer_list(contract, &tournament_id, all_players_activity).await;
            
            log::info!("🎯 TRIGGERER_REGISTER: ✅ Successfully registered {} as triggerer", player_chain_id);
        } else {
            log::warn!("🎯 TRIGGERER_REGISTER: ❌ Tournament ID mismatch: expected {}, got {}", 
                      leaderboard.leaderboard_id.get(), tournament_id);
        }
    }



    /// Emit activity-based triggerer list update event 
    async fn emit_activity_based_triggerer_list(
        contract: &mut crate::Game2048Contract,
        tournament_id: &str,
        all_players_activity: Vec<(String, u32)>,
    ) {
        use linera_sdk::linera_base_types::StreamName;
        use game2048::GameEvent;
        
        let current_time = contract.runtime.system_time().micros();
        
        // Default threshold: 10ms between triggers (for testing)
        let threshold_config = 10_000; // 10ms in microseconds
        
        let triggerer_update_event = GameEvent::LeaderboardUpdate {
            leaderboard_id: tournament_id.to_string(),
            triggerer_list: all_players_activity.clone(), // Full sorted list with (chain_id, activity_score)
            last_update_timestamp: current_time,
            threshold_config,
            total_registered_players: all_players_activity.len() as u32,
        };
        
        let stream_name = StreamName::from("leaderboard_update".to_string());
        log::info!("📡 EMIT: Broadcasting full activity-based triggerer list with {} players for tournament {}", 
                   all_players_activity.len(), tournament_id);
        contract.runtime.emit(stream_name, &triggerer_update_event);
        log::info!("📡 EMIT: ✅ Successfully broadcasted activity-based triggerer list");
    }

    /// Handle trigger update request from player chain
    pub async fn handle_trigger_update(
        contract: &mut crate::Game2048Contract,
        triggerer_chain_id: String,
        tournament_id: String,
        timestamp: u64,
    ) {
        log::info!("⏰ TRIGGER_UPDATE: Processing trigger request from {} for tournament {}", 
                   triggerer_chain_id, tournament_id);
        
        let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
        
        // Verify this is the correct tournament
        if leaderboard.leaderboard_id.get() != &tournament_id {
            log::warn!("⏰ TRIGGER_UPDATE: ❌ Tournament ID mismatch: expected {}, got {}", 
                      leaderboard.leaderboard_id.get(), tournament_id);
            return;
        }
        
        // Check if this chain is authorized to trigger
        let is_authorized = leaderboard.primary_triggerer.get() == &triggerer_chain_id ||
            Self::is_backup_triggerer(&leaderboard, &triggerer_chain_id).await;
        
        if !is_authorized {
            log::warn!("⏰ TRIGGER_UPDATE: ❌ Unauthorized trigger from {}", triggerer_chain_id);
            return;
        }
        
        // Check cooldown period
        let cooldown_until = *leaderboard.trigger_cooldown_until.get();
        
        if timestamp < cooldown_until {
            log::info!("⏰ TRIGGER_UPDATE: ⏳ Trigger in cooldown period until {}, current: {}", 
                      cooldown_until, timestamp);
            return;
        }
        
        // Update trigger tracking
        leaderboard.last_trigger_time.set(timestamp);
        leaderboard.last_trigger_by.set(triggerer_chain_id.clone());
        
        // Set cooldown: 5ms (for testing)
        let cooldown_duration = 5_000; // 5ms in microseconds
        leaderboard.trigger_cooldown_until.set(timestamp + cooldown_duration);
        
        log::info!("⏰ TRIGGER_UPDATE: ✅ Accepted trigger from {}, starting aggregation", triggerer_chain_id);
        
        // Trigger leaderboard update from shard chains
        use crate::contract_domain::handlers::operations::LeaderboardOperationHandler;
        LeaderboardOperationHandler::update_leaderboard_from_shard_chains(contract, Vec::new()).await;
        
        log::info!("⏰ TRIGGER_UPDATE: ✅ Aggregation completed");
    }

    /// Check if a chain ID is in the backup triggerers list
    async fn is_backup_triggerer(
        leaderboard: &crate::state::Leaderboard,
        chain_id: &str,
    ) -> bool {
        if let Ok(backups) = leaderboard.backup_triggerers.read_front(10).await {
            backups.iter().any(|backup_id| backup_id == chain_id)
        } else {
            false
        }
    }

    /// Update triggerer list based on player activity scores (called after activity updates)
    pub async fn update_triggerer_list_by_activity(
        contract: &mut crate::Game2048Contract,
        tournament_id: &str,
    ) {
        let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
        
        // Verify this is the correct tournament
        if leaderboard.leaderboard_id.get() != tournament_id {
            return;
        }
        
        // Get ALL players with activity scores (not just current triggerers)
        let mut all_players_activity = Vec::new();
        leaderboard.player_activity_scores.for_each_index_value_while(|player_id, activity_score| {
            all_players_activity.push((player_id, *activity_score));
            Ok(true)
        }).await.unwrap();
        
        if all_players_activity.is_empty() {
            log::info!("🎯 ACTIVITY_REORDER: No players with activity scores found for tournament {}", tournament_id);
            return;
        }
        
        // Sort by activity score (highest first), then by chain_id for deterministic ordering
        all_players_activity.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))
        });
        
        log::info!("🎯 ACTIVITY_REORDER: Ranking players by activity score for tournament {}", tournament_id);
        for (i, (player_id, activity_score)) in all_players_activity.iter().take(10).enumerate() {
            log::info!("🎯 ACTIVITY_REORDER:   {}. {} (activity: {})", i + 1, player_id, activity_score);
        }
        
        // Update primary triggerer to most active player
        if let Some((most_active_player, _)) = all_players_activity.first() {
            leaderboard.primary_triggerer.set(most_active_player.clone());
        }
        
        // Clear backup triggerers  
        loop {
            match leaderboard.backup_triggerers.read_front(1).await {
                Ok(items) if !items.is_empty() => {
                    leaderboard.backup_triggerers.delete_front();
                }
                _ => break,
            }
        }
        
        // Add top active players as backups (skip first which is now primary, max 4 backups)
        for (player_id, _) in all_players_activity.iter().skip(1).take(4) {
            leaderboard.backup_triggerers.push_back(player_id.clone());
        }
        
        // Update last successful update time
        leaderboard.last_successful_update.set(contract.runtime.system_time().micros());
        
        // Emit updated triggerer list with full rankings
        Self::emit_activity_based_triggerer_list(contract, tournament_id, all_players_activity).await;
        
        log::info!("🎯 ACTIVITY_REORDER: ✅ Updated triggerer list based on activity scores");
    }
}
