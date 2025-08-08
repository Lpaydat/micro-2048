// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Admin state management operations
//!
//! This module handles admin operations, permission validation, contract initialization,
//! and audit logging for administrative actions.

use linera_sdk::linera_base_types::Timestamp;
use crate::core::types::*;
use crate::core::validation::player_validation::PlayerValidator;
use crate::infrastructure::errors::GameHubError;
use super::GameHubState;

/// Initialize contract with default configuration and admin setup
pub async fn initialize_contract(
    state: &mut GameHubState,
    admin_discord_id: &str,
    timestamp: Timestamp,
) -> Result<(), GameHubError> {
    // Validate admin Discord ID format
    PlayerValidator::validate_discord_id(admin_discord_id)?;
    
    // Set up default scoring configuration
    state.scoring_config.set(ScoringConfig::default());
    
    // Add the initial admin
    state.admins.insert(admin_discord_id)?;
    
    // Create audit log entry for initialization
    let audit_entry_id = format!("init_{}", timestamp.micros());
    let audit_entry = AuditLogEntry {
        id: audit_entry_id.clone(),
        action: AdminAction::AdminAdded { 
            admin_id: admin_discord_id.to_string() 
        },
        performed_by: "system".to_string(),
        target: Some(admin_discord_id.to_string()),
        timestamp,
        details: Some("Contract initialization".to_string()),
    };
    
    state.audit_log.insert(&audit_entry_id, audit_entry)?;
    
    Ok(())
}

/// Validate admin permission for a given Discord ID
pub async fn validate_admin_permission(
    state: &GameHubState,
    discord_id: &str,
) -> Result<(), GameHubError> {
    if state.admins.contains(discord_id).await.unwrap_or(false) {
        Ok(())
    } else {
        Err(GameHubError::UnauthorizedAction {
            action: "admin_operation".to_string(),
            discord_id: discord_id.to_string(),
        })
    }
}

/// Validate moderator permission for a given Discord ID
pub async fn validate_moderator_permission(
    state: &GameHubState,
    discord_id: &str,
) -> Result<(), GameHubError> {
    if state.moderators.contains(discord_id).await.unwrap_or(false) ||
       state.admins.contains(discord_id).await.unwrap_or(false) {
        Ok(())
    } else {
        Err(GameHubError::UnauthorizedAction {
            action: "moderator_operation".to_string(),
            discord_id: discord_id.to_string(),
        })
    }
}

/// Add audit log entry for administrative actions
pub async fn add_audit_log_entry(
    state: &mut GameHubState,
    action: AdminAction,
    performed_by: &str,
    target: Option<&str>,
    details: Option<&str>,
    timestamp: Timestamp,
) -> Result<(), GameHubError> {
    let audit_entry_id = format!("audit_{}_{}", timestamp.micros(), performed_by);
    let audit_entry = AuditLogEntry {
        id: audit_entry_id.clone(),
        action,
        performed_by: performed_by.to_string(),
        target: target.map(|t| t.to_string()),
        timestamp,
        details: details.map(|d| d.to_string()),
    };
    
    state.audit_log.insert(&audit_entry_id, audit_entry)
        .map_err(|_| GameHubError::DatabaseError)?;
    
    Ok(())
}

/// Get all audit log entries with chronological sorting
pub async fn get_all_audit_log_entries(
    state: &GameHubState,
    limit: Option<u32>,
) -> Vec<AuditLogEntry> {
    let log_ids = match state.audit_log.indices().await {
        Ok(indices) => indices,
        Err(_) => return Vec::new(),
    };
    
    let mut audit_entries = Vec::new();
    for log_id in log_ids {
        if let Ok(Some(entry)) = state.audit_log.get(&log_id).await {
            audit_entries.push(entry);
        }
    }
    
    // Sort by timestamp (most recent first)
    audit_entries.sort_by(|a, b| b.timestamp.micros().cmp(&a.timestamp.micros()));
    
    // Apply limit if specified
    if let Some(limit) = limit {
        audit_entries.truncate(limit as usize);
    }
    
    audit_entries
}

/// Get all admin Discord IDs
pub async fn get_all_admins(state: &GameHubState) -> Vec<String> {
    match state.admins.indices().await {
        Ok(admin_ids) => admin_ids,
        Err(_) => Vec::new(),
    }
}

/// Get all moderator Discord IDs
pub async fn get_all_moderators(state: &GameHubState) -> Vec<String> {
    match state.moderators.indices().await {
        Ok(moderator_ids) => moderator_ids,
        Err(_) => Vec::new(),
    }
}

/// Check if a Discord ID has admin privileges
pub async fn is_admin(state: &GameHubState, discord_id: &str) -> bool {
    state.admins.contains(discord_id).await.unwrap_or(false)
}

/// Check if a Discord ID has moderator privileges (or admin)
pub async fn is_moderator_or_admin(state: &GameHubState, discord_id: &str) -> bool {
    state.moderators.contains(discord_id).await.unwrap_or(false) ||
    state.admins.contains(discord_id).await.unwrap_or(false)
}

/// Add a new admin (admin-only operation)
pub async fn add_admin(
    state: &mut GameHubState,
    caller_discord_id: &str,
    new_admin_discord_id: &str,
    timestamp: Timestamp,
) -> Result<(), GameHubError> {
    // Validate caller has admin privileges
    validate_admin_permission(state, caller_discord_id).await?;
    
    // Validate new admin Discord ID format
    PlayerValidator::validate_discord_id(new_admin_discord_id)?;
    
    // Check if already an admin
    if is_admin(state, new_admin_discord_id).await {
        return Err(GameHubError::UserAlreadyHasRole {
            discord_id: new_admin_discord_id.to_string(),
            role: "admin".to_string(),
        });
    }
    
    // Add to admin set
    state.admins.insert(new_admin_discord_id)
        .map_err(|_| GameHubError::DatabaseError)?;
    
    // Add audit log entry
    add_audit_log_entry(
        state,
        AdminAction::AdminAdded { 
            admin_id: new_admin_discord_id.to_string() 
        },
        caller_discord_id,
        Some(new_admin_discord_id),
        Some(&format!("Admin {} added by {}", new_admin_discord_id, caller_discord_id)),
        timestamp,
    ).await?;
    
    Ok(())
}

/// Remove an admin (admin-only operation)
pub async fn remove_admin(
    state: &mut GameHubState,
    caller_discord_id: &str,
    admin_discord_id: &str,
    timestamp: Timestamp,
) -> Result<(), GameHubError> {
    // Validate caller has admin privileges
    validate_admin_permission(state, caller_discord_id).await?;
    
    // Cannot remove self if only admin
    let all_admins = get_all_admins(state).await;
    if all_admins.len() <= 1 && caller_discord_id == admin_discord_id {
        return Err(GameHubError::CannotRemoveOnlyAdmin);
    }
    
    // Check if target is actually an admin
    if !is_admin(state, admin_discord_id).await {
        return Err(GameHubError::UserNotFound {
            role: "admin".to_string(),
            discord_id: admin_discord_id.to_string(),
        });
    }
    
    // Remove from admin set
    state.admins.remove(admin_discord_id)
        .map_err(|_| GameHubError::DatabaseError)?;
    
    // Add audit log entry
    add_audit_log_entry(
        state,
        AdminAction::AdminRemoved { 
            admin_id: admin_discord_id.to_string() 
        },
        caller_discord_id,
        Some(admin_discord_id),
        Some(&format!("Admin {} removed by {}", admin_discord_id, caller_discord_id)),
        timestamp,
    ).await?;
    
    Ok(())
}

/// Assign moderator role (admin-only operation)
pub async fn assign_moderator(
    state: &mut GameHubState,
    caller_discord_id: &str,
    moderator_discord_id: &str,
    timestamp: Timestamp,
) -> Result<(), GameHubError> {
    // Validate caller has admin privileges
    validate_admin_permission(state, caller_discord_id).await?;
    
    // Validate moderator Discord ID format
    PlayerValidator::validate_discord_id(moderator_discord_id)?;
    
    // Check if already a moderator or admin
    if is_moderator_or_admin(state, moderator_discord_id).await {
        return Err(GameHubError::UserAlreadyHasRole {
            discord_id: moderator_discord_id.to_string(),
            role: "moderator".to_string(),
        });
    }
    
    // Add to moderator set
    state.moderators.insert(moderator_discord_id)
        .map_err(|_| GameHubError::DatabaseError)?;
    
    // Add audit log entry
    add_audit_log_entry(
        state,
        AdminAction::ModeratorAssigned { 
            moderator_id: moderator_discord_id.to_string() 
        },
        caller_discord_id,
        Some(moderator_discord_id),
        Some(&format!("Moderator {} assigned by {}", moderator_discord_id, caller_discord_id)),
        timestamp,
    ).await?;
    
    Ok(())
}

/// Remove moderator role (admin-only operation)
pub async fn remove_moderator(
    state: &mut GameHubState,
    caller_discord_id: &str,
    moderator_discord_id: &str,
    timestamp: Timestamp,
) -> Result<(), GameHubError> {
    // Validate caller has admin privileges
    validate_admin_permission(state, caller_discord_id).await?;
    
    // Check if target is actually a moderator
    if !state.moderators.contains(moderator_discord_id).await.unwrap_or(false) {
        return Err(GameHubError::UserNotFound {
            role: "moderator".to_string(),
            discord_id: moderator_discord_id.to_string(),
        });
    }
    
    // Remove from moderator set
    state.moderators.remove(moderator_discord_id)
        .map_err(|_| GameHubError::DatabaseError)?;
    
    // Add audit log entry
    add_audit_log_entry(
        state,
        AdminAction::ModeratorRemoved { 
            moderator_id: moderator_discord_id.to_string() 
        },
        caller_discord_id,
        Some(moderator_discord_id),
        Some(&format!("Moderator {} removed by {}", moderator_discord_id, caller_discord_id)),
        timestamp,
    ).await?;
    
    Ok(())
}