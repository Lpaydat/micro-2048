// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

//! Main contract implementation with modular operation handlers

use gamehub::{GameHubAbi, Operation, Message};
use gamehub::infrastructure::state::GameHubState;
use gamehub::core::types::{GameHubEvent, EventType};
use gamehub::infrastructure::handlers::{
    operations::{
        PlayerOperationHandler, AdminOperationHandler, ModerationOperationHandler,
        GameOperationHandler, EventOperationHandler, ConfigOperationHandler,
        player_operations::PlayerOperation,
        admin_operations::AdminOperation,
        moderation_operations::ModerationOperation,
        game_operations::GameOperation,
        event_operations::EventOperation,
        config_operations::ConfigOperation,
    },
    messages::{
        handle_register_game_message,
        handle_batch_event_update_message,
        MessageValidator,
    },
    traits::{OperationHandler, ContractInterface},
};
use linera_sdk::{
    linera_base_types::{WithContractAbi, Timestamp},
    views::{RootView, View},
    Contract, ContractRuntime,
};

// ANCHOR: contract_struct
linera_sdk::contract!(GameHubContract);

pub struct GameHubContract {
    pub state: GameHubState,
    pub runtime: ContractRuntime<Self>,
}
// ANCHOR_END: contract_struct

impl WithContractAbi for GameHubContract {
    type Abi = GameHubAbi;
}

impl ContractInterface for GameHubContract {
    fn get_state(&mut self) -> &mut GameHubState {
        &mut self.state
    }
    
    fn get_timestamp(&mut self) -> Timestamp {
        self.runtime.system_time()
    }
    
    fn log_event(&mut self, event_type: EventType, description: String, actor_id: Option<String>, target_id: Option<String>) {
        let event_id = format!("{}_{}", self.runtime.system_time().micros(), self.runtime.system_time().micros());
        let event = GameHubEvent {
            id: event_id.clone(),
            event_type: event_type.clone(),
            description: description.clone(),
            actor_id,
            target_id,
            timestamp: self.runtime.system_time(),
            metadata: None,
        };
        
        // Store the event with proper error handling
        if let Err(error) = self.state.gamehub_events.insert(&event_id, event) {
            // Log error to console but continue execution (don't panic the contract)
            eprintln!("Failed to store event {}: {}. Event details: {:?} - {}", 
                     event_id, error, event_type, description);
        }
    }
}

impl Contract for GameHubContract {
    type Message = Message;
    type InstantiationArgument = ();
    type Parameters = ();
    type EventValue = ();

    // ANCHOR: load
    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = GameHubState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        GameHubContract { state, runtime }
    }
    // ANCHOR_END: load

    async fn instantiate(&mut self, _argument: ()) {
        // Validate that the application parameters were configured correctly.
        self.runtime.application_parameters();

        // Initialize contract with system admin (use a valid Discord ID format for system)
        // In a real deployment, this would be replaced with the actual admin Discord ID
        let system_admin_id = "123456789012345678"; // Valid Discord ID format for system initialization
        let timestamp = self.runtime.system_time();
        
        match self.state.initialize_contract(system_admin_id, timestamp).await {
            Ok(()) => {
                println!("Contract initialized successfully with system admin");
            },
            Err(e) => {
                eprintln!("Failed to initialize contract: {}", e);
                // Don't panic during initialization - log the error but continue
            }
        }
    }

    // ANCHOR: execute_operation
    async fn execute_operation(&mut self, operation: Operation) -> String {
        match operation {
            // Player operations
            Operation::RegisterPlayer { discord_id, username, avatar_url } => {
                PlayerOperationHandler::handle(self, 
                    PlayerOperation::RegisterPlayer {
                        discord_id, username, avatar_url
                    }).await
            },
            Operation::UpdatePlayerProfile { discord_id, username, avatar_url } => {
                PlayerOperationHandler::handle(self, 
                    PlayerOperation::UpdatePlayerProfile {
                        discord_id, username, avatar_url
                    }).await
            },
            
            // Admin operations
            Operation::ApproveGame { caller_discord_id, game_id } => {
                AdminOperationHandler::handle(self,
                    AdminOperation::ApproveGame {
                        caller_discord_id, game_id
                    }).await
            },
            Operation::RejectGame { caller_discord_id, game_id, reason } => {
                AdminOperationHandler::handle(self,
                    AdminOperation::RejectGame {
                        caller_discord_id, game_id, reason
                    }).await
            },
            Operation::AddAdmin { caller_discord_id, discord_id } => {
                AdminOperationHandler::handle(self,
                    AdminOperation::AddAdmin {
                        caller_discord_id, discord_id
                    }).await
            },
            Operation::RemoveAdmin { caller_discord_id, discord_id } => {
                AdminOperationHandler::handle(self,
                    AdminOperation::RemoveAdmin {
                        caller_discord_id, discord_id
                    }).await
            },
            
            // Moderation operations
            Operation::BanPlayer { caller_discord_id, player_discord_id, reason } => {
                ModerationOperationHandler::handle(self,
                    ModerationOperation::BanPlayer {
                        caller_discord_id, player_discord_id, reason
                    }).await
            },
            Operation::SuspendPlayer { caller_discord_id, player_discord_id, reason, duration_hours } => {
                ModerationOperationHandler::handle(self,
                    ModerationOperation::SuspendPlayer {
                        caller_discord_id, player_discord_id, reason, duration_hours
                    }).await
            },
            Operation::UnbanPlayer { caller_discord_id, player_discord_id } => {
                ModerationOperationHandler::handle(self,
                    ModerationOperation::UnbanPlayer {
                        caller_discord_id, player_discord_id
                    }).await
            },
            Operation::UnsuspendPlayer { caller_discord_id, player_discord_id } => {
                ModerationOperationHandler::handle(self,
                    ModerationOperation::UnsuspendPlayer {
                        caller_discord_id, player_discord_id
                    }).await
            },
            Operation::AssignModerator { caller_discord_id, discord_id } => {
                ModerationOperationHandler::handle(self,
                    ModerationOperation::AssignModerator {
                        caller_discord_id, discord_id
                    }).await
            },
            Operation::RemoveModerator { caller_discord_id, discord_id } => {
                ModerationOperationHandler::handle(self,
                    ModerationOperation::RemoveModerator {
                        caller_discord_id, discord_id
                    }).await
            },
            
            // Game operations
            Operation::SuspendGame { caller_discord_id, game_id, reason } => {
                GameOperationHandler::handle(self,
                    GameOperation::SuspendGame {
                        caller_discord_id, game_id, reason
                    }).await
            },
            Operation::ReactivateGame { caller_discord_id, game_id } => {
                GameOperationHandler::handle(self,
                    GameOperation::ReactivateGame {
                        caller_discord_id, game_id
                    }).await
            },
            Operation::DeprecateGame { caller_discord_id, game_id } => {
                GameOperationHandler::handle(self,
                    GameOperation::DeprecateGame {
                        caller_discord_id, game_id
                    }).await
            },
            
            // Event operations
            Operation::CreateEvent { 
                caller_discord_id, 
                game_id, 
                name, 
                description, 
                start_time, 
                end_time, 
                is_mandatory, 
                max_participants, 
                prize_pool 
            } => {
                EventOperationHandler::handle(self,
                    EventOperation::CreateEvent {
                        caller_discord_id, game_id, name, description, start_time, end_time,
                        is_mandatory, max_participants, prize_pool
                    }).await
            },
            Operation::UpdateEvent {
                caller_discord_id,
                event_id,
                name,
                description,
                start_time,
                end_time,
                is_mandatory,
                max_participants,
                prize_pool,
            } => {
                EventOperationHandler::handle(self,
                    EventOperation::UpdateEvent {
                        caller_discord_id, event_id, name, description, start_time, end_time,
                        is_mandatory, max_participants, prize_pool
                    }).await
            },
            Operation::SetEventMandatory {
                caller_discord_id,
                event_id,
                is_mandatory,
            } => {
                EventOperationHandler::handle(self,
                    EventOperation::SetEventMandatory {
                        caller_discord_id, event_id, is_mandatory
                    }).await
            },
            
            // Config operations
            Operation::UpdateScoringConfig { caller_discord_id, config } => {
                ConfigOperationHandler::handle(self,
                    ConfigOperation::UpdateScoringConfig {
                        caller_discord_id, config
                    }).await
            },
            Operation::ImportLeaderboardData {
                caller_discord_id,
                csv_data,
            } => {
                ConfigOperationHandler::handle(self,
                    ConfigOperation::ImportLeaderboardData {
                        caller_discord_id, csv_data
                    }).await
            },
        }
    }
    // ANCHOR_END: execute_operation

    async fn execute_message(&mut self, message: Message) {
        // Validate message structure first using the centralized validator
        if let Err(error) = MessageValidator::validate_message_structure(&message) {
            let error_message = format!("Invalid cross-chain message structure: {}", error);
            println!("{}", error_message);
            self.log_event(EventType::Error, error_message, Some("cross-chain".to_string()), None);
            return;
        }
        
        // Get blockchain timestamp for consistent timing
        let timestamp = self.runtime.system_time();
        
        match message {
            Message::RegisterGame { game_info } => {
                match handle_register_game_message(&mut self.state, game_info.clone(), timestamp).await {
                    Ok(result_message) => {
                        println!("{}", result_message);
                        self.log_event(
                            EventType::GameSubmitted, 
                            result_message, 
                            Some(game_info.developer_info.name.clone()), 
                            Some(game_info.id.clone())
                        );
                    }
                    Err(error) => {
                        let error_message = format!(
                            "Failed to process cross-chain game registration '{}' (ID: {}): {}", 
                            game_info.name, game_info.id, error
                        );
                        println!("{}", error_message);
                        self.log_event(
                            EventType::Error, 
                            error_message, 
                            Some(game_info.developer_info.name.clone()), 
                            Some(game_info.id.clone())
                        );
                    }
                }
            },
            
            Message::BatchEventUpdate { event_id, game_id, player_updates, final_leaderboard, update_timestamp } => {
                match handle_batch_event_update_message(
                    &mut self.state, 
                    event_id.clone(), 
                    game_id.clone(), 
                    player_updates, 
                    final_leaderboard, 
                    update_timestamp
                ).await {
                    Ok(result_message) => {
                        println!("{}", result_message);
                        self.log_event(
                            EventType::BatchEventUpdate, 
                            result_message, 
                            Some("cross-chain".to_string()), 
                            Some(event_id.clone())
                        );
                    }
                    Err(error) => {
                        let error_message = format!(
                            "Failed to process cross-chain batch update for event '{}' from game '{}': {}", 
                            event_id, game_id, error
                        );
                        println!("{}", error_message);
                        self.log_event(
                            EventType::Error, 
                            error_message, 
                            Some("cross-chain".to_string()), 
                            Some(event_id)
                        );
                    }
                }
            },
        }
    }

    // ANCHOR: store
    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
    // ANCHOR_END: store
}

#[cfg(test)]
mod tests {
    use linera_sdk::{util::BlockingWait, Contract, ContractRuntime, views::View};
    use gamehub::infrastructure::state::GameHubState;
    use gamehub::{Operation, Message};
    use gamehub::core::types::{PendingGame, DeveloperInfo};
    use super::GameHubContract;

    // ANCHOR: GameHub_test
    #[test]
    fn operation() {
        let mut runtime = ContractRuntime::new().with_application_parameters(());
        runtime.set_system_time(linera_sdk::linera_base_types::Timestamp::from(1000000));
        
        let state = GameHubState::load(runtime.root_view_storage_context())
            .blocking_wait()
            .expect("Failed to read from mock key value store");
        let mut game_hub = GameHubContract { state, runtime };

        game_hub
            .instantiate(())
            .blocking_wait();

        let operation = Operation::RegisterPlayer {
            discord_id: "1234567890".to_string(), // Valid Discord ID with 10+ characters
            username: "TestPlayer".to_string(),
            avatar_url: None,
        };
        let response = game_hub
            .execute_operation(operation)
            .blocking_wait();

        assert!(response.contains("TestPlayer"));
        // Verify player was registered by checking if they exist
        let player_exists = game_hub.state.player_exists("1234567890").blocking_wait();
        assert!(player_exists);
    }
    // ANCHOR_END: GameHub_test

    #[test]
    fn message() {
        let mut game_hub = create_and_instantiate_game_hub();
        
        // Create a proper message
        let message = Message::RegisterGame {
            game_info: PendingGame {
                id: "test-game".to_string(),
                name: "Test Game".to_string(),
                description: "A test game".to_string(),
                contract_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
                developer_info: DeveloperInfo {
                    name: "Test Dev".to_string(),
                    contact: "dev@test.com".to_string(),
                },
                created_at: linera_sdk::linera_base_types::Timestamp::from(0),
            },
        };

        game_hub
            .execute_message(message)
            .blocking_wait();
    }

    #[test]
    fn cross_application_call() {
        let mut game_hub = create_and_instantiate_game_hub();
        
        // For testing purposes, manually add "None" as an admin for this test
        // This allows the test to pass permission validation
        let none_debug = "None".to_string();
        game_hub.state.admins.insert(&none_debug).expect("Failed to add admin");

        // First, we need to create a pending game to approve
        let pending_game = PendingGame {
            id: "test-game".to_string(),
            name: "Test Game".to_string(),
            description: "A test game for approval".to_string(),
            contract_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            developer_info: DeveloperInfo {
                name: "Test Dev".to_string(),
                contact: "dev@test.com".to_string(),
            },
            created_at: linera_sdk::linera_base_types::Timestamp::from(1000000),
        };
        
        // Add the pending game to state
        game_hub.state.pending_games.insert("test-game", pending_game).expect("Failed to insert pending game");

        let operation = Operation::ApproveGame {
            caller_discord_id: "None".to_string(), // Matches the debug format added to admins
            game_id: "test-game".to_string(),
        };

        let response = game_hub
            .execute_operation(operation)
            .blocking_wait();

        assert!(response.contains("Test Game"));
        // Verify game was approved by checking if it exists in approved games
        let game_approved = game_hub.state.is_game_approved("test-game").blocking_wait();
        assert!(game_approved);
    }

    fn create_and_instantiate_game_hub() -> GameHubContract {
        let mut runtime = ContractRuntime::new().with_application_parameters(());
        runtime.set_system_time(linera_sdk::linera_base_types::Timestamp::from(1000000));
        
        let mut contract = GameHubContract {
            state: GameHubState::load(runtime.root_view_storage_context())
                .blocking_wait()
                .expect("Failed to read from mock key value store"),
            runtime,
        };

        contract
            .instantiate(())
            .blocking_wait();

        // Contract is now instantiated and ready for testing

        contract
    }
}