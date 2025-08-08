// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for enhanced Event functionality and GraphQL integration

#[cfg(test)]
mod tests {
    use crate::api::graphql_types::{EventObject, EventStatusType};
    use crate::core::types::{Event, EventStatus};
    use linera_sdk::linera_base_types::Timestamp;
    use serde_json;

    #[test]
    fn test_enhanced_event_object_structure() {
        let event_obj = EventObject {
            id: "event_123".to_string(),
            game_id: "game_456".to_string(),
            name: "Weekly Tournament".to_string(),
            description: "Competitive weekly tournament for all players".to_string(),
            start_time: "1691308800000000".to_string(), // Timestamp as microseconds
            end_time: Some("1691395200000000".to_string()),
            status: EventStatusType::Active,
            max_participants: Some(100),
            prize_pool: Some(5000),
            is_mandatory_for_streak: true,
        };

        assert_eq!(event_obj.id, "event_123");
        assert_eq!(event_obj.game_id, "game_456");
        assert_eq!(event_obj.name, "Weekly Tournament");
        assert_eq!(event_obj.description, "Competitive weekly tournament for all players");
        assert!(event_obj.is_mandatory_for_streak);
        assert_eq!(event_obj.max_participants, Some(100));
        assert_eq!(event_obj.prize_pool, Some(5000));
        assert!(matches!(event_obj.status, EventStatusType::Active));
    }

    #[test]
    fn test_event_object_optional_fields() {
        // Test event with no max participants, no prize pool, and not mandatory for streak
        let optional_event = EventObject {
            id: "casual_event".to_string(),
            game_id: "casual_game".to_string(),
            name: "Casual Play Session".to_string(),
            description: "Relaxed gaming session".to_string(),
            start_time: "1691308800000000".to_string(),
            end_time: None, // Ongoing event
            status: EventStatusType::Upcoming,
            max_participants: None, // No limit
            prize_pool: None, // No prizes
            is_mandatory_for_streak: false,
        };

        assert_eq!(optional_event.id, "casual_event");
        assert!(!optional_event.is_mandatory_for_streak);
        assert_eq!(optional_event.max_participants, None);
        assert_eq!(optional_event.prize_pool, None);
        assert_eq!(optional_event.end_time, None);
        assert!(matches!(optional_event.status, EventStatusType::Upcoming));
    }

    #[test]
    fn test_event_mandatory_for_streak_flag() {
        // Test mandatory event
        let mandatory_event = EventObject {
            id: "mandatory_001".to_string(),
            game_id: "competitive_game".to_string(),
            name: "Championship Final".to_string(),
            description: "Final championship event - attendance required for streak maintenance".to_string(),
            start_time: "1691308800000000".to_string(),
            end_time: Some("1691395200000000".to_string()),
            status: EventStatusType::Active,
            max_participants: Some(50),
            prize_pool: Some(10000),
            is_mandatory_for_streak: true,
        };

        // Test optional event
        let optional_event = EventObject {
            id: "optional_001".to_string(),
            game_id: "casual_game".to_string(),
            name: "Fun Friday".to_string(),
            description: "Optional fun event - no impact on streak".to_string(),
            start_time: "1691308800000000".to_string(),
            end_time: Some("1691395200000000".to_string()),
            status: EventStatusType::Upcoming,
            max_participants: None,
            prize_pool: None,
            is_mandatory_for_streak: false,
        };

        assert!(mandatory_event.is_mandatory_for_streak);
        assert!(!optional_event.is_mandatory_for_streak);
        assert!(mandatory_event.description.contains("required for streak"));
        assert!(optional_event.description.contains("no impact on streak"));
    }

    #[test]
    fn test_event_status_type_conversion() {
        // Test all event status variants
        let upcoming_event = EventObject {
            id: "upcoming".to_string(),
            game_id: "game".to_string(),
            name: "Future Event".to_string(),
            description: "Event scheduled for the future".to_string(),
            start_time: "1691308800000000".to_string(),
            end_time: Some("1691395200000000".to_string()),
            status: EventStatusType::Upcoming,
            max_participants: None,
            prize_pool: None,
            is_mandatory_for_streak: false,
        };

        let active_event = EventObject {
            id: "active".to_string(),
            game_id: "game".to_string(),
            name: "Current Event".to_string(),
            description: "Event currently running".to_string(),
            start_time: "1691308800000000".to_string(),
            end_time: Some("1691395200000000".to_string()),
            status: EventStatusType::Active,
            max_participants: None,
            prize_pool: None,
            is_mandatory_for_streak: true,
        };

        let ended_event = EventObject {
            id: "ended".to_string(),
            game_id: "game".to_string(),
            name: "Past Event".to_string(),
            description: "Event that has concluded".to_string(),
            start_time: "1691308800000000".to_string(),
            end_time: Some("1691395200000000".to_string()),
            status: EventStatusType::Ended,
            max_participants: Some(200),
            prize_pool: Some(2500),
            is_mandatory_for_streak: false,
        };

        let cancelled_event = EventObject {
            id: "cancelled".to_string(),
            game_id: "game".to_string(),
            name: "Cancelled Event".to_string(),
            description: "Event that was cancelled".to_string(),
            start_time: "1691308800000000".to_string(),
            end_time: Some("1691395200000000".to_string()),
            status: EventStatusType::Cancelled,
            max_participants: None,
            prize_pool: None,
            is_mandatory_for_streak: false,
        };

        assert!(matches!(upcoming_event.status, EventStatusType::Upcoming));
        assert!(matches!(active_event.status, EventStatusType::Active));
        assert!(matches!(ended_event.status, EventStatusType::Ended));
        assert!(matches!(cancelled_event.status, EventStatusType::Cancelled));
    }

    #[test]
    fn test_event_object_serialization() {
        let event = EventObject {
            id: "serialization_test".to_string(),
            game_id: "game_test".to_string(),
            name: "Serialization Test Event".to_string(),
            description: "Testing serialization of enhanced event object".to_string(),
            start_time: "1691308800000000".to_string(),
            end_time: Some("1691395200000000".to_string()),
            status: EventStatusType::Active,
            max_participants: Some(75),
            prize_pool: Some(1500),
            is_mandatory_for_streak: true,
        };

        let serialized = serde_json::to_string(&event).expect("Failed to serialize event");
        let deserialized: EventObject = serde_json::from_str(&serialized).expect("Failed to deserialize event");

        assert_eq!(deserialized.id, event.id);
        assert_eq!(deserialized.game_id, event.game_id);
        assert_eq!(deserialized.name, event.name);
        assert_eq!(deserialized.description, event.description);
        assert_eq!(deserialized.is_mandatory_for_streak, event.is_mandatory_for_streak);
        assert_eq!(deserialized.max_participants, event.max_participants);
        assert_eq!(deserialized.prize_pool, event.prize_pool);
    }

    #[test]
    fn test_domain_to_graphql_event_conversion() {
        // Test conversion from domain Event to GraphQL EventObject
        let domain_event = Event {
            id: "domain_event_123".to_string(),
            name: "Domain Event Test".to_string(),
            game_id: "domain_game_456".to_string(),
            description: "Testing domain to GraphQL conversion".to_string(),
            start_time: Timestamp::from(1691308800000000u64),
            end_time: Timestamp::from(1691395200000000u64),
            is_mandatory: true,
            is_mandatory_for_streak: true,
            grace_period_hours: 24,
            max_participants: Some(150),
            prize_pool: Some(7500),
            participant_count: 42,
            created_by: "admin_123".to_string(),
            created_at: Timestamp::from(1691222400000000u64),
            status: EventStatus::Active,
        };

        // Simulate the conversion that happens in the service
        let graphql_event = EventObject {
            id: domain_event.id.clone(),
            game_id: domain_event.game_id.clone(),
            name: domain_event.name.clone(),
            description: domain_event.description.clone(),
            start_time: domain_event.start_time.micros().to_string(),
            end_time: Some(domain_event.end_time.micros().to_string()),
            status: EventStatusType::from(&domain_event.status),
            max_participants: domain_event.max_participants,
            prize_pool: domain_event.prize_pool,
            is_mandatory_for_streak: domain_event.is_mandatory_for_streak,
        };

        assert_eq!(graphql_event.id, "domain_event_123");
        assert_eq!(graphql_event.game_id, "domain_game_456");
        assert_eq!(graphql_event.name, "Domain Event Test");
        assert_eq!(graphql_event.description, "Testing domain to GraphQL conversion");
        assert!(graphql_event.is_mandatory_for_streak);
        assert_eq!(graphql_event.max_participants, Some(150));
        assert_eq!(graphql_event.prize_pool, Some(7500));
        assert!(matches!(graphql_event.status, EventStatusType::Active));
    }

    #[test]
    fn test_event_prize_pool_edge_cases() {
        // Test event with zero prize pool (different from None)
        let zero_prize_event = EventObject {
            id: "zero_prize".to_string(),
            game_id: "game".to_string(),
            name: "Zero Prize Event".to_string(),
            description: "Event with explicitly zero prize pool".to_string(),
            start_time: "1691308800000000".to_string(),
            end_time: Some("1691395200000000".to_string()),
            status: EventStatusType::Active,
            max_participants: Some(100),
            prize_pool: Some(0), // Explicitly zero
            is_mandatory_for_streak: false,
        };

        // Test event with very large prize pool
        let high_prize_event = EventObject {
            id: "high_prize".to_string(),
            game_id: "game".to_string(),
            name: "High Stakes Tournament".to_string(),
            description: "Event with large prize pool".to_string(),
            start_time: "1691308800000000".to_string(),
            end_time: Some("1691395200000000".to_string()),
            status: EventStatusType::Upcoming,
            max_participants: Some(1000),
            prize_pool: Some(1000000), // Large prize
            is_mandatory_for_streak: true,
        };

        assert_eq!(zero_prize_event.prize_pool, Some(0));
        assert_eq!(high_prize_event.prize_pool, Some(1000000));
        assert!(high_prize_event.prize_pool > zero_prize_event.prize_pool);
    }

    #[test]
    fn test_event_participant_limits() {
        // Test event with small participant limit
        let small_event = EventObject {
            id: "small_event".to_string(),
            game_id: "intimate_game".to_string(),
            name: "Small Group Event".to_string(),
            description: "Event for a small group".to_string(),
            start_time: "1691308800000000".to_string(),
            end_time: Some("1691395200000000".to_string()),
            status: EventStatusType::Upcoming,
            max_participants: Some(5),
            prize_pool: Some(100),
            is_mandatory_for_streak: false,
        };

        // Test event with very large participant limit
        let mass_event = EventObject {
            id: "mass_event".to_string(),
            game_id: "popular_game".to_string(),
            name: "Mass Participation Event".to_string(),
            description: "Event for many participants".to_string(),
            start_time: "1691308800000000".to_string(),
            end_time: Some("1691395200000000".to_string()),
            status: EventStatusType::Active,
            max_participants: Some(10000),
            prize_pool: Some(50000),
            is_mandatory_for_streak: true,
        };

        assert_eq!(small_event.max_participants, Some(5));
        assert_eq!(mass_event.max_participants, Some(10000));
        assert!(mass_event.max_participants > small_event.max_participants);
    }
}