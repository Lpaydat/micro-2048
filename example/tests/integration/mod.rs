// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Integration test modules for GameHub
//! 
//! This module organizes integration tests following Linera TestValidator patterns
//! for comprehensive chain-level testing of GameHub functionality.

pub mod contract_deployment;
pub mod player_operations;
pub mod cross_chain_messaging;
pub mod timestamp_operations;
pub mod data_integration_tests;

// Additional integration test modules will be added here:
// pub mod admin_operations; 
// pub mod game_lifecycle;
// pub mod scoring_system;