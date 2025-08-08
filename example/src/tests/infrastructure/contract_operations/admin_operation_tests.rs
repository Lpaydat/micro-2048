// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Admin operation tests
//! 
//! Tests for administrative contract operations including admin/moderator management
//! and scoring configuration updates.

#[cfg(test)]
mod tests {
    use crate::infrastructure::operations::Operation;

    #[test]
    fn test_add_admin_operation() {
        // Test admin addition operation
        let operation = Operation::AddAdmin {
            caller_discord_id: "987654321098765432".to_string(),
            discord_id: "123456789012345678".to_string(),
        };

        // Verify operation structure
        match operation {
            Operation::AddAdmin { caller_discord_id, discord_id } => {
                assert_eq!(caller_discord_id, "987654321098765432");
                assert_eq!(discord_id, "123456789012345678");
            }
            _ => panic!("Expected AddAdmin operation"),
        }
    }

    #[test]
    fn test_remove_admin_operation() {
        // Test admin removal operation
        let operation = Operation::RemoveAdmin {
            caller_discord_id: "123456789012345678".to_string(),
            discord_id: "987654321098765432".to_string(),
        };

        // Verify operation structure
        match operation {
            Operation::RemoveAdmin { caller_discord_id, discord_id } => {
                assert_eq!(caller_discord_id, "123456789012345678");
                assert_eq!(discord_id, "987654321098765432");
            }
            _ => panic!("Expected RemoveAdmin operation"),
        }
    }

    #[test]
    fn test_assign_moderator_operation() {
        // Test moderator assignment operation
        let operation = Operation::AssignModerator {
            caller_discord_id: "123456789012345678".to_string(),
            discord_id: "555666777888999000".to_string(),
        };

        // Verify operation structure
        match operation {
            Operation::AssignModerator { caller_discord_id, discord_id } => {
                assert_eq!(caller_discord_id, "123456789012345678");
                assert_eq!(discord_id, "555666777888999000");
            }
            _ => panic!("Expected AssignModerator operation"),
        }
    }

    #[test]
    fn test_remove_moderator_operation() {
        // Test moderator removal operation
        let operation = Operation::RemoveModerator {
            caller_discord_id: "123456789012345678".to_string(),
            discord_id: "111222333444555666".to_string(),
        };

        // Verify operation structure
        match operation {
            Operation::RemoveModerator { caller_discord_id, discord_id } => {
                assert_eq!(caller_discord_id, "123456789012345678");
                assert_eq!(discord_id, "111222333444555666");
            }
            _ => panic!("Expected RemoveModerator operation"),
        }
    }

    #[test]
    fn test_update_scoring_config_operation() {
        // Test scoring configuration update operation
        use crate::core::types::{ScoringConfig, BoosterLevel, StreakResetRules};
        
        let config = ScoringConfig {
            booster_levels: vec![
                BoosterLevel {
                    required_streak: 5,
                    multiplier: 150, // 1.5x multiplier
                    name: "Bronze".to_string(),
                },
                BoosterLevel {
                    required_streak: 10,
                    multiplier: 200, // 2.0x multiplier
                    name: "Silver".to_string(),
                },
            ],
            grace_period_hours: 24,
            streak_reset_rules: StreakResetRules {
                max_missed_events: 2,
                grace_period_enabled: true,
            },
        };

        let operation = Operation::UpdateScoringConfig {
            caller_discord_id: "123456789012345678".to_string(),
            config: config.clone(),
        };

        // Verify operation structure
        match operation {
            Operation::UpdateScoringConfig { caller_discord_id, config: op_config } => {
                assert_eq!(caller_discord_id, "123456789012345678");
                assert_eq!(op_config.booster_levels.len(), 2);
                assert_eq!(op_config.booster_levels[0].required_streak, 5);
                assert_eq!(op_config.booster_levels[0].multiplier, 150);
                assert_eq!(op_config.booster_levels[0].name, "Bronze");
                assert_eq!(op_config.booster_levels[1].required_streak, 10);
                assert_eq!(op_config.booster_levels[1].multiplier, 200);
                assert_eq!(op_config.booster_levels[1].name, "Silver");
                assert_eq!(op_config.grace_period_hours, 24);
                assert_eq!(op_config.streak_reset_rules.max_missed_events, 2);
                assert!(op_config.streak_reset_rules.grace_period_enabled);
            }
            _ => panic!("Expected UpdateScoringConfig operation"),
        }
    }
}