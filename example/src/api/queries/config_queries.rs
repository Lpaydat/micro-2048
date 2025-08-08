// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Configuration-related GraphQL queries

use std::sync::Arc;
use async_graphql::{Object, Result};
use crate::{
    api::graphql_types::ScoringConfigObject,
    infrastructure::state::GameHubState,
};

/// Configuration query resolvers
#[derive(Clone)]
pub struct ConfigQueries {
    pub state: Arc<GameHubState>,
}

#[Object]
impl ConfigQueries {
    /// Get current scoring configuration
    async fn scoring_config(&self) -> Result<ScoringConfigObject> {
        let config = self.state.get_scoring_config().await;
        
        // Extract booster level information from the Vec<BoosterLevel>
        let bronze_booster = config.booster_levels.iter().find(|b| b.name == "Bronze");
        let silver_booster = config.booster_levels.iter().find(|b| b.name == "Silver");
        let gold_booster = config.booster_levels.iter().find(|b| b.name == "Gold");
        
        Ok(ScoringConfigObject {
            base_points_per_event: 100, // Default base points - would need to be configurable
            streak_bonus_multiplier: 110, // Default 1.1x - would need to be configurable
            bronze_booster_threshold: bronze_booster.map(|b| b.required_streak).unwrap_or(3),
            silver_booster_threshold: silver_booster.map(|b| b.required_streak).unwrap_or(5), 
            gold_booster_threshold: gold_booster.map(|b| b.required_streak).unwrap_or(10),
            bronze_multiplier: bronze_booster.map(|b| b.multiplier).unwrap_or(120),
            silver_multiplier: silver_booster.map(|b| b.multiplier).unwrap_or(150),
            gold_multiplier: gold_booster.map(|b| b.multiplier).unwrap_or(200),
            streak_grace_period_hours: config.grace_period_hours,
            max_streak_multiplier: 300, // Default max 3.0x - would need to be configurable
        })
    }
}