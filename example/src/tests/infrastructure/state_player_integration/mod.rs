// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! State player integration test modules
//! 
//! This module organizes state player integration tests into focused areas:
//! - Contract initialization validation workflows
//! - Data structure integration and relationships  
//! - Comprehensive validation workflows
//! - Pending data handling and event score management
//! - Complex workflow integration tests combining multiple operations

pub mod test_helpers;
pub mod contract_initialization_integration_tests;
pub mod data_structure_integration_tests;
pub mod validation_integration_tests;
pub mod pending_data_integration_tests;
pub mod workflow_integration_tests;