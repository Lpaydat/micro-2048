//! Event Management
//!
//! Centralized event handling for the Game2048 contract including:
//! - Event emission utilities
//! - Event reading from remote chains  
//! - Stream processing logic
//! - Subscription management

pub mod emitters;
pub mod processors;
pub mod readers;
pub mod subscriptions;

// Re-export for easier access
pub use processors::StreamProcessor;
pub use readers::EventReader;
pub use subscriptions::SubscriptionManager;
