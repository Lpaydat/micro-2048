// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Event-related GraphQL queries

use std::sync::Arc;
use async_graphql::{Object, Result};
use crate::{
    api::graphql_types::{EventObject, EventStatusType},
    infrastructure::state::GameHubState,
};

/// Event query resolvers
#[derive(Clone)]
pub struct EventQueries {
    pub state: Arc<GameHubState>,
}

#[Object]
impl EventQueries {
    /// Get events with optional status filter and limit
    async fn events(&self, status: Option<EventStatusType>, limit: Option<i32>) -> Result<Vec<EventObject>> {
        let limit = limit.unwrap_or(50).max(1).min(100) as usize; // Reasonable limits
        
        let events = match status {
            Some(status_filter) => {
                // Convert GraphQL EventStatusType to domain EventStatus
                let domain_status = match status_filter {
                    EventStatusType::Upcoming => crate::core::types::event::EventStatus::Upcoming,
                    EventStatusType::Active => crate::core::types::event::EventStatus::Active,
                    EventStatusType::Ended => crate::core::types::event::EventStatus::Ended,
                    EventStatusType::Cancelled => crate::core::types::event::EventStatus::Cancelled,
                };
                self.state.get_events_by_status(domain_status).await
            },
            None => self.state.get_all_events().await,
        };
        
        let mut event_objects = Vec::new();
        for event in events.into_iter().take(limit) {
            event_objects.push(EventObject {
                id: event.id.clone(),
                game_id: event.game_id.clone(),
                name: event.name.clone(),
                description: event.description.clone(),
                start_time: event.start_time.micros().to_string(),
                end_time: Some(event.end_time.micros().to_string()),
                status: EventStatusType::from(&event.status),
                max_participants: event.max_participants,
                prize_pool: event.prize_pool,
                is_mandatory_for_streak: event.is_mandatory_for_streak,
            });
        }
        Ok(event_objects)
    }

    /// Get a single event by ID
    async fn event(&self, id: String) -> Result<Option<EventObject>> {
        match self.state.get_event(&id).await {
            Some(event) => {
                let event_obj = EventObject {
                    id: event.id.clone(),
                    game_id: event.game_id.clone(),
                    name: event.name.clone(),
                    description: event.description.clone(),
                    start_time: event.start_time.micros().to_string(),
                    end_time: Some(event.end_time.micros().to_string()),
                    status: EventStatusType::from(&event.status),
                    max_participants: event.max_participants,
                    prize_pool: event.prize_pool,
                    is_mandatory_for_streak: event.is_mandatory_for_streak,
                };
                Ok(Some(event_obj))
            },
            None => Ok(None),
        }
    }

    /// Get upcoming events with limit
    async fn upcoming_events(&self, limit: Option<i32>) -> Result<Vec<EventObject>> {
        let limit = limit.unwrap_or(20).max(1).min(50) as usize; // Smaller default for upcoming
        
        let events = self.state.get_events_by_status(
            crate::core::types::event::EventStatus::Upcoming
        ).await;
        
        let mut event_objects = Vec::new();
        for event in events.into_iter().take(limit) {
            event_objects.push(EventObject {
                id: event.id.clone(),
                game_id: event.game_id.clone(),
                name: event.name.clone(),
                description: event.description.clone(),
                start_time: event.start_time.micros().to_string(),
                end_time: Some(event.end_time.micros().to_string()),
                status: EventStatusType::from(&event.status),
                max_participants: event.max_participants,
                prize_pool: event.prize_pool,
                is_mandatory_for_streak: event.is_mandatory_for_streak,
            });
        }
        Ok(event_objects)
    }

    /// Get events for a specific game
    async fn events_by_game(&self, game_id: String) -> Result<Vec<EventObject>> {
        let events = self.state.get_events_by_game(&game_id).await;
        
        let mut event_objects = Vec::new();
        for event in events {
            event_objects.push(EventObject {
                id: event.id.clone(),
                game_id: event.game_id.clone(),
                name: event.name.clone(),
                description: event.description.clone(),
                start_time: event.start_time.micros().to_string(),
                end_time: Some(event.end_time.micros().to_string()),
                status: EventStatusType::from(&event.status),
                max_participants: event.max_participants,
                prize_pool: event.prize_pool,
                is_mandatory_for_streak: event.is_mandatory_for_streak,
            });
        }
        Ok(event_objects)
    }
}