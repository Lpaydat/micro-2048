// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Response structure tests
//! 
//! Unit tests for GraphQL response structure validation including basic responses,
//! leaderboard responses, and error responses.

#![cfg(test)]

use async_graphql::{Response, Value, ServerError};
use serde_json::json;

/// Test GraphQL response structure
/// 
/// Tests basic GraphQL response data structure.
#[tokio::test]
async fn test_graphql_response_structure() {
    // Create mock response data
    let response_data = json!({
        "player": {
            "discordId": "response_test_player",
            "username": "ResponseTest#0001",
            "totalPoints": 150,
            "isActive": true,
            "rank": 5
        }
    });
    
    let response = Response::new(Value::from_json(response_data).unwrap());
    
    // Verify response structure
    match response.data {
        Value::Object(data) => {
            // Verify player object exists
            if let Value::Object(player) = data.get("player").unwrap() {
                // Verify all expected fields
                assert!(player.contains_key("discordId"));
                assert!(player.contains_key("username"));
                assert!(player.contains_key("totalPoints"));
                assert!(player.contains_key("isActive"));
                assert!(player.contains_key("rank"));
                
                // Verify field values
                if let Value::String(discord_id) = player.get("discordId").unwrap() {
                    assert_eq!(discord_id, "response_test_player");
                }
                if let Value::String(username) = player.get("username").unwrap() {
                    assert_eq!(username, "ResponseTest#0001");
                }
                if let Value::Number(points) = player.get("totalPoints").unwrap() {
                    assert_eq!(points.as_i64().unwrap(), 150);
                }
                if let Value::Boolean(is_active) = player.get("isActive").unwrap() {
                    assert!(*is_active);
                }
                if let Value::Number(rank) = player.get("rank").unwrap() {
                    assert_eq!(rank.as_i64().unwrap(), 5);
                }
            } else {
                panic!("Expected player to be an object");
            }
        }
        _ => panic!("Expected response data to be an object"),
    }
}

/// Test GraphQL leaderboard response structure
/// 
/// Tests leaderboard response data structure with rankings.
#[tokio::test]
async fn test_graphql_leaderboard_response_structure() {
    // Create mock leaderboard response
    let leaderboard_data = json!({
        "leaderboard": [
            {
                "discordId": "leader1",
                "username": "Leader1#0001",
                "totalPoints": 250,
                "rank": 1
            },
            {
                "discordId": "leader2", 
                "username": "Leader2#0002",
                "totalPoints": 200,
                "rank": 2
            }
        ]
    });
    
    let response = Response::new(Value::from_json(leaderboard_data).unwrap());
    
    // Verify leaderboard response structure
    match response.data {
        Value::Object(data) => {
            if let Value::List(leaderboard) = data.get("leaderboard").unwrap() {
                assert_eq!(leaderboard.len(), 2);
                
                // Check first player
                if let Value::Object(first_player) = &leaderboard[0] {
                    // Verify first player structure
                    assert!(first_player.contains_key("rank"));
                    assert!(first_player.contains_key("totalPoints"));
                    
                    if let Value::Number(rank) = first_player.get("rank").unwrap() {
                        assert_eq!(rank.as_i64().unwrap(), 1);
                    }
                    if let Value::Number(points) = first_player.get("totalPoints").unwrap() {
                        assert_eq!(points.as_i64().unwrap(), 250);
                    }
                } else {
                    panic!("Expected first player to be an object");
                }
                
                // Check second player
                if let Value::Object(second_player) = &leaderboard[1] {
                    // Verify second player structure
                    assert!(second_player.contains_key("rank"));
                    assert!(second_player.contains_key("totalPoints"));
                    
                    if let Value::Number(rank) = second_player.get("rank").unwrap() {
                        assert_eq!(rank.as_i64().unwrap(), 2);
                    }
                    if let Value::Number(points) = second_player.get("totalPoints").unwrap() {
                        assert_eq!(points.as_i64().unwrap(), 200);
                    }
                } else {
                    panic!("Expected second player to be an object");
                }
            } else {
                panic!("Expected leaderboard to be a list");
            }
        }
        _ => panic!("Expected response data to be an object"),
    }
}

/// Test GraphQL error response structure
/// 
/// Tests error handling in GraphQL responses.
#[tokio::test]
async fn test_graphql_error_response_structure() {
    // Create response with errors using ServerError
    let mut response = Response::new(Value::Null);
    response.errors.push(ServerError::new("Player not found", None));
    response.errors.push(ServerError::new("Permission denied", None));
    
    // Verify error structure
    assert_eq!(response.errors.len(), 2);
    assert_eq!(response.errors[0].message, "Player not found");
    assert_eq!(response.errors[1].message, "Permission denied");
    
    // Data should be null when errors occur
    match response.data {
        Value::Null => {}, // Expected
        _ => panic!("Expected response data to be null when errors occur"),
    }
}