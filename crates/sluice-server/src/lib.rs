//! Sluice: A gRPC-native message broker with credit-based flow control.
//!
//! Sluice provides At-Least-Once delivery semantics with SQLite WAL persistence,
//! achieving 5,000+ msg/s throughput via group commit batching.
//!
//! # Architecture
//!
//! - **gRPC-Native**: All communication via tonic/prost generated code
//! - **Credit-Based Flow Control**: Consumers control delivery rate
//! - **Durable**: Messages survive crashes with `synchronous=FULL`
//! - **Observable**: OpenTelemetry metrics and tracing
//!
//! # Modules
//!
//! - [`config`]: CLI and environment configuration
//! - [`flow`]: Credit tracking and notification bus
//! - [`observability`]: Metrics and tracing setup
//! - [`proto`]: Re-exported protobuf code
//! - [`server`]: gRPC server setup
//! - [`service`]: RPC handlers (Publish, Subscribe)
//! - [`storage`]: SQLite persistence layer

// Lint configuration
#![warn(clippy::all)]
#![allow(
    clippy::module_name_repetitions,    // service::publish::PublishService is fine
    clippy::must_use_candidate,         // Not all functions need #[must_use]
    clippy::missing_errors_doc,         // Error docs can be verbose
    clippy::missing_panics_doc,         // Panic docs can be verbose
    clippy::needless_raw_string_hashes, // r#""# is fine for SQL
    clippy::similar_names,              // seq/seq_no/sequence are fine
    clippy::struct_excessive_bools,     // Config structs may have flags
    clippy::too_many_lines              // Some functions are inherently long
)]

pub mod config;
pub mod flow;
pub mod observability;
pub mod proto;
pub mod server;
pub mod service;
pub mod storage;

use uuid::Uuid;

/// Generate a new UUIDv7 (time-sortable) message ID.
///
/// UUIDv7 provides time-sortable IDs that enable efficient range queries
/// and natural ordering by creation time.
///
/// # Example
///
/// ```
/// let id = sluice_server::generate_message_id();
/// assert!(id.len() == 36); // UUID string format
/// ```
#[must_use]
pub fn generate_message_id() -> String {
    Uuid::now_v7().to_string()
}

/// Get the current Unix timestamp in milliseconds.
#[must_use]
pub fn now_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before Unix epoch")
        .as_millis() as i64
}
