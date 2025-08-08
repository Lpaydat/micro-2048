// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Configuration operation handlers

use crate::{
    core::{types::{EventType, ScoringConfig}},
    infrastructure::handlers::traits::{OperationHandler, HandlerUtils}
};

/// Configuration-specific operations
#[derive(Debug)]
pub enum ConfigOperation {
    UpdateScoringConfig {
        caller_discord_id: String,
        config: ScoringConfig,
    },
    ImportLeaderboardData {
        caller_discord_id: String,
        csv_data: String,
    },
}

/// Handler for configuration operations
pub struct ConfigOperationHandler;

impl OperationHandler for ConfigOperationHandler {
    type Operation = ConfigOperation;
    type Result = String;
    
    async fn handle<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        operation: Self::Operation
    ) -> Self::Result {
        match operation {
            ConfigOperation::UpdateScoringConfig { caller_discord_id, config } => {
                Self::update_scoring_config(contract, caller_discord_id, config).await
            }
            ConfigOperation::ImportLeaderboardData { caller_discord_id, csv_data } => {
                Self::import_leaderboard_data(contract, caller_discord_id, csv_data).await
            }
        }
    }
}

impl ConfigOperationHandler {
    /// Handle scoring configuration updates with permission validation and audit logging
    async fn update_scoring_config<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        config: ScoringConfig
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().update_scoring_config(&caller_discord_id, config, timestamp).await {
            Ok(()) => {
                HandlerUtils::success_response(
                    "updated", 
                    "scoring configuration", 
                    None
                )
            }
            Err(error) => {
                HandlerUtils::error_response("updating scoring configuration", &error.to_string())
            }
        }
    }
    
    /// Handle leaderboard data import with permission validation and comprehensive logging
    async fn import_leaderboard_data<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        csv_data: String
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().import_leaderboard_data(&caller_discord_id, &csv_data, timestamp).await {
            Ok(import_result) => {
                // Log the import completion
                HandlerUtils::log_event(
                    contract,
                    EventType::AdminAction,
                    format!("CSV import completed: {} players processed", import_result.total_processed),
                    Some(caller_discord_id),
                    None,
                );
                
                format!(
                    "CSV import completed successfully. Processed: {}, Success: {}, Failed: {}",
                    import_result.total_processed,
                    import_result.successful_imports,
                    import_result.failed_imports
                )
            }
            Err(error) => {
                HandlerUtils::error_response("importing CSV data", &error.to_string())
            }
        }
    }
}

