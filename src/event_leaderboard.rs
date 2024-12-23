use async_graphql::{scalar, InputObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum EventLeaderboardAction {
    Create,
    Update,
    Delete,
    TogglePin,
}

scalar!(EventLeaderboardAction);

#[derive(Debug, Deserialize, Serialize, InputObject, Clone)]
pub struct EventLeaderboardSettings {
    pub name: String,
    pub description: Option<String>,
    pub start_time: String,
    pub end_time: String,
}
