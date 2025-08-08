// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Import/Export state operations
//!
//! This module handles CSV data import/export operations, bulk player data management,
//! and administrative data migration tasks.

use linera_sdk::linera_base_types::Timestamp;
use crate::core::types::*;
use crate::core::validation::player_validation::PlayerValidator;
use crate::infrastructure::errors::GameHubError;
use super::{GameHubState, admin_state, player_state};

/// Import leaderboard data from CSV format
pub async fn import_leaderboard_data(
    state: &mut GameHubState,
    caller_discord_id: &str,
    csv_data: &str,
    timestamp: Timestamp,
) -> Result<ImportResult, GameHubError> {
    // Validate admin permissions
    admin_state::validate_admin_permission(state, caller_discord_id).await?;

    let mut import_result = ImportResult {
        total_processed: 0,
        successful_imports: 0,
        failed_imports: 0,
        errors: Vec::new(),
    };

    // Parse CSV data (expecting format: discord_id,username,total_points,participation_streak)
    let lines: Vec<&str> = csv_data.lines().collect();
    
    // Skip header if present
    let data_lines = if !lines.is_empty() && lines[0].contains("discord_id") {
        &lines[1..]
    } else {
        &lines[..]
    };

    for (line_number, line) in data_lines.iter().enumerate() {
        import_result.total_processed += 1;
        
        let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if fields.len() < 4 {
            import_result.failed_imports += 1;
            import_result.errors.push(format!("Line {}: Invalid CSV format - expected 4 fields", line_number + 2));
            continue;
        }

        let discord_id = fields[0];
        let username = fields[1];
        let total_points: u64 = match fields[2].parse() {
            Ok(points) => points,
            Err(_) => {
                import_result.failed_imports += 1;
                import_result.errors.push(format!("Line {}: Invalid total_points value", line_number + 2));
                continue;
            }
        };
        let participation_streak: u32 = match fields[3].parse() {
            Ok(streak) => streak,
            Err(_) => {
                import_result.failed_imports += 1;
                import_result.errors.push(format!("Line {}: Invalid participation_streak value", line_number + 2));
                continue;
            }
        };

        // Validate Discord ID format
        if let Err(_) = PlayerValidator::validate_discord_id(discord_id) {
            import_result.failed_imports += 1;
            import_result.errors.push(format!("Line {}: Invalid Discord ID format", line_number + 2));
            continue;
        }

        // Validate username
        if let Err(_) = PlayerValidator::validate_username(username) {
            import_result.failed_imports += 1;
            import_result.errors.push(format!("Line {}: Invalid username", line_number + 2));
            continue;
        }

        // Create or update player
        match import_player_data(state, discord_id, username, total_points, participation_streak, timestamp).await {
            Ok(_) => {
                import_result.successful_imports += 1;
            },
            Err(error) => {
                import_result.failed_imports += 1;
                import_result.errors.push(format!("Line {}: {}", line_number + 2, error));
            }
        }
    }

    // Add audit log entry
    admin_state::add_audit_log_entry(
        state,
        AdminAction::CsvDataImported { 
            records_processed: import_result.total_processed,
            successful: import_result.successful_imports,
            failed: import_result.failed_imports 
        },
        caller_discord_id,
        None,
        Some(&format!("CSV import completed: {} processed, {} successful, {} failed", 
            import_result.total_processed, 
            import_result.successful_imports, 
            import_result.failed_imports
        )),
        timestamp,
    ).await?;

    Ok(import_result)
}

/// Helper method to import individual player data
pub async fn import_player_data(
    state: &mut GameHubState,
    discord_id: &str,
    username: &str,
    total_points: u64,
    participation_streak: u32,
    timestamp: Timestamp,
) -> Result<(), GameHubError> {
    // Check if player exists
    if let Some(mut player) = player_state::get_player(state, discord_id).await {
        // Update existing player
        player.username = username.to_string();
        player.total_points = total_points;
        player.participation_streak = participation_streak;
        // Update best_streak if imported streak is higher
        if participation_streak > player.best_streak {
            player.best_streak = participation_streak;
        }
        player.last_active = timestamp;
        
        state.players.insert(discord_id, player).map_err(|_| GameHubError::StorageError)?;
    } else {
        // Create new player
        let new_player = Player {
            discord_id: discord_id.to_string(),
            username: username.to_string(),
            avatar_url: None,
            total_points,
            participation_streak,
            best_streak: participation_streak, // Initialize best_streak with current streak
            current_rank: None,
            status: PlayerStatus::Active,
            created_at: timestamp,
            last_active: timestamp,
        };
        
        state.players.insert(discord_id, new_player).map_err(|_| GameHubError::StorageError)?;
    }

    Ok(())
}

/// Export leaderboard data to CSV format
pub async fn export_leaderboard_data(
    state: &GameHubState,
    caller_discord_id: &str,
    limit: Option<u32>,
) -> Result<String, GameHubError> {
    // Validate admin permissions
    admin_state::validate_admin_permission(state, caller_discord_id).await?;
    
    // Get all players sorted by points
    let mut players = player_state::get_all_players(state).await;
    
    // Apply limit if specified
    if let Some(limit) = limit {
        players.truncate(limit as usize);
    }
    
    // Build CSV content
    let mut csv_content = String::new();
    csv_content.push_str("discord_id,username,total_points,participation_streak,best_streak,status,created_at,last_active\n");
    
    for player in players {
        csv_content.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            player.discord_id,
            player.username,
            player.total_points,
            player.participation_streak,
            player.best_streak,
            match player.status {
                PlayerStatus::Active => "active",
                PlayerStatus::Banned { .. } => "banned",
                PlayerStatus::Suspended { .. } => "suspended",
            },
            player.created_at.micros(),
            player.last_active.micros(),
        ));
    }
    
    Ok(csv_content)
}

/// Export audit log data to CSV format
pub async fn export_audit_log_data(
    state: &GameHubState,
    caller_discord_id: &str,
    limit: Option<u32>,
) -> Result<String, GameHubError> {
    // Validate admin permissions
    admin_state::validate_admin_permission(state, caller_discord_id).await?;
    
    // Get all audit log entries
    let audit_entries = admin_state::get_all_audit_log_entries(state, limit).await;
    
    // Build CSV content
    let mut csv_content = String::new();
    csv_content.push_str("timestamp,action,performed_by,target,details\n");
    
    for entry in audit_entries {
        csv_content.push_str(&format!(
            "{},{},{},{},{}\n",
            entry.timestamp.micros(),
            format!("{:?}", entry.action), // Debug format for action enum
            entry.performed_by,
            entry.target.unwrap_or_else(|| "".to_string()),
            entry.details.unwrap_or_else(|| "".to_string()),
        ));
    }
    
    Ok(csv_content)
}

/// Export event participation data to CSV format
pub async fn export_event_participation_data(
    state: &GameHubState,
    caller_discord_id: &str,
    event_id: Option<String>,
) -> Result<String, GameHubError> {
    // Validate admin permissions
    admin_state::validate_admin_permission(state, caller_discord_id).await?;
    
    // Get pending player data (which contains event participation)
    let pending_data = player_state::get_all_pending_player_data(state).await;
    
    // Build CSV content
    let mut csv_content = String::new();
    csv_content.push_str("discord_id,event_id,game_id,score,participation_timestamp,streak_eligible\n");
    
    for player_data in pending_data {
        for event_score in player_data.event_scores {
            // Filter by event_id if specified
            if let Some(ref filter_event_id) = event_id {
                if &event_score.event_id != filter_event_id {
                    continue;
                }
            }
            
            csv_content.push_str(&format!(
                "{},{},{},{},{},{}\n",
                player_data.discord_id,
                event_score.event_id,
                event_score.game_id,
                event_score.score,
                event_score.participation_timestamp.micros(),
                event_score.streak_eligible,
            ));
        }
    }
    
    Ok(csv_content)
}