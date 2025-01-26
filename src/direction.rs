use async_graphql::scalar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

scalar!(Direction);
