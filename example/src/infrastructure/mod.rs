// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Infrastructure layer for GameHub
//! 
//! This module contains infrastructure concerns including blockchain state management,
//! cross-chain messaging, and contract operations.

pub mod state;
pub mod messages;
pub mod operations;
pub mod errors;
pub mod time_utils;
pub mod handlers;
