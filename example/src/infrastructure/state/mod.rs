// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Modular GameHub state management
//! 
//! This module provides a clean, modular architecture for managing GameHub blockchain state.
//! The state is organized by domain responsibilities for better maintainability.

use linera_sdk::{
    linera_base_types::Timestamp,
    views::{linera_views, MapView, RegisterView, RootView, SetView, ViewStorageContext},
};

use crate::core::types::*;
use crate::infrastructure::errors::GameHubError;

// Module declarations
pub mod player_state;
pub mod event_state;
pub mod batch_state;
pub mod admin_state;
pub mod import_export_state;
pub mod utility_state;
pub mod messaging_state;

// Re-export state implementations for clean API
pub use player_state::*;
pub use event_state::*;
pub use batch_state::*;
pub use admin_state::*;
pub use import_export_state::*;
pub use utility_state::*;
pub use messaging_state::*;

/// The main application state struct
/// 
/// This struct maintains all blockchain state collections and delegates operations
/// to domain-specific modules for better organization and maintainability.
#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct GameHubState {
    /// Player management (Discord ID -> Player)
    pub players: MapView<String, Player>,

    /// Unregistered player scores (for players who haven't registered yet)
    pub pending_player_data: MapView<String, PendingPlayerData>,

    /// Game management (Game ID -> Game)
    pub games: MapView<String, Game>,

    /// Pending games awaiting approval
    pub pending_games: MapView<String, PendingGame>,

    /// Event tracking (Event ID -> Event)
    pub events: MapView<String, Event>,

    /// GameHub event logging for activity tracking (Event ID -> GameHubEvent)
    pub gamehub_events: MapView<String, GameHubEvent>,

    /// Leaderboards (Event ID -> Rankings)
    pub leaderboards: MapView<String, Vec<LeaderboardEntry>>,

    /// Admin permissions (Discord IDs with admin permissions)
    pub admins: SetView<String>,

    /// Moderator permissions (Discord IDs with moderator permissions)
    pub moderators: SetView<String>,

    /// Scoring configuration
    pub scoring_config: RegisterView<ScoringConfig>,

    /// Audit log for administrative actions (Log ID -> AuditLogEntry)
    pub audit_log: MapView<String, AuditLogEntry>,
}

// Implement delegation to domain-specific modules
impl GameHubState {
    // ========== PLAYER MANAGEMENT DELEGATION ==========
    
    /// Initialize contract with default configuration and admin setup
    pub async fn initialize_contract(&mut self, admin_discord_id: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        admin_state::initialize_contract(self, admin_discord_id, timestamp).await
    }

    /// Get player by Discord ID
    pub async fn get_player(&self, discord_id: &str) -> Option<Player> {
        player_state::get_player(self, discord_id).await
    }

    /// Check if player exists
    pub async fn player_exists(&self, discord_id: &str) -> bool {
        player_state::player_exists(self, discord_id).await
    }

    /// Check if player is active (not banned or suspended)
    pub async fn is_player_active(&self, discord_id: &str) -> bool {
        player_state::is_player_active(self, discord_id).await
    }

    /// Update player's last activity timestamp
    pub async fn update_player_activity(&mut self, discord_id: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        player_state::update_player_activity(self, discord_id, timestamp).await
    }

    /// Register or update player with pending data merging
    pub async fn register_or_update_player(&mut self, discord_id: &str, username: &str, avatar_url: Option<String>, timestamp: Timestamp) -> Result<Player, GameHubError> {
        player_state::register_or_update_player(self, discord_id, username, avatar_url, timestamp).await
    }

    /// Get player statistics
    pub async fn get_player_stats(&self, discord_id: &str) -> Option<PlayerStats> {
        player_state::get_player_stats(self, discord_id).await
    }

    /// Update player profile (username/avatar)
    pub async fn update_player_profile(&mut self, discord_id: &str, username: Option<String>, avatar_url: Option<String>, timestamp: Timestamp) -> Result<(), GameHubError> {
        player_state::update_player_profile(self, discord_id, username, avatar_url, timestamp).await
    }

    /// Check if player has pending data
    pub async fn has_pending_data(&self, discord_id: &str) -> bool {
        player_state::has_pending_data(self, discord_id).await
    }

    /// Get pending player data
    pub async fn get_pending_data(&self, discord_id: &str) -> Option<PendingPlayerData> {
        player_state::get_pending_data(self, discord_id).await
    }

    /// Get all pending player data using MapView iteration
    pub async fn get_all_pending_player_data(&self) -> Vec<PendingPlayerData> {
        player_state::get_all_pending_player_data(self).await
    }

    /// Calculate event-based points earned for a player
    pub async fn get_player_event_points(&self, discord_id: &str) -> u64 {
        player_state::get_player_event_points(self, discord_id).await
    }

    /// Get all players using MapView iteration
    pub async fn get_all_players(&self) -> Vec<Player> {
        player_state::get_all_players(self).await
    }

    // ========== EVENT MANAGEMENT DELEGATION ==========

    /// Get event by ID
    pub async fn get_event(&self, event_id: &str) -> Option<Event> {
        event_state::get_event(self, event_id).await
    }

    /// Check if event exists
    pub async fn event_exists(&self, event_id: &str) -> bool {
        event_state::event_exists(self, event_id).await
    }

    /// Get game ID for event
    pub async fn get_game_id_for_event(&self, event_id: &str) -> Option<String> {
        event_state::get_game_id_for_event(self, event_id).await
    }

    /// Get all events with sorting (most recent first)
    pub async fn get_all_events(&self) -> Vec<Event> {
        event_state::get_all_events(self).await
    }

    /// Get all events for a specific game
    pub async fn get_events_by_game(&self, game_id: &str) -> Vec<Event> {
        event_state::get_events_by_game(self, game_id).await
    }

    /// Get events by status
    pub async fn get_events_by_status(&self, status: EventStatus) -> Vec<Event> {
        event_state::get_events_by_status(self, status).await
    }

    /// Create a new event with admin permission validation
    pub async fn create_event(&mut self, caller_discord_id: &str, game_id: &str, name: &str, description: &str, start_time: Timestamp, end_time: Timestamp, is_mandatory: bool, max_participants: Option<u32>, prize_pool: Option<u64>, timestamp: Timestamp) -> Result<Event, GameHubError> {
        event_state::create_event(self, caller_discord_id, game_id, name, description, start_time, end_time, is_mandatory, max_participants, prize_pool, timestamp).await
    }

    /// Update an existing event with admin permission validation
    pub async fn update_event(&mut self, caller_discord_id: &str, event_id: &str, name: Option<&str>, description: Option<&str>, start_time: Option<Timestamp>, end_time: Option<Timestamp>, is_mandatory: Option<bool>, max_participants: Option<u32>, prize_pool: Option<u64>, timestamp: Timestamp) -> Result<(), GameHubError> {
        event_state::update_event(self, caller_discord_id, event_id, name, description, start_time, end_time, is_mandatory, max_participants, prize_pool, timestamp).await
    }

    /// Set the mandatory status of an event for streak control
    pub async fn set_event_mandatory(&mut self, caller_discord_id: &str, event_id: &str, is_mandatory: bool, timestamp: Timestamp) -> Result<(), GameHubError> {
        event_state::set_event_mandatory(self, caller_discord_id, event_id, is_mandatory, timestamp).await
    }

    // ========== BATCH PROCESSING DELEGATION ==========

    /// Process single player update
    pub async fn process_player_update(&mut self, update: &PlayerEventUpdate, event_id: &str) -> Result<(), GameHubError> {
        batch_state::process_player_update(self, update, event_id).await
    }

    /// Process registered player update
    pub async fn process_registered_player_update(&mut self, update: &PlayerEventUpdate, event_id: &str) -> Result<(), GameHubError> {
        batch_state::process_registered_player_update(self, update, event_id).await
    }

    /// Process batch player updates
    pub async fn process_batch_player_updates(&mut self, updates: Vec<PlayerEventUpdate>, event_id: &str) -> BatchUpdateResult {
        batch_state::process_batch_player_updates(self, updates, event_id).await
    }

    /// Add pending player data
    pub async fn add_pending_player_data(&mut self, update: &PlayerEventUpdate, event_id: &str) -> Result<(), GameHubError> {
        batch_state::add_pending_player_data(self, update, event_id).await
    }

    // ========== IMPORT/EXPORT DELEGATION ==========

    /// Import leaderboard data from CSV format
    pub async fn import_leaderboard_data(&mut self, caller_discord_id: &str, csv_data: &str, timestamp: Timestamp) -> Result<ImportResult, GameHubError> {
        import_export_state::import_leaderboard_data(self, caller_discord_id, csv_data, timestamp).await
    }

    // ========== UTILITY DELEGATION ==========

    /// Validate contract address format
    pub async fn validate_contract_address(&self, contract_address: &str) -> bool {
        utility_state::validate_contract_address(self, contract_address).await
    }

    /// Calculate streak from pending event scores
    pub async fn calculate_streak_from_pending(&self, pending_data: &PendingPlayerData) -> u32 {
        utility_state::calculate_streak_from_pending(self, pending_data).await
    }
}