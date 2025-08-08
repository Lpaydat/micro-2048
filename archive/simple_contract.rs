#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};

use self::state::{Game2048State, ParticipantInfo, GameSession};
use game2048::{Operation, Message, OperationResponse, Game2048Abi, GameVariant};

pub struct Game2048Contract {
    state: Game2048State,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(Game2048Contract);

impl WithContractAbi for Game2048Contract {
    type Abi = Game2048Abi;
}

impl Contract for Game2048Contract {
    type Message = Message;
    type Parameters = ();
    type InstantiationArgument = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = Game2048State::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        Game2048Contract { state, runtime }
    }

    async fn instantiate(&mut self, _argument: Self::InstantiationArgument) {
        // Initialize the contract
        self.runtime.application_parameters();
        
        // Initialize counters
        self.state.participants_count.set(0);
        self.state.sessions_count.set(0);
    }

    async fn execute_operation(&mut self, operation: Operation) -> OperationResponse {
        match operation {
            Operation::RegisterParticipant { username } => {
                let participant_id = *self.state.participants_count.get() + 1;
                self.state.participants_count.set(participant_id);
                
                let participant = ParticipantInfo {
                    participant_id,
                    username: username.clone(),
                    chain_id: self.runtime.chain_id().to_string(),
                    registration_time: self.runtime.system_time(),
                    total_sessions: 0,
                    best_score: 0,
                };
                
                self.state.participants.insert(&username, participant).unwrap();
                OperationResponse::ParticipantRegistered { participant_id }
            }
            Operation::CreateGameSession { game_variant, competition_id: _ } => {
                let session_id = *self.state.sessions_count.get() + 1;
                self.state.sessions_count.set(session_id);
                
                // Create a simple game session
                let session = GameSession {
                    session_id,
                    participant_id: 1, // Simplified - would get from authenticated user
                    game_variant,
                    board_state: 0, // Empty board
                    score: 0,
                    move_count: 0,
                    highest_tile: 2,
                    is_ended: false,
                    created_at: self.runtime.system_time(),
                    last_move_at: None,
                };
                
                self.state.game_sessions.insert(&session_id, session).unwrap();
                OperationResponse::GameSessionCreated { session_id }
            }
            Operation::MakeMove { session_id, direction: _ } => {
                // Simplified move processing
                if let Some(mut session) = self.state.game_sessions.get(&session_id).await.unwrap() {
                    session.move_count += 1;
                    session.score += 4; // Simple score increment
                    session.last_move_at = Some(self.runtime.system_time());
                    
                    self.state.game_sessions.insert(&session_id, session).unwrap();
                    OperationResponse::MoveProcessed { 
                        session_id, 
                        score_delta: 4, 
                        game_ended: false 
                    }
                } else {
                    OperationResponse::Success
                }
            }
        }
    }

    async fn execute_message(&mut self, message: Message) {
        match message {
            Message::CrossChainMessage(_) => {
                // Handle cross-chain message
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}