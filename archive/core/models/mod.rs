//! Domain models representing the core business entities.
//! These models use improved naming conventions and are designed for extensibility.

use crate::core::value_objects::*;
use linera_sdk::linera_base_types::{ChainId, Timestamp};
use serde::{Deserialize, Serialize};

pub mod competition;
pub mod game_session;
pub mod participant;

// Re-export all model types
pub use competition::*;
pub use game_session::*;
pub use participant::*;
