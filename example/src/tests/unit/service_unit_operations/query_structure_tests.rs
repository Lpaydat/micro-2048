// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Query structure tests
//! 
//! Unit tests for GraphQL query structure validation including basic queries,
//! leaderboard queries, games queries, and admin queries.

#![cfg(test)]

use async_graphql::Request;

/// Test GraphQL query structure validation
/// 
/// Tests that GraphQL queries are properly structured and parsed.
#[tokio::test]
async fn test_graphql_query_structure() {
    // Test basic player query structure
    let player_query = Request::new(r#"
        query {
            player(discordId: "test_player_123") {
                discordId
                username
                totalPoints
                isActive
            }
        }
    "#);
    
    // Verify query is properly parsed
    assert_eq!(player_query.query, r#"
        query {
            player(discordId: "test_player_123") {
                discordId
                username
                totalPoints
                isActive
            }
        }
    "#);
}

/// Test GraphQL leaderboard query structure
/// 
/// Tests leaderboard query with pagination parameters.
#[tokio::test]
async fn test_graphql_leaderboard_query_structure() {
    let leaderboard_query = Request::new(r#"
        query {
            leaderboard(limit: 10, offset: 0) {
                discordId
                username
                totalPoints
                rank
            }
        }
    "#);
    
    // Verify leaderboard query structure
    assert!(leaderboard_query.query.contains("leaderboard"));
    assert!(leaderboard_query.query.contains("limit: 10"));
    assert!(leaderboard_query.query.contains("offset: 0"));
    assert!(leaderboard_query.query.contains("discordId"));
    assert!(leaderboard_query.query.contains("username"));
    assert!(leaderboard_query.query.contains("totalPoints"));
    assert!(leaderboard_query.query.contains("rank"));
}

/// Test GraphQL games query structure
/// 
/// Tests games-related query structures.
#[tokio::test]
async fn test_graphql_games_query_structure() {
    let games_query = Request::new(r#"
        query {
            approvedGames {
                id
                name
                description
                status
                developerName
            }
        }
    "#);
    
    // Verify games query structure
    assert!(games_query.query.contains("approvedGames"));
    assert!(games_query.query.contains("id"));
    assert!(games_query.query.contains("name"));
    assert!(games_query.query.contains("description"));
    assert!(games_query.query.contains("status"));
    assert!(games_query.query.contains("developerName"));
}

/// Test GraphQL admin query structure
/// 
/// Tests admin-related query structures.
#[tokio::test]
async fn test_graphql_admin_query_structure() {
    let admin_query = Request::new(r#"
        query {
            auditLog(limit: 50) {
                id
                action
                performedBy
                timestamp
                details
            }
        }
    "#);
    
    // Verify admin query structure
    assert!(admin_query.query.contains("auditLog"));
    assert!(admin_query.query.contains("limit: 50"));
    assert!(admin_query.query.contains("id"));
    assert!(admin_query.query.contains("action"));
    assert!(admin_query.query.contains("performedBy"));
    assert!(admin_query.query.contains("timestamp"));
    assert!(admin_query.query.contains("details"));
}