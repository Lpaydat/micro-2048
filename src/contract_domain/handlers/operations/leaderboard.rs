//! Leaderboard Operations Handler
//! 
//! Handles leaderboard-related operations including creation, updates, and management.

use std::str::FromStr;
use linera_sdk::{
    linera_base_types::{Amount, ApplicationPermissions, ChainId, Timestamp},
};
use game2048::{LeaderboardAction, LeaderboardSettings, RegistrationCheck};

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
                }
                
                contract.upsert_leaderboard(
                    chain_id,
                    &settings.name,
                    &settings.description.unwrap_or_default(),
                    &player,
                    start_time,
                    end_time,
                    None,
                )
                .await;
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
