// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Player operation tests
//! 
//! Tests for player-related contract operations including registration, profile updates,
//! banning, suspension, and unbanning.

#[cfg(test)]
mod tests {
    use crate::infrastructure::operations::Operation;

    #[test]
    fn test_register_player_operation() {
        // Test basic player registration
        let operation = Operation::RegisterPlayer {
            discord_id: "123456789012345678".to_string(),
            username: "TestPlayer".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
        };

        // Verify operation structure
        match operation {
            Operation::RegisterPlayer { discord_id, username, avatar_url } => {
                assert_eq!(discord_id, "123456789012345678");
                assert_eq!(username, "TestPlayer");
                assert_eq!(avatar_url, Some("https://example.com/avatar.png".to_string()));
            }
            _ => panic!("Expected RegisterPlayer operation"),
        }
    }

    #[test]
    fn test_update_player_profile_operation() {
        // Test player profile update operation
        let operation = Operation::UpdatePlayerProfile {
            discord_id: "123456789012345678".to_string(),
            username: Some("NewUsername".to_string()),
            avatar_url: Some("https://example.com/new-avatar.png".to_string()),
        };

        // Verify operation structure
        match operation {
            Operation::UpdatePlayerProfile { discord_id, username, avatar_url } => {
                assert_eq!(discord_id, "123456789012345678");
                assert_eq!(username, Some("NewUsername".to_string()));
                assert_eq!(avatar_url, Some("https://example.com/new-avatar.png".to_string()));
            }
            _ => panic!("Expected UpdatePlayerProfile operation"),
        }
    }

    #[test]
    fn test_ban_player_operation() {
        // Test player banning operation
        let operation = Operation::BanPlayer {
            caller_discord_id: "123456789012345678".to_string(),
            player_discord_id: "987654321098765432".to_string(),
            reason: "Violation of terms of service".to_string(),
        };

        // Verify operation structure
        match operation {
            Operation::BanPlayer { caller_discord_id, player_discord_id, reason } => {
                assert_eq!(caller_discord_id, "123456789012345678");
                assert_eq!(player_discord_id, "987654321098765432");
                assert_eq!(reason, "Violation of terms of service");
            }
            _ => panic!("Expected BanPlayer operation"),
        }
    }

    #[test]
    fn test_suspend_player_operation() {
        // Test player suspension operation
        let duration = Some(24); // 24 hours
        let operation = Operation::SuspendPlayer {
            caller_discord_id: "123456789012345678".to_string(),
            player_discord_id: "555666777888999000".to_string(),
            reason: "Inappropriate behavior".to_string(),
            duration_hours: duration,
        };

        // Verify operation structure
        match operation {
            Operation::SuspendPlayer { caller_discord_id, player_discord_id, reason, duration_hours } => {
                assert_eq!(caller_discord_id, "123456789012345678");
                assert_eq!(player_discord_id, "555666777888999000");
                assert_eq!(reason, "Inappropriate behavior");
                assert_eq!(duration_hours, Some(24));
            }
            _ => panic!("Expected SuspendPlayer operation"),
        }
    }

    #[test]
    fn test_unban_player_operation() {
        // Test player unbanning operation
        let operation = Operation::UnbanPlayer {
            caller_discord_id: "123456789012345678".to_string(),
            player_discord_id: "111222333444555666".to_string(),
        };

        // Verify operation structure
        match operation {
            Operation::UnbanPlayer { caller_discord_id, player_discord_id } => {
                assert_eq!(caller_discord_id, "123456789012345678");
                assert_eq!(player_discord_id, "111222333444555666");
            }
            _ => panic!("Expected UnbanPlayer operation"),
        }
    }
}