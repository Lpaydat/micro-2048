use crate::core::types::*;
use crate::infrastructure::state::GameHubState;

impl GameHubState {
    // ========== LEADERBOARD PROCESSING METHODS ==========

    /// Update leaderboard with ranking
    pub async fn update_leaderboard(&mut self, event_id: &str, entries: Vec<LeaderboardEntry>) -> Result<(), String> {
        let processed_entries = self.process_leaderboard_entries(entries).await;
        self.leaderboards.insert(event_id, processed_entries).map_err(|e| format!("Failed to update leaderboard: {}", e))?;
        
        // Update player ranks
        self.update_player_ranks_from_leaderboard(event_id).await?;
        
        Ok(())
    }

    /// Process leaderboard entries with proper ranking
    pub async fn process_leaderboard_entries(&self, mut entries: Vec<LeaderboardEntry>) -> Vec<LeaderboardEntry> {
        // Sort by score (descending) then by participation timestamp (ascending for tie-breaking)
        entries.sort_by(|a, b| {
            b.score.cmp(&a.score)
                .then_with(|| a.participation_data.participation_timestamp.cmp(&b.participation_data.participation_timestamp))
        });
        
        // Assign ranks (handle ties)
        let mut current_rank = 1u32;
        let mut last_score = None;
        
        for (index, entry) in entries.iter_mut().enumerate() {
            if let Some(prev_score) = last_score {
                if entry.score != prev_score {
                    current_rank = (index + 1) as u32;
                }
            }
            
            entry.rank = current_rank;
            last_score = Some(entry.score);
        }
        
        entries
    }

    /// Update player ranks from leaderboard
    pub async fn update_player_ranks_from_leaderboard(&mut self, event_id: &str) -> Result<(), String> {
        let leaderboard = self.leaderboards.get(event_id).await.ok().flatten().unwrap_or_default();
        
        for entry in leaderboard {
            if let Some(mut player) = self.get_player(&entry.player_discord_id).await {
                player.current_rank = Some(entry.rank);
                self.players.insert(&entry.player_discord_id, player).map_err(|e| format!("Failed to update player rank: {}", e))?;
            }
        }
        
        Ok(())
    }

    /// Calculate comprehensive leaderboard with streak bonuses
    pub async fn calculate_comprehensive_leaderboard(&self, _event_id: &str, player_updates: Vec<PlayerEventUpdate>) -> Vec<LeaderboardEntry> {
        let mut entries = Vec::new();
        
        for update in player_updates {
            if let Some(player) = self.get_player(&update.discord_id).await {
                let boosted_score = self.calculate_points_with_streak_booster(update.score, player.participation_streak).await;
                
                let participation_data = ParticipationData {
                    streak_level: player.participation_streak,
                    streak_multiplier: self.get_streak_multiplier(player.participation_streak).await,
                    total_points_earned: boosted_score,
                    participation_timestamp: update.participation_timestamp,
                };
                
                entries.push(LeaderboardEntry {
                    player_discord_id: update.discord_id,
                    score: boosted_score,
                    rank: 0, // Will be set by process_leaderboard_entries
                    participation_data,
                });
            } else {
                // Handle unregistered players with potential streak
                let potential_streak = if let Some(pending_data) = self.get_pending_data(&update.discord_id).await {
                    self.calculate_potential_streak_from_pending(&pending_data).await
                } else {
                    0
                };
                
                let boosted_score = self.calculate_points_with_streak_booster(update.score, potential_streak).await;
                
                let participation_data = ParticipationData {
                    streak_level: potential_streak,
                    streak_multiplier: self.get_streak_multiplier(potential_streak).await,
                    total_points_earned: boosted_score,
                    participation_timestamp: update.participation_timestamp,
                };
                
                entries.push(LeaderboardEntry {
                    player_discord_id: update.discord_id,
                    score: boosted_score,
                    rank: 0, // Will be set by process_leaderboard_entries
                    participation_data,
                });
            }
        }
        
        self.process_leaderboard_entries(entries).await
    }

    /// Calculate potential streak from pending data
    pub async fn calculate_potential_streak_from_pending(&self, pending_data: &PendingPlayerData) -> u32 {
        // This is similar to calculate_streak_from_pending but considers potential future streak
        self.calculate_streak_from_pending(pending_data).await
    }

    /// Get leaderboard for event
    pub async fn get_leaderboard(&self, event_id: &str) -> Vec<LeaderboardEntry> {
        self.leaderboards.get(event_id).await.ok().flatten().unwrap_or_default()
    }
}