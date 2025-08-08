// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Event operation handlers

use crate::{
    core::types::EventType,
    infrastructure::handlers::traits::{OperationHandler, HandlerUtils}
};

/// Event-specific operations
#[derive(Debug)]
pub enum EventOperation {
    CreateEvent {
        caller_discord_id: String,
        game_id: String,
        name: String,
        description: String,
        start_time: u64,
        end_time: u64,
        is_mandatory: bool,
        max_participants: Option<u32>,
        prize_pool: Option<u64>,
    },
    UpdateEvent {
        caller_discord_id: String,
        event_id: String,
        name: Option<String>,
        description: Option<String>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        is_mandatory: Option<bool>,
        max_participants: Option<u32>,
        prize_pool: Option<u64>,
    },
    SetEventMandatory {
        caller_discord_id: String,
        event_id: String,
        is_mandatory: bool,
    },
}

/// Handler for event operations
pub struct EventOperationHandler;

impl OperationHandler for EventOperationHandler {
    type Operation = EventOperation;
    type Result = String;
    
    async fn handle<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        operation: Self::Operation
    ) -> Self::Result {
        match operation {
            EventOperation::CreateEvent {
                caller_discord_id,
                game_id,
                name,
                description,
                start_time,
                end_time,
                is_mandatory,
                max_participants,
                prize_pool,
            } => {
                Self::create_event(
                    contract,
                    caller_discord_id,
                    game_id,
                    name,
                    description,
                    start_time,
                    end_time,
                    is_mandatory,
                    max_participants,
                    prize_pool,
                ).await
            }
            EventOperation::UpdateEvent {
                caller_discord_id,
                event_id,
                name,
                description,
                start_time,
                end_time,
                is_mandatory,
                max_participants,
                prize_pool,
            } => {
                Self::update_event(
                    contract,
                    caller_discord_id,
                    event_id,
                    name,
                    description,
                    start_time,
                    end_time,
                    is_mandatory,
                    max_participants,
                    prize_pool,
                ).await
            }
            EventOperation::SetEventMandatory {
                caller_discord_id,
                event_id,
                is_mandatory,
            } => {
                Self::set_event_mandatory(contract, caller_discord_id, event_id, is_mandatory).await
            }
        }
    }
}

impl EventOperationHandler {
    /// Handle event creation with validation and logging
    async fn create_event<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        game_id: String,
        name: String,
        description: String,
        start_time: u64,
        end_time: u64,
        is_mandatory: bool,
        max_participants: Option<u32>,
        prize_pool: Option<u64>,
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        let start_timestamp = linera_sdk::linera_base_types::Timestamp::from(start_time);
        let end_timestamp = linera_sdk::linera_base_types::Timestamp::from(end_time);
        
        match contract.get_state().create_event(
            &caller_discord_id,
            &game_id,
            &name,
            &description,
            start_timestamp,
            end_timestamp,
            is_mandatory,
            max_participants,
            prize_pool,
            timestamp
        ).await {
            Ok(event) => {
                // Log the event creation
                HandlerUtils::log_event(
                    contract,
                    EventType::EventCreated,
                    format!("Event '{}' created for game {}", name, game_id),
                    Some(caller_discord_id),
                    Some(event.id.clone()),
                );
                
                format!("Event '{}' created successfully with ID: {}", name, event.id)
            }
            Err(error) => {
                HandlerUtils::error_response("creating event", &error.to_string())
            }
        }
    }
    
    /// Handle event updates with validation and logging
    async fn update_event<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        event_id: String,
        name: Option<String>,
        description: Option<String>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        is_mandatory: Option<bool>,
        max_participants: Option<u32>,
        prize_pool: Option<u64>,
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        let start_timestamp = start_time.map(|t| linera_sdk::linera_base_types::Timestamp::from(t));
        let end_timestamp = end_time.map(|t| linera_sdk::linera_base_types::Timestamp::from(t));
        
        match contract.get_state().update_event(
            &caller_discord_id,
            &event_id,
            name.as_deref(),
            description.as_deref(),
            start_timestamp,
            end_timestamp,
            is_mandatory,
            max_participants,
            prize_pool,
            timestamp
        ).await {
            Ok(()) => {
                // Log the event update
                HandlerUtils::log_event(
                    contract,
                    EventType::EventUpdated,
                    format!("Event {} updated", event_id),
                    Some(caller_discord_id),
                    Some(event_id.clone()),
                );
                
                format!("Event {} updated successfully", event_id)
            }
            Err(error) => {
                HandlerUtils::error_response("updating event", &error.to_string())
            }
        }
    }
    
    /// Handle setting event mandatory status with validation and logging
    async fn set_event_mandatory<T: crate::infrastructure::handlers::traits::ContractInterface>(
        contract: &mut T,
        caller_discord_id: String,
        event_id: String,
        is_mandatory: bool,
    ) -> String {
        let timestamp = HandlerUtils::get_timestamp(contract);
        
        match contract.get_state().set_event_mandatory(&caller_discord_id, &event_id, is_mandatory, timestamp).await {
            Ok(()) => {
                // Log the mandatory status change
                HandlerUtils::log_event(
                    contract,
                    EventType::EventUpdated,
                    format!("Event {} mandatory status set to {}", event_id, is_mandatory),
                    Some(caller_discord_id),
                    Some(event_id.clone()),
                );
                
                format!("Event {} mandatory status updated to {}", event_id, is_mandatory)
            }
            Err(error) => {
                HandlerUtils::error_response("setting event mandatory status", &error.to_string())
            }
        }
    }
}

