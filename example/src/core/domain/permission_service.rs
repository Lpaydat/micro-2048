use crate::core::types::*;
use crate::infrastructure::{state::GameHubState, errors::GameHubError};
use linera_sdk::linera_base_types::Timestamp;


impl GameHubState {
    // ========== PERMISSION CHECK METHODS ==========

    /// Check if user is an admin
    pub async fn is_admin(&self, discord_id: &str) -> bool {
        self.admins.contains(discord_id).await.unwrap_or(false)
    }

    /// Check if user is moderator or admin
    pub async fn is_moderator_or_admin(&self, discord_id: &str) -> bool {
        self.is_admin(discord_id).await || self.moderators.contains(discord_id).await.unwrap_or(false)
    }

    /// Check if user has admin privileges (alias for is_admin)
    pub async fn has_admin_privileges(&self, discord_id: &str) -> bool {
        self.is_admin(discord_id).await
    }

    /// Validate admin permission
    pub async fn validate_admin_permission(&self, discord_id: &str) -> Result<(), GameHubError> {
        if !self.is_admin(discord_id).await {
            return Err(GameHubError::InsufficientPermissions);
        }
        Ok(())
    }

    /// Validate moderator permission
    pub async fn validate_moderator_permission(&self, discord_id: &str) -> Result<(), GameHubError> {
        if !self.is_moderator_or_admin(discord_id).await {
            return Err(GameHubError::InsufficientPermissions);
        }
        Ok(())
    }

    // ========== ADMIN MANAGEMENT METHODS ==========

    /// Add admin role
    pub async fn add_admin(&mut self, admin_discord_id: &str, new_admin_discord_id: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate admin permission
        self.validate_admin_permission(admin_discord_id).await?;
        
        // Add new admin
        self.admins.insert(new_admin_discord_id).map_err(|_| GameHubError::DatabaseError)?;
        
        // Add audit log entry
        self.add_audit_log_entry(
            AdminAction::AdminAdded { admin_id: new_admin_discord_id.to_string() },
            admin_discord_id,
            Some(new_admin_discord_id),
            Some("Admin role granted"),
            timestamp,
        ).await?;
        
        Ok(())
    }

    /// Remove admin role
    pub async fn remove_admin(&mut self, admin_discord_id: &str, target_admin_discord_id: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate admin permission
        self.validate_admin_permission(admin_discord_id).await?;
        
        // Prevent self-removal (simple check)
        if admin_discord_id == target_admin_discord_id {
            return Err(GameHubError::InvalidInput { 
                field: "target_admin".to_string(), 
                reason: "Cannot remove yourself as admin".to_string() 
            });
        }
        
        // Remove admin
        self.admins.remove(target_admin_discord_id).map_err(|_| GameHubError::DatabaseError)?;
        
        // Add audit log entry
        self.add_audit_log_entry(
            AdminAction::AdminRemoved { admin_id: target_admin_discord_id.to_string() },
            admin_discord_id,
            Some(target_admin_discord_id),
            Some("Admin role revoked"),
            timestamp,
        ).await?;
        
        Ok(())
    }

    // ========== MODERATOR MANAGEMENT METHODS ==========

    /// Assign moderator role
    pub async fn assign_moderator(&mut self, admin_discord_id: &str, moderator_discord_id: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate admin permission
        self.validate_admin_permission(admin_discord_id).await?;
        
        // Add moderator
        self.moderators.insert(moderator_discord_id).map_err(|_| GameHubError::DatabaseError)?;
        
        // Add audit log entry
        self.add_audit_log_entry(
            AdminAction::ModeratorAssigned { moderator_id: moderator_discord_id.to_string() },
            admin_discord_id,
            Some(moderator_discord_id),
            Some("Moderator role granted"),
            timestamp,
        ).await?;
        
        Ok(())
    }

    /// Remove moderator role
    pub async fn remove_moderator(&mut self, admin_discord_id: &str, moderator_discord_id: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate admin permission
        self.validate_admin_permission(admin_discord_id).await?;
        
        // Remove moderator
        self.moderators.remove(moderator_discord_id).map_err(|_| GameHubError::DatabaseError)?;
        
        // Add audit log entry
        self.add_audit_log_entry(
            AdminAction::ModeratorRemoved { moderator_id: moderator_discord_id.to_string() },
            admin_discord_id,
            Some(moderator_discord_id),
            Some("Moderator role revoked"),
            timestamp,
        ).await?;
        
        Ok(())
    }

    // ========== ROLE LISTING METHODS ==========

    /// Get all admins using SetView iteration
    pub async fn get_all_admins(&self) -> Vec<String> {
        // Use indices() method to iterate through SetView
        match self.admins.indices().await {
            Ok(admin_ids) => admin_ids,
            Err(_) => {
                // Return empty vector on storage errors to maintain API contract
                Vec::new()
            }
        }
    }

    /// Get all moderators using SetView iteration
    pub async fn get_all_moderators(&self) -> Vec<String> {
        // Use indices() method to iterate through SetView
        match self.moderators.indices().await {
            Ok(moderator_ids) => moderator_ids,
            Err(_) => {
                // Return empty vector on storage errors to maintain API contract
                Vec::new()
            }
        }
    }

    // ========== AUDIT LOGGING METHODS ==========

    /// Get all audit log entries using MapView iteration, sorted chronologically
    pub async fn get_audit_log_entries(&self) -> Vec<AuditLogEntry> {
        // Use indices() method to iterate through MapView
        let log_ids = match self.audit_log.indices().await {
            Ok(indices) => indices,
            Err(_) => {
                // Return empty vector on storage errors to maintain API contract
                return Vec::new();
            }
        };
        
        let mut entries = Vec::new();
        for log_id in log_ids {
            if let Ok(Some(entry)) = self.audit_log.get(&log_id).await {
                entries.push(entry);
            }
        }
        
        // Sort entries by timestamp (chronological order)
        entries.sort_by(|a, b| a.timestamp.micros().cmp(&b.timestamp.micros()));
        entries
    }

    /// Get audit log entries by performer using MapView iteration and filtering
    pub async fn get_audit_log_entries_by_performer(&self, performer: &str) -> Vec<AuditLogEntry> {
        // Get all audit log entries first
        let all_entries = self.get_audit_log_entries().await;
        
        // Filter entries by performer
        all_entries
            .into_iter()
            .filter(|entry| entry.performed_by == performer)
            .collect()
    }

    /// Get audit log entries by target using MapView iteration and filtering
    pub async fn get_audit_log_entries_by_target(&self, target: &str) -> Vec<AuditLogEntry> {
        // Get all audit log entries first
        let all_entries = self.get_audit_log_entries().await;
        
        // Filter entries by target
        all_entries
            .into_iter()
            .filter(|entry| entry.target.as_ref() == Some(&target.to_string()))
            .collect()
    }

    /// Add audit log entry
    pub async fn add_audit_log_entry(
        &mut self,
        action: AdminAction,
        performed_by: &str,
        target: Option<&str>,
        details: Option<&str>,
        timestamp: Timestamp,
    ) -> Result<(), GameHubError> {
        // Generate unique log ID using action type and timestamp
        let action_type = match &action {
            AdminAction::AdminAdded { .. } => "admin_added",
            AdminAction::AdminRemoved { .. } => "admin_removed",
            AdminAction::ModeratorAssigned { .. } => "moderator_assigned",
            AdminAction::ModeratorRemoved { .. } => "moderator_removed",
            AdminAction::PlayerBanned { .. } => "player_banned",
            AdminAction::PlayerSuspended { .. } => "player_suspended",
            AdminAction::PlayerUnbanned { .. } => "player_unbanned",
            AdminAction::GameApproved { .. } => "game_approved",
            AdminAction::GameRejected { .. } => "game_rejected",
            AdminAction::GameSuspended { .. } => "game_suspended",
            AdminAction::GameReactivated { .. } => "game_reactivated",
            AdminAction::GameDeprecated { .. } => "game_deprecated",
            AdminAction::ScoringConfigUpdated { .. } => "scoring_updated",
            AdminAction::PlayerProfileUpdated { .. } => "profile_updated",
            AdminAction::PlayerUnsuspended { .. } => "player_unsuspended",
            AdminAction::EventCreated { .. } => "event_created",
            AdminAction::EventUpdated { .. } => "event_updated",
            AdminAction::CsvDataImported { .. } => "csv_imported",
        };
        
        let log_id = format!("{}_{}", action_type, timestamp.micros());
        let audit_entry = AuditLogEntry {
            id: log_id.clone(),
            action,
            performed_by: performed_by.to_string(),
            target: target.map(|t| t.to_string()),
            timestamp,
            details: details.map(|d| d.to_string()),
        };
        
        self.audit_log.insert(&log_id, audit_entry).map_err(|_| GameHubError::DatabaseError)?;
        
        Ok(())
    }
}