// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Game operation tests
//! 
//! Tests for game-related contract operations including approval, rejection,
//! suspension, reactivation, and deprecation.

#[cfg(test)]
mod tests {
    use crate::infrastructure::operations::Operation;

    #[test]
    fn test_approve_game_operation() {
        // Test game approval operation
        let operation = Operation::ApproveGame {
            caller_discord_id: "123456789012345678".to_string(),
            game_id: "test-game-123".to_string(),
        };

        // Verify operation structure
        match operation {
            Operation::ApproveGame { caller_discord_id, game_id } => {
                assert_eq!(caller_discord_id, "123456789012345678");
                assert_eq!(game_id, "test-game-123");
            }
            _ => panic!("Expected ApproveGame operation"),
        }
    }

    #[test]
    fn test_reject_game_operation() {
        // Test game rejection operation
        let operation = Operation::RejectGame {
            caller_discord_id: "123456789012345678".to_string(),
            game_id: "test-game-456".to_string(),
            reason: "Game does not meet quality standards".to_string(),
        };

        // Verify operation structure
        match operation {
            Operation::RejectGame { caller_discord_id, game_id, reason } => {
                assert_eq!(caller_discord_id, "123456789012345678");
                assert_eq!(game_id, "test-game-456");
                assert_eq!(reason, "Game does not meet quality standards");
            }
            _ => panic!("Expected RejectGame operation"),
        }
    }

    #[test]
    fn test_game_management_operations() {
        // Test game suspension operation
        let suspend_op = Operation::SuspendGame {
            caller_discord_id: "123456789012345678".to_string(),
            game_id: "game123".to_string(),
            reason: "Quality issues reported".to_string(),
        };

        match suspend_op {
            Operation::SuspendGame { caller_discord_id, game_id, reason } => {
                assert_eq!(caller_discord_id, "123456789012345678");
                assert_eq!(game_id, "game123");
                assert_eq!(reason, "Quality issues reported");
            }
            _ => panic!("Expected SuspendGame operation"),
        }

        // Test game reactivation operation
        let reactivate_op = Operation::ReactivateGame {
            caller_discord_id: "123456789012345678".to_string(),
            game_id: "game456".to_string(),
        };

        match reactivate_op {
            Operation::ReactivateGame { caller_discord_id, game_id } => {
                assert_eq!(caller_discord_id, "123456789012345678");
                assert_eq!(game_id, "game456");
            }
            _ => panic!("Expected ReactivateGame operation"),
        }

        // Test game deprecation operation
        let deprecate_op = Operation::DeprecateGame {
            caller_discord_id: "123456789012345678".to_string(),
            game_id: "game789".to_string(),
        };

        match deprecate_op {
            Operation::DeprecateGame { caller_discord_id, game_id } => {
                assert_eq!(caller_discord_id, "123456789012345678");
                assert_eq!(game_id, "game789");
            }
            _ => panic!("Expected DeprecateGame operation"),
        }
    }
}