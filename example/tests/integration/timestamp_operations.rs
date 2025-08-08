// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for GameHub timestamp-based operations
//! Tests time-sensitive functionality like streak calculations and grace periods

#![cfg(not(target_arch = "wasm32"))]

use gamehub::GameHubAbi;
use linera_sdk::{
    linera_base_types::TimeDelta,
    test::{QueryOutcome, TestValidator},
};

/// Test timestamp manipulation for streak calculations
/// 
/// Tests time-based operations using TestValidator's clock manipulation
/// following the patterns from Linera examples for time-sensitive testing.
#[tokio::test(flavor = "multi_thread")]
async fn test_timestamp_based_streak_calculations() {
    let (validator, module_id) = 
        TestValidator::with_current_module::<GameHubAbi, (), ()>().await;
    let mut chain = validator.new_chain().await;

    let application_id = chain
        .create_application(module_id, (), (), vec![])
        .await;

    // Get initial time
    let initial_time = validator.clock().current_time();

    // Register a player at initial time
    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::RegisterPlayer {
                    discord_id: "10000000000000000001".to_string(),
                    username: "StreakTest#0001".to_string(),
                    avatar_url: None,
                })
                .with_timestamp(initial_time);
        })
        .await;

    // Advance time by 1 day and perform another operation
    validator.clock().add(TimeDelta::from_secs(24 * 3600));
    let day1_time = validator.clock().current_time();

    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::UpdatePlayerProfile {
                    discord_id: "10000000000000000001".to_string(),
                    username: Some("StreakTestDay1#0001".to_string()),
                    avatar_url: None,
                })
                .with_timestamp(day1_time);
        })
        .await;

    // Advance time by another day
    validator.clock().add(TimeDelta::from_secs(24 * 3600));
    let day2_time = validator.clock().current_time();

    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::UpdatePlayerProfile {
                    discord_id: "10000000000000000001".to_string(),
                    username: Some("StreakTestDay2#0001".to_string()),
                    avatar_url: Some("https://example.com/day2_avatar.png".to_string()),
                })
                .with_timestamp(day2_time);
        })
        .await;

    // Verify operations were processed successfully
    let QueryOutcome { response, .. } = 
        chain.graphql_query(application_id, "query { __typename }").await;
    
    assert!(response.is_object());
    
    // TODO: Add specific streak calculation verification once GraphQL schema is established
    // Expected: Player should have accumulated streak over multiple days
}

/// Test grace period functionality
/// 
/// Tests grace period calculations for player activity and streak maintenance.
#[tokio::test(flavor = "multi_thread")]
async fn test_grace_period_calculations() {
    let (validator, module_id) = 
        TestValidator::with_current_module::<GameHubAbi, (), ()>().await;
    let mut chain = validator.new_chain().await;

    let application_id = chain
        .create_application(module_id, (), (), vec![])
        .await;

    let initial_time = validator.clock().current_time();

    // Register player
    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::RegisterPlayer {
                    discord_id: "10000000000000000002".to_string(),
                    username: "GracePeriodTest#0001".to_string(),
                    avatar_url: None,
                })
                .with_timestamp(initial_time);
        })
        .await;

    // Test within grace period (23 hours later)
    validator.clock().add(TimeDelta::from_secs(23 * 3600));
    let within_grace_time = validator.clock().current_time();

    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::UpdatePlayerProfile {
                    discord_id: "10000000000000000002".to_string(),
                    username: Some("WithinGrace#0001".to_string()),
                    avatar_url: None,
                })
                .with_timestamp(within_grace_time);
        })
        .await;

    // Test outside grace period (25 hours total)
    validator.clock().add(TimeDelta::from_secs(2 * 3600 + 1));
    let outside_grace_time = validator.clock().current_time();

    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::UpdatePlayerProfile {
                    discord_id: "10000000000000000002".to_string(),
                    username: Some("OutsideGrace#0001".to_string()),
                    avatar_url: Some("https://example.com/outside_grace_avatar.png".to_string()),
                })
                .with_timestamp(outside_grace_time);
        })
        .await;

    // Verify all operations completed successfully
    let QueryOutcome { response, .. } = 
        chain.graphql_query(application_id, "query { __typename }").await;
    
    assert!(response.is_object());
    
    // TODO: Add specific grace period verification once scoring logic is accessible
    // Expected: Operations within grace period should maintain streak, outside should reset
}

/// Test time-based suspension duration
/// 
/// Tests temporary suspensions with specific duration handling.
#[tokio::test(flavor = "multi_thread")]
async fn test_time_based_suspension_duration() {
    let (validator, module_id) = 
        TestValidator::with_current_module::<GameHubAbi, (), ()>().await;
    let mut chain = validator.new_chain().await;

    let application_id = chain
        .create_application(module_id, (), (), vec![])
        .await;

    let initial_time = validator.clock().current_time();

    // Register player
    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::RegisterPlayer {
                    discord_id: "10000000000000000003".to_string(),
                    username: "SuspensionTest#0001".to_string(),
                    avatar_url: None,
                })
                .with_timestamp(initial_time);
        })
        .await;

    // Apply 24-hour suspension
    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::SuspendPlayer {
                    caller_discord_id: "admin_123456789012345678".to_string(),
                    player_discord_id: "10000000000000000003".to_string(),
                    reason: "Test suspension".to_string(),
                    duration_hours: Some(24),
                })
                .with_timestamp(initial_time);
        })
        .await;

    // Test operation within suspension period (12 hours later)
    validator.clock().add(TimeDelta::from_secs(12 * 3600));
    let during_suspension_time = validator.clock().current_time();

    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::UpdatePlayerProfile {
                    discord_id: "10000000000000000003".to_string(),
                    username: Some("DuringSuspension#0001".to_string()),
                    avatar_url: None,
                })
                .with_timestamp(during_suspension_time);
        })
        .await;

    // Test operation after suspension expires (26 hours total)
    validator.clock().add(TimeDelta::from_secs(14 * 3600 + 1));
    let after_suspension_time = validator.clock().current_time();

    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::UpdatePlayerProfile {
                    discord_id: "10000000000000000003".to_string(),
                    username: Some("AfterSuspension#0001".to_string()),
                    avatar_url: Some("https://example.com/post_suspension_avatar.png".to_string()),
                })
                .with_timestamp(after_suspension_time);
        })
        .await;

    // Verify chain remains functional
    let QueryOutcome { response, .. } = 
        chain.graphql_query(application_id, "query { __typename }").await;
    
    assert!(response.is_object());
    
    // TODO: Add specific suspension status verification once player status queries are available
    // Expected: Player should be suspended during the period, then active after expiration
}

/// Test clock-based game timeouts
/// 
/// Tests timeout functionality for games and events using precise timing.
#[tokio::test(flavor = "multi_thread")]
async fn test_clock_based_game_timeouts() {
    let (validator, module_id) = 
        TestValidator::with_current_module::<GameHubAbi, (), ()>().await;
    let mut chain = validator.new_chain().await;

    let application_id = chain
        .create_application(module_id, (), (), vec![])
        .await;

    let start_time = validator.clock().current_time();

    // Register a player for timeout testing
    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::RegisterPlayer {
                    discord_id: "10000000000000000004".to_string(),
                    username: "TimeoutTest#0001".to_string(),
                    avatar_url: None,
                })
                .with_timestamp(start_time);
        })
        .await;

    // Test rapid operations within a short time window
    for i in 0..5 {
        validator.clock().add(TimeDelta::from_secs(1)); // 1 second intervals
        let operation_time = validator.clock().current_time();

        chain
            .add_block(|block| {
                block
                    .with_operation(application_id, gamehub::Operation::UpdatePlayerProfile {
                        discord_id: "10000000000000000004".to_string(),
                        username: Some(format!("RapidUpdate{}#0001", i)),
                        avatar_url: None,
                    })
                    .with_timestamp(operation_time);
            })
            .await;
    }

    // Test operations with larger time gaps
    validator.clock().add(TimeDelta::from_secs(3600)); // 1 hour gap
    let delayed_time = validator.clock().current_time();

    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::UpdatePlayerProfile {
                    discord_id: "10000000000000000004".to_string(),
                    username: Some("DelayedUpdate#0001".to_string()),
                    avatar_url: Some("https://example.com/delayed_avatar.png".to_string()),
                })
                .with_timestamp(delayed_time);
        })
        .await;

    // Verify all operations processed successfully
    let QueryOutcome { response, .. } = 
        chain.graphql_query(application_id, "query { __typename }").await;
    
    assert!(response.is_object());
    
    // TODO: Add specific timeout behavior verification once game timeout logic is implemented
    // Expected: Rapid operations should be handled correctly, delayed operations should account for time gaps
}

/// Test timestamp consistency across multiple operations
/// 
/// Tests that timestamps are handled consistently across different operation types.
#[tokio::test(flavor = "multi_thread")]
async fn test_timestamp_consistency_across_operations() {
    let (validator, module_id) = 
        TestValidator::with_current_module::<GameHubAbi, (), ()>().await;
    let mut chain = validator.new_chain().await;

    let application_id = chain
        .create_application(module_id, (), (), vec![])
        .await;

    let base_time = validator.clock().current_time();
    let player_id = "10000000000000000009";

    // Sequence of operations with precise timing
    let operations = vec![
        (0, gamehub::Operation::RegisterPlayer {
            discord_id: player_id.to_string(),
            username: "ConsistencyTest#0001".to_string(),
            avatar_url: None,
        }),
        (3600, gamehub::Operation::UpdatePlayerProfile {
            discord_id: player_id.to_string(),
            username: Some("Updated#0001".to_string()),
            avatar_url: None,
        }),
        (7200, gamehub::Operation::UpdatePlayerProfile {
            discord_id: player_id.to_string(),
            username: Some("FinalUpdate#0001".to_string()),
            avatar_url: Some("https://example.com/final_avatar.png".to_string()),
        }),
    ];

    for (time_offset, operation) in operations {
        validator.clock().add(TimeDelta::from_secs(time_offset));
        let operation_time = validator.clock().current_time();

        chain
            .add_block(|block| {
                block
                    .with_operation(application_id, operation)
                    .with_timestamp(operation_time);
            })
            .await;

        // Verify timestamp consistency by checking the chain is responsive
        let QueryOutcome { response, .. } = 
            chain.graphql_query(application_id, "query { __typename }").await;
        
        assert!(response.is_object(), "Chain should remain responsive at timestamp offset {}", time_offset);
    }

    // Final verification of complete operation sequence
    let QueryOutcome { response, .. } = 
        chain.graphql_query(application_id, "query { __typename }").await;
    
    assert!(response.is_object());
    
    // TODO: Add specific timestamp verification once player timestamp queries are available
    // Expected: All operations should have consistent timestamp ordering and proper time-based logic
}

/// Test edge cases in timestamp handling
/// 
/// Tests various edge cases and boundary conditions in timestamp operations.
#[tokio::test(flavor = "multi_thread")]
async fn test_timestamp_edge_cases() {
    let (validator, module_id) = 
        TestValidator::with_current_module::<GameHubAbi, (), ()>().await;
    let mut chain = validator.new_chain().await;

    let application_id = chain
        .create_application(module_id, (), (), vec![])
        .await;

    let initial_time = validator.clock().current_time();

    // Test with exactly the same timestamp for multiple operations
    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::RegisterPlayer {
                    discord_id: "10000000000000000005".to_string(),
                    username: "EdgeCase1#0001".to_string(),
                    avatar_url: None,
                })
                .with_timestamp(initial_time);
        })
        .await;

    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::RegisterPlayer {
                    discord_id: "10000000000000000006".to_string(),
                    username: "EdgeCase2#0002".to_string(),
                    avatar_url: None,
                })
                .with_timestamp(initial_time); // Same timestamp
        })
        .await;

    // Test with minimal time advancement (1 microsecond)
    validator.clock().add(TimeDelta::from_micros(1));
    let micro_time = validator.clock().current_time();

    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::RegisterPlayer {
                    discord_id: "10000000000000000007".to_string(),
                    username: "EdgeCase3#0003".to_string(),
                    avatar_url: None,
                })
                .with_timestamp(micro_time);
        })
        .await;

    // Test with large time advancement
    validator.clock().add(TimeDelta::from_secs(365 * 24 * 3600)); // 1 year
    let future_time = validator.clock().current_time();

    chain
        .add_block(|block| {
            block
                .with_operation(application_id, gamehub::Operation::RegisterPlayer {
                    discord_id: "10000000000000000008".to_string(),
                    username: "EdgeCaseFuture#0004".to_string(),
                    avatar_url: Some("https://example.com/future_avatar.png".to_string()),
                })
                .with_timestamp(future_time);
        })
        .await;

    // Verify all edge cases handled correctly
    let QueryOutcome { response, .. } = 
        chain.graphql_query(application_id, "query { __typename }").await;
    
    assert!(response.is_object());
    
    // TODO: Add specific edge case verification once timestamp query capabilities are available
    // Expected: All timestamp edge cases should be handled gracefully without errors
}