//! Subscription handling for Sluice client.

use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use tonic::Streaming;

use crate::proto::sluice::v1::sluice_client::SluiceClient as ProtoClient;
use crate::proto::sluice::v1::{
    subscribe_downstream, subscribe_upstream, Ack, CreditGrant, InitialPosition, MessageDelivery,
    SubscribeDownstream, SubscribeUpstream, SubscriptionInit,
};

/// A handle for controlling an active subscription.
///
/// Manages credit-based flow control and provides methods for receiving
/// messages and sending acknowledgments.
pub struct Subscription {
    /// Sender for upstream messages (credits, acks).
    tx: mpsc::Sender<SubscribeUpstream>,
    /// Receiver for downstream message deliveries.
    rx: Streaming<SubscribeDownstream>,
    /// Configured credits window for refill policy.
    credits_window: u32,
    /// Remaining credits before refill is needed.
    remaining_credits: u32,
}

impl Subscription {
    /// Start a new subscription.
    pub(crate) async fn start(
        client: &mut ProtoClient<Channel>,
        topic: String,
        consumer_group: Option<String>,
        consumer_id: Option<String>,
        initial_position: InitialPosition,
        credits_window: u32,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<SubscribeUpstream>(32);

        // Send init message
        let init = SubscribeUpstream {
            request: Some(subscribe_upstream::Request::Init(SubscriptionInit {
                topic,
                consumer_group: consumer_group.unwrap_or_else(|| "default".to_string()),
                consumer_id: consumer_id.unwrap_or_default(),
                initial_position: initial_position.into(),
            })),
        };
        tx.send(init)
            .await
            .map_err(|_| anyhow!("failed to send subscription init"))?;

        // Send initial credit grant
        let credit = SubscribeUpstream {
            request: Some(subscribe_upstream::Request::Credit(CreditGrant {
                credits: credits_window,
            })),
        };
        tx.send(credit)
            .await
            .map_err(|_| anyhow!("failed to send initial credits"))?;

        // Start the bidirectional stream
        let stream = ReceiverStream::new(rx);
        let response = client
            .subscribe(stream)
            .await
            .context("subscribe RPC failed")?;

        Ok(Self {
            tx,
            rx: response.into_inner(),
            credits_window,
            remaining_credits: credits_window,
        })
    }

    /// Get the next message delivery, returning None if stream ends.
    pub async fn next_message(&mut self) -> Result<Option<MessageDelivery>> {
        match self.rx.next().await {
            Some(Ok(downstream)) => {
                if let Some(subscribe_downstream::Response::Delivery(msg)) = downstream.response {
                    self.consume_credit();
                    Ok(Some(msg))
                } else {
                    Ok(None)
                }
            }
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Check if credits should be refilled and send grant if needed.
    /// Returns true if a grant was sent.
    pub async fn maybe_refill_credits(&mut self) -> Result<bool> {
        let threshold = self.credits_window / 2;
        if self.remaining_credits < threshold {
            let grant = self.credits_window;
            self.send_credit(grant).await?;
            self.remaining_credits += grant;
            return Ok(true);
        }
        Ok(false)
    }

    /// Decrement remaining credits (called when a message is received).
    fn consume_credit(&mut self) {
        self.remaining_credits = self.remaining_credits.saturating_sub(1);
    }

    /// Send a CreditGrant message.
    pub async fn send_credit(&self, credits: u32) -> Result<()> {
        self.tx
            .send(SubscribeUpstream {
                request: Some(subscribe_upstream::Request::Credit(CreditGrant { credits })),
            })
            .await
            .map_err(|_| anyhow!("subscription channel closed"))
    }

    /// Send an Ack message for a specific message ID.
    pub async fn send_ack(&self, message_id: &str) -> Result<()> {
        self.tx
            .send(SubscribeUpstream {
                request: Some(subscribe_upstream::Request::Ack(Ack {
                    message_id: message_id.to_string(),
                })),
            })
            .await
            .map_err(|_| anyhow!("subscription channel closed"))
    }

    /// Get the configured credits window size.
    pub fn credits_window(&self) -> u32 {
        self.credits_window
    }

    /// Get the current remaining credits.
    pub fn remaining_credits(&self) -> u32 {
        self.remaining_credits
    }
}
