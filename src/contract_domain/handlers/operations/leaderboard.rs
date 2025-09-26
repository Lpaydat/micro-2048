//! Leaderboard Operations Handler
//! 
//! Handles leaderboard-related operations including creation, updates, and management.

use std::str::FromStr;
use linera_sdk::{
    linera_base_types::{Amount, ApplicationPermissions, ChainId, Timestamp},
};
use game2048::{LeaderboardAction, LeaderboardSettings, RegistrationCheck, Message};

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
        contract.validate_player_password(&player, &password_hash).await;
        let is_main_chain = contract.is_main_chain();
        if !is_main_chain {
            panic!("Only main chain can perform event leaderboard action");
        }

        contract.check_player_registered(&player, RegistrationCheck::EnsureRegistered)
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
            let chain_id = contract.runtime.open_chain(chain_ownership, application_permissions, amount);
            chain_id
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
                let start_time = settings.start_time.parse::<u64>().unwrap();
                let end_time = settings.end_time.parse::<u64>().unwrap();

                let current_time = contract.runtime.system_time();
                let end_timestamp = Timestamp::from(end_time);
                if start_time >= end_time {
                    panic!("Start time cannot be after end time");
                } else if current_time >= end_timestamp {
                    panic!("Current time cannot be after planned end time");
                };

                if !settings.name.is_empty() {
                    leaderboard.name.set(settings.name.clone());
                }

                if let Some(desc) = settings.description.clone() {
                    leaderboard.description.set(desc);
                }

                if start_time != 0 {
                    leaderboard.start_time.set(start_time);
                }

                if end_time != 0 {
                    leaderboard.end_time.set(end_time);
                }

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
                        let shard_application_permissions = ApplicationPermissions::new_single(shard_app_id);
                        let shard_amount = Amount::from_tokens(1);
                        let shard_id = contract.runtime.open_chain(shard_chain_ownership, shard_application_permissions, shard_amount);
                        

                        created_shard_ids.push(shard_id.to_string());
                        
                        // Send CreateLeaderboard message to each shard
                        contract.runtime
                            .prepare_message(Message::CreateLeaderboard {
                                leaderboard_id: chain_id.to_string(),
                                name: settings.name.clone(),
                                description: settings.description.clone(),
                                chain_id: chain_id.to_string(),
                                host: player.clone(),
                                start_time,
                                end_time,
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
                    main_leaderboard.current_shard_id.set(created_shard_ids.first().cloned().unwrap_or_default());

                    // Send CreateLeaderboard message to new leaderboard chain with shard IDs
                    log::info!("🔍 DEBUG: Sending CreateLeaderboard to leaderboard chain {} with {} shard IDs: {:?}", chain_id, created_shard_ids.len(), created_shard_ids);
                    contract.runtime
                        .prepare_message(Message::CreateLeaderboard {
                            leaderboard_id: chain_id.to_string(),
                            name: settings.name.clone(),
                            description: settings.description.clone(),
                            chain_id: chain_id.to_string(),
                            host: player.clone(),
                            start_time,
                            end_time,
                            shard_ids: created_shard_ids.clone(),
                        })
                        .send_to(chain_id);
                } else if action == LeaderboardAction::Update {
                    // For updates, just send message to existing leaderboard chain (no shard creation)
                    contract.runtime
                        .prepare_message(Message::CreateLeaderboard {
                            leaderboard_id: chain_id.to_string(),
                            name: settings.name.clone(),
                            description: settings.description.clone(),
                            chain_id: chain_id.to_string(),
                            host: player.clone(),
                            start_time,
                            end_time,
                            shard_ids: vec![], // No shard changes on update
                        })
                        .send_to(chain_id);
                }
            }
            LeaderboardAction::Delete => {
                if leaderboard.leaderboard_id.get().is_empty() {
                    panic!("Cannot delete the main leaderboard");
                }

                contract.state
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
}
