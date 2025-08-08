// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Result of CSV import operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportResult {
    pub total_processed: u32,
    pub successful_imports: u32,
    pub failed_imports: u32,
    pub errors: Vec<String>,
}

impl ImportResult {
    pub fn new() -> Self {
        Self {
            total_processed: 0,
            successful_imports: 0,
            failed_imports: 0,
            errors: Vec::new(),
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_processed == 0 {
            0.0
        } else {
            (self.successful_imports as f64 / self.total_processed as f64) * 100.0
        }
    }
}