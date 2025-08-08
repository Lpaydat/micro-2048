// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Time utilities for GameHub
//! 
//! This module provides time constants and calculations for blockchain-compatible
//! timestamp handling. Actual timestamps should come from runtime.system_time().

use linera_sdk::linera_base_types::Timestamp;

/// Create a timestamp from microseconds (blockchain-compatible utility)
pub fn from_micros(micros: u64) -> Timestamp {
    Timestamp::from(micros)
}

/// Mock time provider for testing environments only
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockTimeProvider {
    fixed_time: u64,
}

#[cfg(test)]
impl MockTimeProvider {
    /// Create a new mock time provider with a fixed timestamp
    pub fn new(fixed_micros: u64) -> Self {
        Self {
            fixed_time: fixed_micros,
        }
    }
    
    /// Update the mock time
    pub fn set_time(&mut self, micros: u64) {
        self.fixed_time = micros;
    }
    
    /// Advance time by the given microseconds
    pub fn advance_by(&mut self, micros: u64) {
        self.fixed_time += micros;
    }
    
    /// Get the current mock timestamp
    pub fn now(&self) -> Timestamp {
        Timestamp::from(self.fixed_time)
    }
}


/// Time-related constants
pub mod constants {
    /// Microseconds in a second
    pub const MICROS_PER_SECOND: u64 = 1_000_000;
    
    /// Microseconds in a minute
    pub const MICROS_PER_MINUTE: u64 = 60 * MICROS_PER_SECOND;
    
    /// Microseconds in an hour
    pub const MICROS_PER_HOUR: u64 = 60 * MICROS_PER_MINUTE;
    
    /// Microseconds in a day
    pub const MICROS_PER_DAY: u64 = 24 * MICROS_PER_HOUR;
    
    /// Microseconds in a week (used for streak calculations)
    pub const MICROS_PER_WEEK: u64 = 7 * MICROS_PER_DAY;
}

/// Time calculation utilities
pub mod calculations {
    use super::constants::*;
    use linera_sdk::linera_base_types::Timestamp;
    
    /// Check if two timestamps are within the given duration
    pub fn within_duration(ts1: Timestamp, ts2: Timestamp, max_diff_micros: u64) -> bool {
        let diff = if ts1.micros() > ts2.micros() {
            ts1.micros() - ts2.micros()
        } else {
            ts2.micros() - ts1.micros()
        };
        diff <= max_diff_micros
    }
    
    /// Check if two timestamps are within a week (for streak calculations)
    pub fn within_week(ts1: Timestamp, ts2: Timestamp) -> bool {
        within_duration(ts1, ts2, MICROS_PER_WEEK)
    }
    
    /// Get the difference between two timestamps in hours
    pub fn hours_between(earlier: Timestamp, later: Timestamp) -> u64 {
        if later.micros() <= earlier.micros() {
            return 0;
        }
        (later.micros() - earlier.micros()) / MICROS_PER_HOUR
    }
    
    /// Add hours to a timestamp
    pub fn add_hours(timestamp: Timestamp, hours: u64) -> Timestamp {
        Timestamp::from(timestamp.micros() + (hours * MICROS_PER_HOUR))
    }
    
    /// Check if a timestamp is in the past compared to current
    pub fn is_past(timestamp: Timestamp, current: Timestamp) -> bool {
        timestamp.micros() < current.micros()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::calculations::*;
    use super::constants::*;
    
    #[test]
    fn test_from_micros_utility() {
        let timestamp = from_micros(1000000);
        assert_eq!(timestamp.micros(), 1000000);
        
        let large_timestamp = from_micros(u64::MAX);
        assert_eq!(large_timestamp.micros(), u64::MAX);
    }
    
    #[test]
    fn test_mock_time_provider() {
        let mut provider = MockTimeProvider::new(1000000);
        
        assert_eq!(provider.now().micros(), 1000000);
        
        provider.advance_by(500000);
        assert_eq!(provider.now().micros(), 1500000);
        
        provider.set_time(2000000);
        assert_eq!(provider.now().micros(), 2000000);
    }
    
    #[test]
    fn test_time_calculations() {
        let ts1 = Timestamp::from(1000000);
        let ts2 = Timestamp::from(2000000);
        
        assert!(within_duration(ts1, ts2, 1500000));
        assert!(!within_duration(ts1, ts2, 500000));
        
        assert_eq!(hours_between(ts1, ts2), 0); // Less than an hour
        
        let ts3 = Timestamp::from(1000000 + MICROS_PER_HOUR * 5);
        assert_eq!(hours_between(ts1, ts3), 5);
        
        let future = add_hours(ts1, 3);
        assert_eq!(future.micros(), 1000000 + MICROS_PER_HOUR * 3);
        
        assert!(is_past(ts1, ts2));
        assert!(!is_past(ts2, ts1));
    }
    
    #[test]
    fn test_streak_timing() {
        let base_time = Timestamp::from(1000000);
        let one_day_later = Timestamp::from(1000000 + MICROS_PER_DAY);
        let two_weeks_later = Timestamp::from(1000000 + 2 * MICROS_PER_WEEK);
        
        assert!(within_week(base_time, one_day_later));
        assert!(!within_week(base_time, two_weeks_later));
    }
}