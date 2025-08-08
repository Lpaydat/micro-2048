// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use linera_sdk::{linera_base_types::Timestamp, views::ViewError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Comprehensive error types for GameHub operations
#[derive(Debug, Error, Serialize, Deserialize, Clone)]
pub enum GameHubError {
    // Player errors
    #[error("Player not found")]
    PlayerNotFound,

    #[error("Player already exists")]
    PlayerAlreadyExists,

    #[error("Player is banned: {reason}")]
    PlayerBanned { reason: String },

    #[error("Player is suspended: {reason} until {until:?}")]
    PlayerSuspended {
        reason: String,
        until: Option<Timestamp>,
    },
    #[error("Player is not suspended")]
    PlayerNotSuspended,

    // Game errors
    #[error("Game not found: {game_id}")]
    GameNotFound { game_id: String },

    #[error("Game not approved")]
    GameNotApproved,

    #[error("Game already exists")]
    GameAlreadyExists,

    #[error("Invalid contract address")]
    InvalidContractAddress,

    // Event errors
    #[error("Event not found: {event_id}")]
    EventNotFound { event_id: String },

    #[error("Event not active")]
    EventNotActive,

    #[error("Event has ended")]
    EventEnded,

    #[error("Invalid time range: start time must be before end time")]
    InvalidTimeRange,

    #[error("Participation too late")]
    ParticipationTooLate,

    // Permission errors
    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Not an admin")]
    NotAdmin,

    #[error("Not a moderator")]
    NotModerator,

    // Validation errors
    #[error("Invalid input for field '{field}': {reason}")]
    InvalidInput { field: String, reason: String },

    #[error("Invalid Discord ID: {reason}")]
    InvalidDiscordId { reason: String },

    #[error("Invalid score: {reason}")]
    InvalidScore { reason: String },

    #[error("Invalid username: {reason}")]
    InvalidUsername { reason: String },

    #[error("Invalid game name: {reason}")]
    InvalidGameName { reason: String },

    #[error("Invalid event ID: {reason}")]
    InvalidEventId { reason: String },

    #[error("Invalid timestamp: {reason}")]
    InvalidTimestamp { reason: String },

    #[error("Invalid URL format: {reason}")]
    InvalidUrl { reason: String },

    #[error("Invalid email format: {reason}")]
    InvalidEmail { reason: String },

    #[error("Input too long: field '{field}' exceeds maximum length of {max_length} characters")]
    InputTooLong { field: String, max_length: usize },

    #[error("Input too short: field '{field}' requires minimum length of {min_length} characters")]
    InputTooShort { field: String, min_length: usize },

    #[error("Invalid character in field '{field}': {reason}")]
    InvalidCharacter { field: String, reason: String },

    #[error("Missing required field: {field}")]
    MissingRequiredField { field: String },

    #[error("Value out of range for field '{field}': {reason}")]
    ValueOutOfRange { field: String, reason: String },

    // Additional validation errors
    #[error("Invalid avatar URL: {reason}")]
    InvalidAvatarUrl { reason: String },

    #[error("Invalid streak: {reason}")]
    InvalidStreak { reason: String },

    #[error("Duplicate Discord IDs found: {ids:?}")]
    DuplicateDiscordIds { ids: Vec<String> },

    #[error("Invalid game ID: {reason}")]
    InvalidGameId { reason: String },

    #[error("Invalid developer name: {reason}")]
    InvalidDeveloperName { reason: String },

    #[error("Invalid developer contact: {reason}")]
    InvalidDeveloperContact { reason: String },

    #[error("Invalid contract address: {reason}")]
    InvalidContractAddressWithReason { reason: String },

    #[error("Invalid duration: {reason}")]
    InvalidDuration { reason: String },

    #[error("Invalid leaderboard size: {reason}")]
    InvalidLeaderboardSize { reason: String },

    #[error("Invalid multiplier for field '{field}': {reason}")]
    InvalidMultiplier { field: String, reason: String },

    #[error("Invalid grace period: {reason}")]
    InvalidGracePeriod { reason: String },

    #[error("Invalid booster level name: {reason}")]
    InvalidBoosterLevelName { reason: String },

    #[error("Invalid batch size for operation '{operation}': {reason}")]
    InvalidBatchSize { operation: String, reason: String },

    #[error("Value too small for field '{field}': minimum value is {min_value}")]
    ValueTooSmall { field: String, min_value: String },

    #[error("Value too large for field '{field}': maximum value is {max_value}")]
    ValueTooLarge { field: String, max_value: String },

    #[error("Invalid reason: {reason}")]
    InvalidReason { reason: String },

    #[error("Empty string found in collection '{field}' at index {index}")]
    EmptyStringInCollection { field: String, index: usize },

    #[error("Empty collection not allowed for field '{field}'")]
    EmptyCollection { field: String },

    #[error("Collection too small for field '{field}': minimum size is {min_size}, got {actual_size}")]
    CollectionTooSmall { field: String, min_size: usize, actual_size: usize },

    #[error("Collection too large for field '{field}': maximum size is {max_size}, got {actual_size}")]
    CollectionTooLarge { field: String, max_size: usize, actual_size: usize },

    // Time and system errors
    #[error("Timestamp error: {reason}")]
    TimestampError { reason: String },

    // System errors
    #[error("Database error")]
    DatabaseError,

    #[error("Message processing error")]
    MessageProcessingError,

    #[error("Configuration error")]
    ConfigurationError,

    #[error("Storage operation failed")]
    StorageError,

    #[error("Unauthorized action '{action}' attempted by user {discord_id}")]
    UnauthorizedAction { action: String, discord_id: String },

    // Admin/User role errors
    #[error("User {discord_id} already has {role} role")]
    UserAlreadyHasRole { discord_id: String, role: String },

    #[error("Cannot remove the only admin from the system")]
    CannotRemoveOnlyAdmin,

    #[error("{role} user {discord_id} not found")]
    UserNotFound { role: String, discord_id: String },

    // Cross-chain messaging errors
    #[error("Feature '{feature}' is not yet implemented")]
    NotImplemented { feature: String },

    #[error("Invalid cross-chain message: {reason}")]
    InvalidCrossChainMessage { reason: String },

    #[error("Untrusted message source")]
    UntrustedMessageSource,

    #[error("Cross-chain communication failed: {reason}")]
    CrossChainCommunicationError { reason: String },

    #[error("Invalid chain ID: {chain_id}")]
    InvalidChainId { chain_id: String },

    #[error("Message too large: size {size} bytes exceeds maximum {max_size} bytes")]
    MessageTooLarge { size: usize, max_size: usize },

    #[error("Cross-chain message timeout: message {message_id} timed out after {timeout_seconds}s")]
    MessageTimeout { message_id: String, timeout_seconds: u64 },

    #[error("Cross-chain operation rejected: {reason}")]
    CrossChainOperationRejected { reason: String },

    #[error("Invalid message payload: {reason}")]
    InvalidMessagePayload { reason: String },

    #[error("Cross-chain authentication failed: {reason}")]
    CrossChainAuthenticationFailed { reason: String },
}

impl From<ViewError> for GameHubError {
    fn from(_: ViewError) -> Self {
        GameHubError::DatabaseError
    }
}