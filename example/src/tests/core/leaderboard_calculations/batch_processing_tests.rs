// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Batch processing tests for leaderboard calculations
//! 
//! Unit tests for batch update result handling and error processing.

#![cfg(test)]

use crate::core::types::*;
use crate::infrastructure::errors::GameHubError;

#[test]
fn test_batch_update_result_structure() {
    let batch_result = BatchUpdateResult {
        successful_updates: vec!["player1".to_string(), "player2".to_string()],
        failed_updates: vec![
            BatchUpdateError {
                player_discord_id: "player3".to_string(),
                error: GameHubError::PlayerNotFound,
            }
        ],
        unregistered_players: vec!["player4".to_string()],
    };

    assert_eq!(batch_result.successful_updates.len(), 2);
    assert_eq!(batch_result.failed_updates.len(), 1);
    assert_eq!(batch_result.unregistered_players.len(), 1);
    
    assert_eq!(batch_result.successful_updates[0], "player1");
    assert_eq!(batch_result.failed_updates[0].player_discord_id, "player3");
    assert_eq!(batch_result.unregistered_players[0], "player4");
}

#[test]
fn test_batch_update_error_structure() {
    let error = BatchUpdateError {
        player_discord_id: "player123456789012345".to_string(),
        error: GameHubError::InvalidInput { 
            field: "score".to_string(), 
            reason: "Invalid score value".to_string() 
        },
    };

    assert_eq!(error.player_discord_id, "player123456789012345");
    // Test that the error is the expected variant
    match error.error {
        GameHubError::InvalidInput { field, reason } => {
            assert_eq!(field, "score");
            assert_eq!(reason, "Invalid score value");
        },
        _ => panic!("Expected InvalidInput error variant"),
    }
}