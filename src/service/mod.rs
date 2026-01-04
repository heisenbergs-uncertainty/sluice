//! gRPC service handlers for Sluice.

pub mod publish;
pub mod registry;
pub mod subscribe;

pub use registry::{ConnectionRegistry, ConsumerGroupKey};

use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::Stream;
use tonic::{Request, Response, Status, Streaming};

use crate::proto::sluice::v1::sluice_server::Sluice;
use crate::proto::sluice::v1::{
    PublishRequest, PublishResponse, SubscribeDownstream, SubscribeUpstream,
};
use crate::server::ServerState;

/// Sluice gRPC service implementation.
pub struct SluiceService {
    state: Arc<ServerState>,
}

impl SluiceService {
    /// Create a new Sluice service with shared state.
    pub fn new(state: Arc<ServerState>) -> Self {
        Self { state }
    }
}

type SubscribeStream =
    Pin<Box<dyn Stream<Item = Result<SubscribeDownstream, Status>> + Send + 'static>>;

#[tonic::async_trait]
impl Sluice for SluiceService {
    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        publish::handle_publish(&self.state, request).await
    }

    type SubscribeStream = SubscribeStream;

    async fn subscribe(
        &self,
        request: Request<Streaming<SubscribeUpstream>>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        subscribe::handle_subscribe(&self.state, request).await
    }
}
