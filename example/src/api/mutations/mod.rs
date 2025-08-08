// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! GraphQL mutation modules

pub mod player_mutations;
pub mod moderation_mutations;
pub mod game_mutations;
pub mod admin_mutations;
pub mod config_mutations;

pub use player_mutations::PlayerMutations;
pub use moderation_mutations::ModerationMutations;
pub use game_mutations::GameMutations;
pub use admin_mutations::AdminMutations;
pub use config_mutations::ConfigMutations;