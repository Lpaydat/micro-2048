// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for scoring service

#[cfg(test)]
mod tests {
    use crate::core::types::*;

    fn create_test_scoring_config() -> ScoringConfig {
        ScoringConfig {
            booster_levels: vec![
                BoosterLevel {
                    required_streak: 3,
                    name: "Bronze".to_string(),
                    multiplier: 150, // 1.5x
                },
                BoosterLevel {
                    required_streak: 7,
                    name: "Silver".to_string(),
                    multiplier: 200, // 2.0x
                },
                BoosterLevel {
                    required_streak: 15,
                    name: "Gold".to_string(),
                    multiplier: 300, // 3.0x
                },
            ],
            grace_period_hours: 48,
            streak_reset_rules: StreakResetRules {
                max_missed_events: 1,
                grace_period_enabled: true,
            },
        }
    }

    #[test]
    fn test_scoring_config_default() {
        let config = ScoringConfig::default();
        
        // Verify default configuration values
        assert_eq!(config.grace_period_hours, 24);
        assert_eq!(config.streak_reset_rules.max_missed_events, 1);
        assert_eq!(config.streak_reset_rules.grace_period_enabled, true);
        assert!(!config.booster_levels.is_empty());
        assert_eq!(config.booster_levels.len(), 3);
        
        // Verify default booster levels
        assert_eq!(config.booster_levels[0].required_streak, 3);
        assert_eq!(config.booster_levels[0].multiplier, 120);
        assert_eq!(config.booster_levels[0].name, "Bronze");
        
        assert_eq!(config.booster_levels[1].required_streak, 5);
        assert_eq!(config.booster_levels[1].multiplier, 150);
        assert_eq!(config.booster_levels[1].name, "Silver");
        
        assert_eq!(config.booster_levels[2].required_streak, 10);
        assert_eq!(config.booster_levels[2].multiplier, 200);
        assert_eq!(config.booster_levels[2].name, "Gold");
    }

    #[test]
    fn test_scoring_config_custom() {
        let custom_config = create_test_scoring_config();
        
        assert_eq!(custom_config.grace_period_hours, 48);
        assert_eq!(custom_config.streak_reset_rules.max_missed_events, 1);
        assert_eq!(custom_config.streak_reset_rules.grace_period_enabled, true);
        assert_eq!(custom_config.booster_levels.len(), 3);
        
        // Verify custom booster levels
        assert_eq!(custom_config.booster_levels[0].required_streak, 3);
        assert_eq!(custom_config.booster_levels[0].multiplier, 150);
        assert_eq!(custom_config.booster_levels[0].name, "Bronze");
        
        assert_eq!(custom_config.booster_levels[1].required_streak, 7);
        assert_eq!(custom_config.booster_levels[1].multiplier, 200);
        assert_eq!(custom_config.booster_levels[1].name, "Silver");
        
        assert_eq!(custom_config.booster_levels[2].required_streak, 15);
        assert_eq!(custom_config.booster_levels[2].multiplier, 300);
        assert_eq!(custom_config.booster_levels[2].name, "Gold");
    }

    #[test]
    fn test_booster_level_creation() {
        let booster = BoosterLevel {
            required_streak: 5,
            name: "Test Level".to_string(),
            multiplier: 175, // 1.75x
        };
        
        assert_eq!(booster.required_streak, 5);
        assert_eq!(booster.name, "Test Level");
        assert_eq!(booster.multiplier, 175);
    }

    #[test]
    fn test_streak_reset_rules_default() {
        let rules = StreakResetRules::default();
        
        assert_eq!(rules.max_missed_events, 1);
        assert_eq!(rules.grace_period_enabled, true);
    }

    #[test]
    fn test_streak_reset_rules_custom() {
        let rules = StreakResetRules {
            max_missed_events: 3,
            grace_period_enabled: false,
        };
        
        assert_eq!(rules.max_missed_events, 3);
        assert_eq!(rules.grace_period_enabled, false);
    }

    #[test]
    fn test_scoring_config_clone() {
        let original = create_test_scoring_config();
        let cloned = original.clone();
        
        assert_eq!(original.grace_period_hours, cloned.grace_period_hours);
        assert_eq!(original.booster_levels.len(), cloned.booster_levels.len());
        assert_eq!(original.streak_reset_rules.max_missed_events, cloned.streak_reset_rules.max_missed_events);
    }

    #[test]
    fn test_booster_levels_sorted_by_requirement() {
        let config = ScoringConfig::default();
        
        // Verify booster levels are in ascending order by required_streak
        for i in 1..config.booster_levels.len() {
            assert!(config.booster_levels[i].required_streak > config.booster_levels[i-1].required_streak);
        }
    }

    #[test]
    fn test_multiplier_calculation_logic() {
        let config = create_test_scoring_config();
        
        // Test points calculation logic (simulating the actual method logic)
        let base_points = 100u64;
        
        // Test no streak (should return base points)
        let streak_level = 0u32;
        let applicable_level = config.booster_levels.iter()
            .filter(|level| streak_level >= level.required_streak)
            .max_by_key(|level| level.required_streak);
        
        let multiplier = applicable_level.map(|level| level.multiplier).unwrap_or(100);
        let boosted_points = (base_points * multiplier) / 100;
        assert_eq!(boosted_points, 100); // No boost
        
        // Test bronze level (streak 5)
        let streak_level = 5u32;
        let applicable_level = config.booster_levels.iter()
            .filter(|level| streak_level >= level.required_streak)
            .max_by_key(|level| level.required_streak);
        
        let multiplier = applicable_level.map(|level| level.multiplier).unwrap_or(100);
        let boosted_points = (base_points * multiplier) / 100;
        assert_eq!(boosted_points, 150); // Bronze boost (1.5x)
        
        // Test silver level (streak 10)
        let streak_level = 10u32;
        let applicable_level = config.booster_levels.iter()
            .filter(|level| streak_level >= level.required_streak)
            .max_by_key(|level| level.required_streak);
        
        let multiplier = applicable_level.map(|level| level.multiplier).unwrap_or(100);
        let boosted_points = (base_points * multiplier) / 100;
        assert_eq!(boosted_points, 200); // Silver boost (2.0x)
        
        // Test gold level (streak 20)
        let streak_level = 20u32;
        let applicable_level = config.booster_levels.iter()
            .filter(|level| streak_level >= level.required_streak)
            .max_by_key(|level| level.required_streak);
        
        let multiplier = applicable_level.map(|level| level.multiplier).unwrap_or(100);
        let boosted_points = (base_points * multiplier) / 100;
        assert_eq!(boosted_points, 300); // Gold boost (3.0x)
    }

    #[test]
    fn test_grace_period_calculation_logic() {
        let grace_period_hours = 48u32;
        
        // Simulate grace period timing logic
        let base_time_micros = 1000000000000u64; // Example timestamp
        let grace_period_micros = (grace_period_hours as u64) * 3600 * 1_000_000;
        
        // Test within grace period (24 hours later)
        let later_time_micros = base_time_micros + (24 * 3600 * 1_000_000u64);
        let time_diff = later_time_micros - base_time_micros;
        assert!(time_diff <= grace_period_micros);
        
        // Test outside grace period (72 hours later)
        let later_time_micros = base_time_micros + (72 * 3600 * 1_000_000u64);
        let time_diff = later_time_micros - base_time_micros;
        assert!(time_diff > grace_period_micros);
    }

    #[test]
    fn test_scoring_config_serialization() {
        let config = create_test_scoring_config();
        
        // Test that the config can be serialized (this ensures serde attributes work)
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
        
        // Test deserialization
        let deserialized: Result<ScoringConfig, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
        
        let deserialized_config = deserialized.unwrap();
        assert_eq!(config.grace_period_hours, deserialized_config.grace_period_hours);
        assert_eq!(config.booster_levels.len(), deserialized_config.booster_levels.len());
    }

    #[test]
    fn test_edge_cases_multiplier_calculation() {
        let config = create_test_scoring_config();
        
        // Test with 0 base points
        let base_points = 0u64;
        let streak_level = 10u32;
        let applicable_level = config.booster_levels.iter()
            .filter(|level| streak_level >= level.required_streak)
            .max_by_key(|level| level.required_streak);
        
        let multiplier = applicable_level.map(|level| level.multiplier).unwrap_or(100);
        let boosted_points = (base_points * multiplier) / 100;
        assert_eq!(boosted_points, 0);
        
        // Test with very high streak (should use highest available level)
        let base_points = 100u64;
        let streak_level = 1000u32;
        let applicable_level = config.booster_levels.iter()
            .filter(|level| streak_level >= level.required_streak)
            .max_by_key(|level| level.required_streak);
        
        let multiplier = applicable_level.map(|level| level.multiplier).unwrap_or(100);
        let boosted_points = (base_points * multiplier) / 100;
        assert_eq!(boosted_points, 300); // Should use gold level (highest)
    }

    #[test]
    fn test_empty_booster_levels_fallback() {
        let config = ScoringConfig {
            booster_levels: vec![], // Empty booster levels
            grace_period_hours: 24,
            streak_reset_rules: StreakResetRules::default(),
        };
        
        // Test that empty booster levels fall back to 100 (no multiplier)
        let base_points = 100u64;
        let streak_level = 10u32;
        let applicable_level = config.booster_levels.iter()
            .filter(|level| streak_level >= level.required_streak)
            .max_by_key(|level| level.required_streak);
        
        let multiplier = applicable_level.map(|level| level.multiplier).unwrap_or(100);
        let boosted_points = (base_points * multiplier) / 100;
        assert_eq!(boosted_points, 100); // No boost applied
    }
}