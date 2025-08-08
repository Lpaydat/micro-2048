// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use async_graphql::InputObject;

/// Scoring configuration with booster levels and streak rules
#[derive(Debug, Serialize, Deserialize, Clone, InputObject)]
pub struct ScoringConfig {
    pub booster_levels: Vec<BoosterLevel>,
    pub grace_period_hours: u32,
    pub streak_reset_rules: StreakResetRules,
}

/// Booster level for streak multipliers
#[derive(Debug, Serialize, Deserialize, Clone, InputObject)]
pub struct BoosterLevel {
    pub required_streak: u32,
    pub multiplier: u64, // Multiplier * 100 (e.g., 150 = 1.5x multiplier)
    pub name: String,
}

/// Streak reset rules for participation management
#[derive(Debug, Serialize, Deserialize, Clone, InputObject)]
pub struct StreakResetRules {
    pub max_missed_events: u32,
    pub grace_period_enabled: bool,
}

impl Default for StreakResetRules {
    fn default() -> Self {
        Self {
            max_missed_events: 1,
            grace_period_enabled: true,
        }
    }
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            booster_levels: vec![
                BoosterLevel {
                    required_streak: 3,
                    multiplier: 120, // 1.2x multiplier
                    name: "Bronze".to_string(),
                },
                BoosterLevel {
                    required_streak: 5,
                    multiplier: 150, // 1.5x multiplier
                    name: "Silver".to_string(),
                },
                BoosterLevel {
                    required_streak: 10,
                    multiplier: 200, // 2.0x multiplier
                    name: "Gold".to_string(),
                },
            ],
            grace_period_hours: 24,
            streak_reset_rules: StreakResetRules::default(),
        }
    }
}