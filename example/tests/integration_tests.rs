// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for GameHub following Linera TestValidator patterns
//! 
//! This file serves as the main entry point for integration tests that use
//! TestValidator for comprehensive chain-level testing of GameHub functionality.

#![cfg(not(target_arch = "wasm32"))]

// Include all integration test modules
mod integration;