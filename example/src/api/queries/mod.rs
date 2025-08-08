// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! GraphQL query modules

pub mod player_queries;
pub mod leaderboard_queries;
pub mod game_queries;
pub mod event_queries;
pub mod admin_queries;
pub mod analytics_queries;
pub mod config_queries;

pub use player_queries::PlayerQueries;
pub use leaderboard_queries::LeaderboardQueries;
pub use game_queries::GameQueries;
pub use event_queries::EventQueries;
pub use admin_queries::AdminQueries;
pub use analytics_queries::AnalyticsQueries;
pub use config_queries::ConfigQueries;