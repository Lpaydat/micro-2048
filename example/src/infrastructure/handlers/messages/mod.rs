// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Cross-chain message handlers module
//! 
//! This module provides handlers for all types of cross-chain messages
//! that GameHub can receive from external game contracts and other chains.
//! It follows a clean separation of concerns with specialized handlers
//! for different message types and centralized validation.

pub mod game_registration;
pub mod batch_updates;
pub mod validation;

// Re-export main handler functions for easy access
pub use game_registration::handle_register_game_message;
pub use batch_updates::handle_batch_event_update_message;
pub use validation::MessageValidator;