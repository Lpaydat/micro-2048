// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Operation infrastructure tests
//! 
//! Unit tests for operation infrastructure including serialization,
//! debug implementation, and field validation patterns.

#![cfg(test)]

use crate::Operation;

/// Test operation serialization support
/// 
/// Tests that operations can be serialized and deserialized properly.
#[test]
fn test_operation_serialization() {
    let register_op = Operation::RegisterPlayer {
        discord_id: "serialization_test_player".to_string(),
        username: "SerializationTest#0001".to_string(),
        avatar_url: Some("https://example.com/avatar.png".to_string()),
    };

    // Test that operation can be converted to and from JSON
    let serialized = serde_json::to_string(&register_op).expect("Should serialize");
    let deserialized: Operation = serde_json::from_str(&serialized).expect("Should deserialize");

    // Verify round-trip serialization
    match deserialized {
        Operation::RegisterPlayer { discord_id, username, avatar_url } => {
            assert_eq!(discord_id, "serialization_test_player");
            assert_eq!(username, "SerializationTest#0001");
            assert_eq!(avatar_url, Some("https://example.com/avatar.png".to_string()));
        }
        _ => panic!("Expected RegisterPlayer operation after deserialization"),
    }
}

/// Test operation debug implementation
/// 
/// Tests that operations can be formatted with Debug trait.
#[test]
fn test_operation_debug_implementation() {
    let op = Operation::RegisterPlayer {
        discord_id: "debug_test_player".to_string(),
        username: "DebugTest#0001".to_string(),
        avatar_url: None,
    };

    // Should be able to format with Debug
    let debug_string = format!("{:?}", op);
    
    // Should contain the operation type and key fields
    assert!(debug_string.contains("RegisterPlayer"));
    assert!(debug_string.contains("debug_test_player"));
    assert!(debug_string.contains("DebugTest#0001"));
}

/// Test operation field validation patterns
/// 
/// Tests common validation patterns for operation fields.
#[test]
fn test_operation_field_validation_patterns() {
    // Test empty string handling
    let empty_id_op = Operation::RegisterPlayer {
        discord_id: "".to_string(),
        username: "ValidUsername#0001".to_string(),
        avatar_url: None,
    };

    // Operation should be created (validation happens at business logic layer)
    match empty_id_op {
        Operation::RegisterPlayer { discord_id, .. } => {
            assert_eq!(discord_id, "");
        }
        _ => panic!("Expected RegisterPlayer operation"),
    }

    // Test very long strings
    let long_string = "a".repeat(1000);
    let long_id_op = Operation::RegisterPlayer {
        discord_id: long_string.clone(),
        username: "ValidUsername#0001".to_string(),
        avatar_url: None,
    };

    match long_id_op {
        Operation::RegisterPlayer { discord_id, .. } => {
            assert_eq!(discord_id.len(), 1000);
        }
        _ => panic!("Expected RegisterPlayer operation"),
    }
}