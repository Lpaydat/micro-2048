use crate::core::types::*;
use crate::infrastructure::{state::GameHubState, errors::GameHubError};
use linera_sdk::linera_base_types::Timestamp;


impl GameHubState {
    // ========== GAME RETRIEVAL METHODS ==========

    /// Get game by ID
    pub async fn get_game(&self, game_id: &str) -> Option<Game> {
        self.games.get(game_id).await.ok().flatten()
    }

    /// Get pending game by ID
    pub async fn get_pending_game(&self, game_id: &str) -> Option<PendingGame> {
        self.pending_games.get(game_id).await.ok().flatten()
    }

    /// Check if game exists (pending or approved)
    pub async fn game_exists(&self, game_id: &str) -> bool {
        self.get_game(game_id).await.is_some() || self.get_pending_game(game_id).await.is_some()
    }

    /// Check if game is approved
    pub async fn is_game_approved(&self, game_id: &str) -> bool {
        self.get_game(game_id).await.is_some()
    }

    // ========== PENDING GAME MANAGEMENT ==========

    /// Add pending game
    pub async fn add_pending_game(&mut self, pending_game: PendingGame) -> Result<(), GameHubError> {
        let game_id = pending_game.id.clone();
        self.pending_games.insert(&game_id, pending_game).map_err(|_| GameHubError::DatabaseError)?;
        Ok(())
    }

    /// Approve game (move from pending to active)
    pub async fn approve_game(&mut self, admin_discord_id: &str, game_id: &str, timestamp: Timestamp) -> Result<Game, GameHubError> {
        // Validate permission
        self.validate_admin_permission(admin_discord_id).await?;
        
        // Get pending game
        let pending_game = self.get_pending_game(game_id).await
            .ok_or_else(|| GameHubError::GameNotFound { game_id: game_id.to_string() })?;
        
        // Create approved game
        let approved_game = Game {
            id: pending_game.id.clone(),
            name: pending_game.name.clone(),
            description: pending_game.description.clone(),
            contract_address: pending_game.contract_address.clone(),
            developer_info: pending_game.developer_info.clone(),
            status: GameStatus::Active,
            approved_by: Some(admin_discord_id.to_string()),
            created_at: pending_game.created_at,
            approved_at: Some(timestamp),
        };
        
        // Add to approved games
        self.games.insert(&approved_game.id, approved_game.clone()).map_err(|_| GameHubError::DatabaseError)?;
        
        // Remove from pending
        self.pending_games.remove(game_id).map_err(|_| GameHubError::DatabaseError)?;
        
        // Log the action
        self.add_audit_log_entry(
            AdminAction::GameApproved { 
                game_id: game_id.to_string(), 
                game_name: approved_game.name.clone() 
            },
            admin_discord_id,
            Some(game_id),
            Some(&format!("Game '{}' approved", approved_game.name)),
            timestamp,
        ).await?;
        
        Ok(approved_game)
    }

    /// Reject game (remove from pending)
    pub async fn reject_game(&mut self, admin_discord_id: &str, game_id: &str, reason: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate permission
        self.validate_admin_permission(admin_discord_id).await?;
        
        // Verify pending game exists
        let pending_game = self.get_pending_game(game_id).await
            .ok_or_else(|| GameHubError::GameNotFound { game_id: game_id.to_string() })?;
        
        // Remove from pending
        self.pending_games.remove(game_id).map_err(|_| GameHubError::DatabaseError)?;
        
        // Log the action
        self.add_audit_log_entry(
            AdminAction::GameRejected { 
                game_id: game_id.to_string(), 
                reason: reason.to_string() 
            },
            admin_discord_id,
            Some(game_id),
            Some(&format!("Game '{}' rejected: {}", pending_game.name, reason)),
            timestamp,
        ).await?;
        
        Ok(())
    }

    // ========== GAME STATUS MANAGEMENT ==========

    /// Suspend game with reason
    pub async fn suspend_game(&mut self, admin_discord_id: &str, game_id: &str, reason: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate permission
        self.validate_moderator_permission(admin_discord_id).await?;
        
        // Get game
        let mut game = self.get_game(game_id).await
            .ok_or_else(|| GameHubError::GameNotFound { game_id: game_id.to_string() })?;
        
        // Update status
        game.status = GameStatus::Suspended { reason: reason.to_string() };
        
        // Save updated game
        self.games.insert(game_id, game).map_err(|_| GameHubError::DatabaseError)?;
        
        // Log the action
        self.add_audit_log_entry(
            AdminAction::GameSuspended { 
                game_id: game_id.to_string(), 
                reason: reason.to_string() 
            },
            admin_discord_id,
            Some(game_id),
            Some(&format!("Game suspended: {}", reason)),
            timestamp,
        ).await?;
        
        Ok(())
    }

    /// Reactivate suspended game
    pub async fn reactivate_game(&mut self, admin_discord_id: &str, game_id: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate permission
        self.validate_admin_permission(admin_discord_id).await?;
        
        // Get game
        let mut game = self.get_game(game_id).await
            .ok_or_else(|| GameHubError::GameNotFound { game_id: game_id.to_string() })?;
        
        // Update status to active
        game.status = GameStatus::Active;
        
        // Save updated game
        self.games.insert(game_id, game).map_err(|_| GameHubError::DatabaseError)?;
        
        // Log the action
        self.add_audit_log_entry(
            AdminAction::GameReactivated { 
                game_id: game_id.to_string() 
            },
            admin_discord_id,
            Some(game_id),
            Some("Game reactivated"),
            timestamp,
        ).await?;
        
        Ok(())
    }

    /// Deprecate game
    pub async fn deprecate_game(&mut self, admin_discord_id: &str, game_id: &str, timestamp: Timestamp) -> Result<(), GameHubError> {
        // Validate permission
        self.validate_admin_permission(admin_discord_id).await?;
        
        // Get game
        let mut game = self.get_game(game_id).await
            .ok_or_else(|| GameHubError::GameNotFound { game_id: game_id.to_string() })?;
        
        // Update status to deprecated
        game.status = GameStatus::Deprecated;
        
        // Save updated game
        self.games.insert(game_id, game).map_err(|_| GameHubError::DatabaseError)?;
        
        // Log the action
        self.add_audit_log_entry(
            AdminAction::GameDeprecated { 
                game_id: game_id.to_string() 
            },
            admin_discord_id,
            Some(game_id),
            Some("Game deprecated"),
            timestamp,
        ).await?;
        
        Ok(())
    }

    // ========== GAME LOOKUP UTILITIES ==========

    /// Get game by contract address using real MapView iteration search
    pub async fn get_game_by_contract_address(&self, contract_address: &str) -> Option<Game> {
        let game_ids = match self.games.indices().await {
            Ok(indices) => indices,
            Err(_) => return None,
        };
        
        for game_id in game_ids {
            if let Ok(Some(game)) = self.games.get(&game_id).await {
                if game.contract_address == contract_address {
                    return Some(game);
                }
            }
        }
        
        None
    }

    /// Get games by status using real MapView iteration and filtering
    pub async fn get_games_by_status(&self, status: GameStatus) -> Vec<Game> {
        let game_ids = match self.games.indices().await {
            Ok(indices) => indices,
            Err(_) => return Vec::new(),
        };
        
        let mut matching_games = Vec::new();
        for game_id in game_ids {
            if let Ok(Some(game)) = self.games.get(&game_id).await {
                if game.status == status {
                    matching_games.push(game);
                }
            }
        }
        
        // Sort by creation date (most recent first)
        matching_games.sort_by(|a, b| b.created_at.micros().cmp(&a.created_at.micros()));
        matching_games
    }

    /// Get all games using real MapView iteration
    pub async fn get_all_games(&self) -> Vec<Game> {
        let game_ids = match self.games.indices().await {
            Ok(indices) => indices,
            Err(_) => return Vec::new(),
        };
        
        let mut games = Vec::new();
        for game_id in game_ids {
            if let Ok(Some(game)) = self.games.get(&game_id).await {
                games.push(game);
            }
        }
        
        // Sort games by creation date (most recent first)
        games.sort_by(|a, b| b.created_at.micros().cmp(&a.created_at.micros()));
        games
    }

    /// Get all pending games using real MapView iteration
    pub async fn get_all_pending_games(&self) -> Vec<PendingGame> {
        let game_ids = match self.pending_games.indices().await {
            Ok(indices) => indices,
            Err(_) => return Vec::new(),
        };
        
        let mut pending_games = Vec::new();
        for game_id in game_ids {
            if let Ok(Some(pending_game)) = self.pending_games.get(&game_id).await {
                pending_games.push(pending_game);
            }
        }
        
        // Sort pending games by creation date (oldest first for review queue)
        pending_games.sort_by(|a, b| a.created_at.micros().cmp(&b.created_at.micros()));
        pending_games
    }
}