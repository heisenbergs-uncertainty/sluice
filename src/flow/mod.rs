//! Flow control and notification infrastructure.
//!
//! Provides:
//! - Credit-based flow control for subscriptions
//! - Notification bus for waking sleeping subscriptions

pub mod credit;
pub mod notify;
