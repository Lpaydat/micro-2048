// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! General validation utilities

use crate::infrastructure::errors::GameHubError;

/// Maximum lengths for general fields
pub const MAX_REASON_LENGTH: usize = 500;
pub const MIN_REASON_LENGTH: usize = 1;

/// General validation utilities
pub struct GeneralValidator;

impl GeneralValidator {
    /// Validate reason text (for bans, suspensions, etc.)
    pub fn validate_reason(reason: &str) -> Result<(), GameHubError> {
        if reason.is_empty() {
            return Err(GameHubError::MissingRequiredField {
                field: "reason".to_string(),
            });
        }

        let trimmed = reason.trim();
        if trimmed.is_empty() {
            return Err(GameHubError::InvalidReason {
                reason: "Reason cannot be only whitespace".to_string(),
            });
        }

        if trimmed.len() < MIN_REASON_LENGTH {
            return Err(GameHubError::InputTooShort {
                field: "reason".to_string(),
                min_length: MIN_REASON_LENGTH,
            });
        }

        if trimmed.len() > MAX_REASON_LENGTH {
            return Err(GameHubError::InputTooLong {
                field: "reason".to_string(),
                max_length: MAX_REASON_LENGTH,
            });
        }

        // Check for prohibited content or patterns
        if trimmed.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
            return Err(GameHubError::InvalidReason {
                reason: "Reason cannot contain control characters".to_string(),
            });
        }

        Ok(())
    }

    /// Validate no empty strings in a collection
    pub fn validate_no_empty_strings(strings: &[String], field_name: &str) -> Result<(), GameHubError> {
        for (index, s) in strings.iter().enumerate() {
            if s.trim().is_empty() {
                return Err(GameHubError::EmptyStringInCollection {
                    field: field_name.to_string(),
                    index,
                });
            }
        }
        Ok(())
    }

    /// Sanitize text input by removing/replacing potentially harmful characters
    pub fn sanitize_text_input(input: &str) -> String {
        input
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Sanitize HTML content by removing HTML tags (basic implementation)
    pub fn sanitize_html_content(input: &str) -> String {
        // This is a basic implementation. In production, consider using a proper HTML sanitization library
        // Important: Replace & first to avoid double encoding
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    /// Validate that a string is not empty after trimming
    pub fn validate_non_empty_trimmed(value: &str, field_name: &str) -> Result<(), GameHubError> {
        if value.trim().is_empty() {
            return Err(GameHubError::MissingRequiredField {
                field: field_name.to_string(),
            });
        }
        Ok(())
    }

    /// Validate string length bounds
    pub fn validate_string_length(
        value: &str,
        field_name: &str,
        min_length: Option<usize>,
        max_length: Option<usize>,
    ) -> Result<(), GameHubError> {
        let trimmed = value.trim();
        
        if let Some(min) = min_length {
            if trimmed.len() < min {
                return Err(GameHubError::InputTooShort {
                    field: field_name.to_string(),
                    min_length: min,
                });
            }
        }

        if let Some(max) = max_length {
            if trimmed.len() > max {
                return Err(GameHubError::InputTooLong {
                    field: field_name.to_string(),
                    max_length: max,
                });
            }
        }

        Ok(())
    }

    /// Validate that a collection is not empty
    pub fn validate_non_empty_collection<T>(
        collection: &[T],
        field_name: &str,
    ) -> Result<(), GameHubError> {
        if collection.is_empty() {
            return Err(GameHubError::EmptyCollection {
                field: field_name.to_string(),
            });
        }
        Ok(())
    }

    /// Validate collection size bounds
    pub fn validate_collection_size<T>(
        collection: &[T],
        field_name: &str,
        min_size: Option<usize>,
        max_size: Option<usize>,
    ) -> Result<(), GameHubError> {
        let size = collection.len();

        if let Some(min) = min_size {
            if size < min {
                return Err(GameHubError::CollectionTooSmall {
                    field: field_name.to_string(),
                    min_size: min,
                    actual_size: size,
                });
            }
        }

        if let Some(max) = max_size {
            if size > max {
                return Err(GameHubError::CollectionTooLarge {
                    field: field_name.to_string(),
                    max_size: max,
                    actual_size: size,
                });
            }
        }

        Ok(())
    }
}