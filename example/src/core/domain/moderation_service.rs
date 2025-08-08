use crate::core::types::*;
use crate::infrastructure::{state::GameHubState, errors::GameHubError};
use linera_sdk::linera_base_types::Timestamp;

impl GameHubState {
    // ========== PLAYER MODERATION METHODS ==========

    /// Ban player permanently
    pub async fn ban_player(&mut self, admin_discord_id: &str, player_discord_id: &str, reason: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate moderator permission
        self.validate_moderator_permission(admin_discord_id).await?;
        
        // Get player
        let mut player = self.get_player(player_discord_id).await
            .ok_or(GameHubError::PlayerNotFound)?;
        
        // Update player status
        player.status = PlayerStatus::Banned { reason: reason.to_string() };
        player.last_active = timestamp;
        
        // Save updated player
        self.players.insert(player_discord_id, player).map_err(|_| GameHubError::DatabaseError)?;
        
        // Log the action
        self.add_audit_log_entry(
            AdminAction::PlayerBanned { 
                player_id: player_discord_id.to_string(), 
                reason: reason.to_string() 
            },
            admin_discord_id,
            Some(player_discord_id),
            Some(&format!("Player banned: {}", reason)),
            timestamp,
        ).await?;
        
        Ok(())
    }

    /// Suspend player with duration
    pub async fn suspend_player(&mut self, admin_discord_id: &str, player_discord_id: &str, reason: &str, duration_hours: Option<u32>, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate permission
        self.validate_moderator_permission(admin_discord_id).await?;
        
        // Get player
        let mut player = self.get_player(player_discord_id).await
            .ok_or(GameHubError::PlayerNotFound)?;
        
        // Calculate suspension end time if duration provided
        let until = duration_hours.map(|hours| {
            let duration_micros = hours as u64 * 3600 * 1_000_000;
            Timestamp::from(timestamp.micros() + duration_micros)
        });
        
        // Update player status
        player.status = PlayerStatus::Suspended { 
            reason: reason.to_string(),
            until,
        };
        player.last_active = timestamp;
        
        // Save updated player
        self.players.insert(player_discord_id, player).map_err(|_| GameHubError::DatabaseError)?;
        
        // Log the action
        let details = match duration_hours {
            Some(hours) => format!("Player suspended for {} hours: {}", hours, reason),
            None => format!("Player suspended indefinitely: {}", reason),
        };
        
        self.add_audit_log_entry(
            AdminAction::PlayerSuspended { 
                player_id: player_discord_id.to_string(), 
                reason: reason.to_string(),
                duration_hours,
            },
            admin_discord_id,
            Some(player_discord_id),
            Some(&details),
            timestamp,
        ).await?;
        
        Ok(())
    }

    /// Unban player (set back to active)
    pub async fn unban_player(&mut self, admin_discord_id: &str, player_discord_id: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate permission
        self.validate_moderator_permission(admin_discord_id).await?;
        
        // Get player
        let mut player = self.get_player(player_discord_id).await
            .ok_or(GameHubError::PlayerNotFound)?;
        
        // Update player status back to active
        player.status = PlayerStatus::Active;
        player.last_active = timestamp;
        
        // Save updated player
        self.players.insert(player_discord_id, player).map_err(|_| GameHubError::DatabaseError)?;
        
        // Log the action
        self.add_audit_log_entry(
            AdminAction::PlayerUnbanned { player_id: player_discord_id.to_string() },
            admin_discord_id,
            Some(player_discord_id),
            Some("Player unbanned"),
            timestamp,
        ).await?;
        
        Ok(())
    }

    /// Unsuspend a player (admin/moderator operation with permission validation and audit logging)
    pub async fn unsuspend_player(&mut self, admin_discord_id: &str, player_discord_id: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate permission - admin or moderator required
        self.validate_moderator_permission(admin_discord_id).await?;
        
        // Get player and validate they exist
        let mut player = self.get_player(player_discord_id).await
            .ok_or(GameHubError::PlayerNotFound)?;
        
        // Check if player is currently suspended
        match &player.status {
            PlayerStatus::Suspended { .. } => {
                // Change status to active
                player.status = PlayerStatus::Active;
                player.last_active = timestamp;
                
                // Save updated player
                self.players.insert(player_discord_id, player).map_err(|_| GameHubError::DatabaseError)?;
                
                // Add audit log entry
                self.add_audit_log_entry(
                    AdminAction::PlayerUnsuspended { 
                        player_id: player_discord_id.to_string() 
                    },
                    admin_discord_id,
                    Some(player_discord_id),
                    Some("Player unsuspended and set to active status"),
                    timestamp,
                ).await?;
                
                Ok(())
            },
            PlayerStatus::Active => Err(GameHubError::PlayerNotSuspended),
            PlayerStatus::Banned { .. } => Err(GameHubError::PlayerBanned { reason: "Player is banned and cannot be unsuspended".to_string() }),
        }
    }
}