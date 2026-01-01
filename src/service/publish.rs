//! Publish RPC handler implementation.
//!
//! Handles unary Publish requests with durable persistence.

use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::generate_message_id;
use crate::proto::sluice::v1::{PublishRequest, PublishResponse};
use crate::server::ServerState;
use crate::storage::writer::WriterError;

/// Maximum payload size (4MB, gRPC default limit).
const MAX_PAYLOAD_SIZE: usize = 4 * 1024 * 1024;

/// Handle a Publish RPC request.
///
/// Persists the message durably with fsync before returning.
#[tracing::instrument(skip(state, request), fields(topic))]
pub async fn handle_publish(
    state: &Arc<ServerState>,
    request: Request<PublishRequest>,
) -> Result<Response<PublishResponse>, Status> {
    let req = request.into_inner();

    // Validate topic
    if req.topic.is_empty() {
        return Err(Status::invalid_argument("topic cannot be empty"));
    }

    if req.topic.len() > 255 {
        return Err(Status::invalid_argument("topic name too long (max 255 characters)"));
    }

    // Validate payload size
    if req.payload.len() > MAX_PAYLOAD_SIZE {
        return Err(Status::resource_exhausted(format!(
            "payload too large: {} bytes (max {} bytes)",
            req.payload.len(),
            MAX_PAYLOAD_SIZE
        )));
    }

    tracing::Span::current().record("topic", &req.topic);

    // Serialize attributes to JSON
    let attributes = if req.attributes.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&req.attributes).map_err(|e| {
            Status::invalid_argument(format!("invalid attributes: {e}"))
        })?)
    };

    // Generate message ID
    let message_id = generate_message_id();

    // Submit to writer
    let result = state
        .writer
        .publish(
            req.topic,
            message_id.clone(),
            if req.payload.is_empty() {
                None
            } else {
                Some(req.payload)
            },
            attributes,
        )
        .await
        .map_err(|e| match e {
            WriterError::ChannelClosed => {
                Status::unavailable("server is shutting down")
            }
            WriterError::Database(msg) if msg.contains("disk") || msg.contains("full") => {
                Status::unavailable(format!("storage error: {msg}"))
            }
            WriterError::Database(msg) => Status::internal(format!("database error: {msg}")),
            WriterError::ThreadPanic => Status::internal("internal error"),
        })?;

    tracing::debug!(
        message_id = %result.message_id,
        sequence = result.sequence,
        "Message published"
    );

    Ok(Response::new(PublishResponse {
        message_id: result.message_id,
        sequence: result.sequence as u64,
        timestamp: result.timestamp,
    }))
}

