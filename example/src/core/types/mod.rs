// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Domain types for GameHub
//! 
//! This module contains all the core data structures and enums used throughout
//! the GameHub application, organized by domain.

pub mod player;
pub mod game;
pub mod event;
pub mod leaderboard;
pub mod scoring;
pub mod audit;
pub mod batch;
pub mod admin;
pub mod streak;
pub mod points;
pub mod governance;

// Re-export all types for convenience
pub use player::*;
pub use game::*;
pub use event::*;
pub use leaderboard::*;
pub use scoring::*;
pub use audit::*;
pub use batch::*;
pub use admin::*;
pub use streak::*;
pub use points::*;
pub use governance::*;