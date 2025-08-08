// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Operation handlers for GameHub contract

pub mod player_operations;
pub mod admin_operations;
pub mod moderation_operations;
pub mod game_operations;
pub mod event_operations;
pub mod config_operations;
pub mod messaging_operations;

// Re-export all handlers
pub use player_operations::PlayerOperationHandler;
pub use admin_operations::AdminOperationHandler;
pub use moderation_operations::ModerationOperationHandler;
pub use game_operations::GameOperationHandler;
pub use event_operations::EventOperationHandler;
pub use config_operations::ConfigOperationHandler;

// Re-export messaging operation functions
pub use messaging_operations::{
    handle_request_game_registration,
    handle_send_event_update,
    handle_broadcast_leaderboard_update,
    handle_tournament_coordination,
};