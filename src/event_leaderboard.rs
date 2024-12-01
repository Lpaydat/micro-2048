use async_graphql::{scalar, InputObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum EventLeaderboardAction {
    Create,
    Update,
    Delete,
}

scalar!(EventLeaderboardAction);

#[derive(Debug, Deserialize, Serialize, InputObject, Clone)]
pub struct EventLeaderboardSettings {
    pub name: String,
    pub start_time: String,
    pub end_time: String,
}
