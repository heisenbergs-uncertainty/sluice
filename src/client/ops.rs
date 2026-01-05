//! Operation result types for Sluice client.

/// Result of a publish operation.
#[derive(Debug, Clone)]
pub struct PublishResult {
    /// The server-assigned message ID.
    pub message_id: String,
}
