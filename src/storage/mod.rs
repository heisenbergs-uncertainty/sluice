//! SQLite storage layer for Sluice.
//!
//! Provides:
//! - Schema initialization and migrations
//! - Dedicated writer thread with group commit
//! - Read connection pool for subscriptions
//! - Batch commit logic for high throughput

pub mod batch;
pub mod reader;
pub mod schema;
pub mod writer;
