use crate::core::types::*;
use crate::infrastructure::{state::GameHubState, errors::GameHubError};
use linera_sdk::linera_base_types::Timestamp;

impl GameHubState {
    // ========== SCORING CONFIGURATION METHODS ==========

    /// Get scoring configuration
    pub async fn get_scoring_config(&self) -> ScoringConfig {
        self.scoring_config.get().clone()
    }

    /// Update scoring configuration
    pub async fn update_scoring_config(&mut self, admin_discord_id: &str, config: ScoringConfig, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate permission
        self.validate_admin_permission(admin_discord_id).await?;
        
        // Update configuration
        self.scoring_config.set(config);
        
        // Log the action
        self.add_audit_log_entry(
            AdminAction::ScoringConfigUpdated,
            admin_discord_id,
            None,
            Some("Scoring configuration updated"),
            timestamp,
        ).await?;
        
        Ok(())
    }

    // ========== POINTS CALCULATION METHODS ==========

    /// Calculate points with streak booster
    pub async fn calculate_points_with_streak_booster(&self, base_points: u64, streak_level: u32) -> u64 {
        let multiplier = self.get_streak_multiplier(streak_level).await;
        (base_points * multiplier) / 100 // Multiplier is stored as * 100
    }

    /// Get streak multiplier for streak level
    pub async fn get_streak_multiplier(&self, streak_level: u32) -> u64 {
        let config = self.get_scoring_config().await;
        
        // Find the highest applicable booster level
        let mut applicable_multiplier = 100; // Default 1.0x multiplier
        
        for booster in &config.booster_levels {
            if streak_level >= booster.required_streak {
                applicable_multiplier = booster.multiplier;
            }
        }
        
        applicable_multiplier
    }

    // ========== STREAK CALCULATION METHODS ==========

    /// Calculate new streak with timing rules
    pub async fn calculate_new_streak(&self, player_discord_id: &str, last_participation: Timestamp, new_participation: Timestamp) -> u32 {
        let current_streak = if let Some(player) = self.get_player(player_discord_id).await {
            player.participation_streak
        } else {
            0
        };
        
        let config = self.get_scoring_config().await;
        
        // Check if within grace period
        if self.is_within_grace_period(last_participation, new_participation, config.grace_period_hours).await {
            current_streak + 1
        } else {
            // Check streak reset rules
            if config.streak_reset_rules.grace_period_enabled {
                1 // Start new streak
            } else {
                0 // Reset to 0 if no grace period
            }
        }
    }

    /// Check if within grace period
    pub async fn is_within_grace_period(&self, last_participation: Timestamp, new_participation: Timestamp, grace_period_hours: u32) -> bool {
        let time_diff_micros = new_participation.micros() - last_participation.micros();
        let grace_period_micros = grace_period_hours as u64 * 60 * 60 * 1_000_000;
        
        time_diff_micros <= grace_period_micros
    }
}