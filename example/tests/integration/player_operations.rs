// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for GameHub player operations
//! Tests player registration, management, and state changes using TestValidator

#![cfg(not(target_arch = "wasm32"))]

use gamehub::{GameHubAbi, Operation};
use linera_sdk::test::{QueryOutcome, TestValidator};

/// Test player registration workflow
/// 
/// Tests the complete player registration process through blockchain operations
/// following the TestValidator pattern for chain-level integration testing.
#[tokio::test(flavor = "multi_thread")]
async fn test_player_registration_workflow() {
    let (validator, module_id) = 
        TestValidator::with_current_module::<GameHubAbi, (), ()>().await;
    let mut chain = validator.new_chain().await;

    let application_id = chain
        .create_application(module_id, (), (), vec![])
        .await;

    // Create player registration operation
    let register_operation = Operation::RegisterPlayer {
        discord_id: "123456789012345678".to_string(),
        username: "TestPlayer#1234".to_string(),
        avatar_url: Some("https://cdn.discordapp.com/avatars/123456789012345678/avatar.png".to_string()),
    };

    // Execute player registration through blockchain operation
    chain
        .add_block(|block| {
            block.with_operation(application_id, register_operation);
        })
        .await;

    // Verify player was registered by querying state
    let QueryOutcome { response, .. } = 
        chain.graphql_query(
            application_id, 
            "query { playerExists(discordId: \"123456789012345678\") }"
        ).await;

    // Verify player registration was successful
    assert!(response["playerExists"].as_bool() == Some(true));
    
    // Also verify player details using player query
    let QueryOutcome { response, .. } = 
        chain.graphql_query(
            application_id, 
            "query { player(discordId: \"123456789012345678\") { discordId username } }"
        ).await;
    
    assert!(response["player"]["discordId"].as_str() == Some("123456789012345678"));
    assert!(response["player"]["username"].as_str() == Some("TestPlayer#1234"));
    
    // TODO: Add specific player data verification once GraphQL schema is established
    // Expected: Player with discord_id "123456789012345678" should exist in response
}

/// Test player profile updates
/// 
/// Tests updating player information through blockchain operations.
#[tokio::test(flavor = "multi_thread")]
async fn test_player_profile_updates() {
    let (validator, module_id) = 
        TestValidator::with_current_module::<GameHubAbi, (), ()>().await;
    let mut chain = validator.new_chain().await;

    let application_id = chain
        .create_application(module_id, (), (), vec![])
        .await;

    let player_id = "123456789012345678";

    // First, register a player
    let register_operation = Operation::RegisterPlayer {
        discord_id: player_id.to_string(),
        username: "OriginalName#1234".to_string(),
        avatar_url: None,
    };

    chain
        .add_block(|block| {
            block.with_operation(application_id, register_operation);
        })
        .await;

    // Then update the player's profile
    let update_operation = Operation::UpdatePlayerProfile {
        discord_id: player_id.to_string(),
        username: Some("UpdatedName#5678".to_string()),
        avatar_url: Some("https://cdn.discordapp.com/avatars/123456789012345678/new_avatar.png".to_string()),
    };

    chain
        .add_block(|block| {
            block.with_operation(application_id, update_operation); 
        })
        .await;

    // Verify profile was updated
    let QueryOutcome { response, .. } = 
        chain.graphql_query(
            application_id,
            "query { player(discordId: \"123456789012345678\") { discordId username avatarUrl } }"
        ).await;

    // Verify profile changes
    assert!(response["player"]["discordId"].as_str() == Some("123456789012345678"));
    assert!(response["player"]["username"].as_str() == Some("UpdatedName#5678"));
    assert!(response["player"]["avatarUrl"].as_str() == Some("https://cdn.discordapp.com/avatars/123456789012345678/new_avatar.png"));
    // Expected: Player should have updated username and avatar_url
}

/// Test player moderation operations
/// 
/// Tests player banning, suspension, and unbanning workflows.
#[tokio::test(flavor = "multi_thread")]
async fn test_player_moderation_workflow() {
    let (validator, module_id) = 
        TestValidator::with_current_module::<GameHubAbi, (), ()>().await;
    let mut chain = validator.new_chain().await;

    let application_id = chain
        .create_application(module_id, (), (), vec![])
        .await;

    let player_id = "11223344556677889900";

    // Register a player first
    chain
        .add_block(|block| {
            block.with_operation(application_id, Operation::RegisterPlayer {
                discord_id: player_id.to_string(),
                username: "TroubleMaker#9999".to_string(),
                avatar_url: None,
            });
        })
        .await;

    // Test player suspension (use system admin ID that is initialized in contract)
    let suspend_operation = Operation::SuspendPlayer {
        caller_discord_id: "123456789012345678".to_string(), // System admin from contract initialization
        player_discord_id: player_id.to_string(),
        reason: "Inappropriate conduct".to_string(),
        duration_hours: Some(24),
    };

    chain
        .add_block(|block| {
            block.with_operation(application_id, suspend_operation);
        })
        .await;

    // Verify suspension
    let QueryOutcome { response, .. } = 
        chain.graphql_query(
            application_id,
            "query { player(discordId: \"11223344556677889900\") { discordId status } }"
        ).await;

    // Verify player is suspended
    assert!(response["player"]["discordId"].as_str() == Some("11223344556677889900"));
    assert!(response["player"]["status"].as_str() == Some("SUSPENDED"));

    // Test player banning
    let ban_operation = Operation::BanPlayer {
        caller_discord_id: "123456789012345678".to_string(), // System admin from contract initialization
        player_discord_id: player_id.to_string(),
        reason: "Repeated offenses".to_string(),
    };

    chain
        .add_block(|block| {
            block.with_operation(application_id, ban_operation);
        })
        .await;

    // Test unbanning
    let unban_operation = Operation::UnbanPlayer {
        caller_discord_id: "123456789012345678".to_string(), // System admin from contract initialization
        player_discord_id: player_id.to_string(),
    };

    chain
        .add_block(|block| {
            block.with_operation(application_id, unban_operation);
        })
        .await;

    // Verify final state - player should be unbanned (Active after unban)
    let QueryOutcome { response, .. } = 
        chain.graphql_query(
            application_id,
            "query { player(discordId: \"11223344556677889900\") { discordId status } }"
        ).await;
        
    // Verify player is unbanned (Active status)
    assert!(response["player"]["discordId"].as_str() == Some("11223344556677889900"));
    assert!(response["player"]["status"].as_str() == Some("ACTIVE"));
}

/// Test multiple player operations in single block
/// 
/// Tests batch processing of multiple player operations within a single block.
#[tokio::test(flavor = "multi_thread")]
async fn test_batch_player_operations() {
    let (validator, module_id) = 
        TestValidator::with_current_module::<GameHubAbi, (), ()>().await;
    let mut chain = validator.new_chain().await;

    let application_id = chain
        .create_application(module_id, (), (), vec![])
        .await;

    // Register multiple players in a single block
    chain
        .add_block(|block| {
            // Register player 1
            block.with_operation(application_id, Operation::RegisterPlayer {
                discord_id: "11111111111111111111".to_string(),
                username: "Player1#0001".to_string(),
                avatar_url: None,
            });
            
            // Register player 2
            block.with_operation(application_id, Operation::RegisterPlayer {
                discord_id: "22222222222222222222".to_string(),
                username: "Player2#0002".to_string(),
                avatar_url: None,
            });
            
            // Register player 3
            block.with_operation(application_id, Operation::RegisterPlayer {
                discord_id: "33333333333333333333".to_string(),
                username: "Player3#0003".to_string(),
                avatar_url: None,
            });
        })
        .await;

    // Verify all players were registered using leaderboard query
    let QueryOutcome { response, .. } = 
        chain.graphql_query(
            application_id,
            "query { leaderboard(limit: 10) { playerDiscordId playerUsername } }"
        ).await;

    // Verify we have players in the leaderboard
    assert!(response["leaderboard"].is_array());
    // Should have at least some players registered
    assert!(response["leaderboard"].as_array().unwrap().len() > 0);
    // Expected: Response should contain all three registered players
}

/// Test player operation error scenarios
/// 
/// Tests various error conditions in player operations to ensure proper handling.
#[tokio::test(flavor = "multi_thread")]
async fn test_player_operation_errors() {
    let (validator, module_id) = 
        TestValidator::with_current_module::<GameHubAbi, (), ()>().await;
    let mut chain = validator.new_chain().await;

    let application_id = chain
        .create_application(module_id, (), (), vec![])
        .await;

    // Test operations with invalid player IDs
    // Note: Error handling will depend on how the contract is implemented
    
    // Attempt to update non-existent player
    let invalid_update = Operation::UpdatePlayerProfile {
        discord_id: "99999999999999999999".to_string(),
        username: Some("NewName#1234".to_string()),
        avatar_url: None,
    };

    // This should either fail or be handled gracefully by the contract
    chain
        .add_block(|block| {
            block.with_operation(application_id, invalid_update);
        })
        .await;

    // Verify contract is still functional after error
    let QueryOutcome { response, .. } = 
        chain.graphql_query(application_id, "query { __typename }").await;
    
    assert!(response.is_object());
    
    // TODO: Add specific error scenario testing once contract error handling is defined
}

/// Test player state consistency across operations
/// 
/// Verifies that player state remains consistent through multiple operations.
#[tokio::test(flavor = "multi_thread")]
async fn test_player_state_consistency() {
    let (validator, module_id) = 
        TestValidator::with_current_module::<GameHubAbi, (), ()>().await;
    let mut chain = validator.new_chain().await;

    let application_id = chain
        .create_application(module_id, (), (), vec![])
        .await;

    let player_id = "consistency_test_player";

    // Perform a series of operations on the same player
    let operations = vec![
        Operation::RegisterPlayer {
            discord_id: player_id.to_string(),
            username: "ConsistencyTest#0001".to_string(),
            avatar_url: None,
        },
        Operation::UpdatePlayerProfile {
            discord_id: player_id.to_string(),
            username: Some("UpdatedConsistency#0001".to_string()),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
        },
        Operation::SuspendPlayer {
            caller_discord_id: "123456789012345678".to_string(), // System admin from contract initialization
            player_discord_id: player_id.to_string(),
            reason: "Test suspension".to_string(),
            duration_hours: Some(1),
        },
        Operation::UnbanPlayer {
            caller_discord_id: "123456789012345678".to_string(), // System admin from contract initialization
            player_discord_id: player_id.to_string(),
        },
    ];

    for operation in operations {
        chain
            .add_block(|block| {
                block.with_operation(application_id, operation);
            })
            .await;

        // Verify contract remains responsive after each operation
        let QueryOutcome { response, .. } = 
            chain.graphql_query(application_id, "query { __typename }").await;
        
        assert!(response.is_object());
    }

    // Final state verification - check system health
    let QueryOutcome { response, .. } = 
        chain.graphql_query(
            application_id,
            "query { systemHealth { totalPlayers } }"
        ).await;

    // Verify system is responsive and has player count
    assert!(response["systemHealth"]["totalPlayers"].as_u64().is_some());
}