//! Shared gRPC client library for Sluice.
//!
//! This module provides a reusable client for connecting to Sluice servers,
//! used by both the `lazysluice` TUI and the `sluicectl` CLI.

mod connection;
mod ops;
mod subscription;

pub use connection::{ConnectConfig, SluiceClient};
pub use ops::PublishResult;
pub use subscription::Subscription;

// Re-export proto types that clients need
pub use crate::proto::sluice::v1::{InitialPosition, MessageDelivery, PublishResponse, Topic};
