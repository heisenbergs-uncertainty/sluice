//! Topic discovery service (ListTopics).

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::sluice::v1::{ListTopicsRequest, ListTopicsResponse, Topic};
use crate::server::ServerState;

pub async fn handle_list_topics(
    state: &Arc<ServerState>,
    _request: Request<ListTopicsRequest>,
) -> Result<Response<ListTopicsResponse>, Status> {
    let topics = state
        .reader_pool
        .list_topics()
        .map_err(|e| Status::internal(format!("failed to list topics: {e}")))?
        .into_iter()
        .map(|(name, created_at)| Topic { name, created_at })
        .collect::<Vec<_>>();

    Ok(Response::new(ListTopicsResponse { topics }))
}
