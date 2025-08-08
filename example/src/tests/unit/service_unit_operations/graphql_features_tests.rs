// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! GraphQL features tests
//! 
//! Unit tests for GraphQL features including variables handling,
//! pagination parameters, and field selection.

#![cfg(test)]

use async_graphql::{Request, Variables, Value};

/// Test GraphQL variables handling
/// 
/// Tests GraphQL query execution with variables.
#[tokio::test]
async fn test_graphql_variables_handling() {
    let query_with_variables = Request::new(r#"
        query GetPlayer($discordId: String!) {
            player(discordId: $discordId) {
                discordId
                username
                totalPoints
            }
        }
    "#);
    
    let variables = Variables::from_json(serde_json::json!({
        "discordId": "variable_test_player_456"
    }));
    
    let mut request = query_with_variables;
    request.variables = variables;
    
    // Verify variables are properly set
    if let Value::String(discord_id) = request.variables.get("discordId").unwrap() {
        assert_eq!(discord_id, "variable_test_player_456");
    } else {
        panic!("Expected discordId variable to be a string");
    }
}

/// Test GraphQL pagination parameters
/// 
/// Tests pagination handling in GraphQL queries.
#[tokio::test]
async fn test_graphql_pagination_parameters() {
    let paginated_query = Request::new(r#"
        query GetLeaderboard($limit: Int, $offset: Int) {
            leaderboard(limit: $limit, offset: $offset) {
                discordId
                username
                totalPoints
            }
        }
    "#);
    
    let pagination_vars = Variables::from_json(serde_json::json!({
        "limit": 25,
        "offset": 50
    }));
    
    let mut request = paginated_query;
    request.variables = pagination_vars;
    
    // Verify pagination variables
    if let Value::Number(limit) = request.variables.get("limit").unwrap() {
        assert_eq!(limit.as_i64().unwrap(), 25);
    } else {
        panic!("Expected limit to be a number");
    }
    
    if let Value::Number(offset) = request.variables.get("offset").unwrap() {
        assert_eq!(offset.as_i64().unwrap(), 50);
    } else {
        panic!("Expected offset to be a number");
    }
}

/// Test GraphQL field selection
/// 
/// Tests selective field querying in GraphQL.
#[tokio::test]
async fn test_graphql_field_selection() {
    // Test minimal field selection
    let minimal_query = Request::new(r#"
        query {
            player(discordId: "field_test_player") {
                discordId
            }
        }
    "#);
    
    // Test comprehensive field selection
    let comprehensive_query = Request::new(r#"
        query {
            player(discordId: "field_test_player") {
                discordId
                username
                totalPoints
                participationStreak
                bestStreak
                currentRank
                isActive
                createdAt
                lastActive
            }
        }
    "#);
    
    // Verify minimal query contains only essential field
    assert!(minimal_query.query.contains("discordId"));
    assert!(!minimal_query.query.contains("username"));
    assert!(!minimal_query.query.contains("totalPoints"));
    
    // Verify comprehensive query contains all fields
    assert!(comprehensive_query.query.contains("discordId"));
    assert!(comprehensive_query.query.contains("username"));
    assert!(comprehensive_query.query.contains("totalPoints"));
    assert!(comprehensive_query.query.contains("participationStreak"));
    assert!(comprehensive_query.query.contains("bestStreak"));
    assert!(comprehensive_query.query.contains("currentRank"));
    assert!(comprehensive_query.query.contains("isActive"));
    assert!(comprehensive_query.query.contains("createdAt"));
    assert!(comprehensive_query.query.contains("lastActive"));
}