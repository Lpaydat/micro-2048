// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Core domain logic for GameHub
//! 
//! This module contains the pure business logic of the GameHub application,
//! including domain types, services, and validation logic.

pub mod types;
pub mod domain;
pub mod validation;

// Re-export commonly used types
pub use types::*;