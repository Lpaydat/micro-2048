//! Smart contract implementation with DDD architecture and improved error handling.

use linera_sdk::linera_base_types::{ChainId, Timestamp};
use crate::{
    core::{validators::*, value_objects::*},
    infrastructure::state::StateError,
};

/// Operations that can be performed on the contract
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Operation {
    // Participant operations
    RegisterParticipant {
        username: String,
    },
    UpdateParticipantProfile {
        display_name: Option<String>,
        avatar_hash: Option<String>,
    },

    // Game session operations
    CreateGameSession {
        game_variant: GameVariant,
        competition_id: Option<CompetitionId>,
    },
    MakeMove {
        session_id: GameSessionId,
        direction: Direction,
    },
    AbandonGameSession {
        session_id: GameSessionId,
    },

    // Competition operations
    CreateCompetition {
        title: String,
        format: CompetitionFormat,
        start_time: Timestamp,
        end_time: Timestamp,
        leaderboard_chain: ChainId,
    },
    JoinCompetition {
        competition_id: CompetitionId,
    },

    // Administrative operations
    BanParticipant {
        participant_id: ParticipantId,
        reason: BanReason,
    },
    SuspendParticipant {
        participant_id: ParticipantId,
        until: Timestamp,
    },
    ReactivateParticipant {
        participant_id: ParticipantId,
    },
}

/// Messages for cross-chain communication
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Message {
    // Cross-chain coordination messages
    CrossChainMessage(CrossChainMessage),
}

// Simplified implementation methods for compilation

/// Response types for operations
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum OperationResponse {
    Success,
    ParticipantRegistered { participant_id: ParticipantId },
    GameSessionCreated { session_id: GameSessionId },
    MoveProcessed { session_id: GameSessionId, score_delta: u64, game_ended: bool },
    CompetitionCreated { competition_id: CompetitionId },
    GraphQLResponse(Vec<u8>),
}

/// Contract errors - simplified for compilation
#[derive(Debug)]
pub enum ContractError {
    StateError(StateError),
    ValidationError(ValidationError),
    UnexpectedError(String),
}

impl std::fmt::Display for ContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractError::StateError(err) => write!(f, "State error: {}", err),
            ContractError::ValidationError(err) => write!(f, "Validation error: {}", err),
            ContractError::UnexpectedError(msg) => write!(f, "Unexpected error: {}", msg),
        }
    }
}

impl std::error::Error for ContractError {}

/// Contract ABI definition
pub struct Game2048Abi;

impl linera_sdk::abi::ContractAbi for Game2048Abi {
    type Operation = Operation;
    type Response = OperationResponse;
}

impl linera_sdk::abi::ServiceAbi for Game2048Abi {
    type Query = async_graphql::Request;
    type QueryResponse = async_graphql::Response;
}