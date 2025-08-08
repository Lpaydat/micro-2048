// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Timestamp utilities for tests

use linera_sdk::linera_base_types::Timestamp;

/// Create a standard test timestamp
pub fn test_timestamp() -> Timestamp {
    Timestamp::from(1000000)
}

/// Create a test timestamp with custom micros
pub fn test_timestamp_from_micros(micros: u64) -> Timestamp {
    Timestamp::from(micros)
}

/// Create an earlier test timestamp  
pub fn earlier_test_timestamp() -> Timestamp {
    Timestamp::from(500000)
}

/// Create a later test timestamp
pub fn later_test_timestamp() -> Timestamp {
    Timestamp::from(1500000)
}