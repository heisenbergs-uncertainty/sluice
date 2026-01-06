//! Subscribe command implementation.

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use sluice_client::{ConnectConfig, InitialPosition, SluiceClient};
use tokio::signal;

use crate::OutputFormat;

#[derive(Serialize)]
struct MessageOutput {
    message_id: String,
    topic: String,
    sequence: u64,
    payload: String,
    payload_bytes: usize,
}

pub async fn run(
    config: ConnectConfig,
    topic: &str,
    group: &str,
    position: &str,
    credits: u32,
    count: u64,
    auto_ack: bool,
    format: OutputFormat,
) -> Result<()> {
    let initial_position = match position.to_lowercase().as_str() {
        "latest" => InitialPosition::Latest,
        "earliest" => InitialPosition::Earliest,
        _ => return Err(anyhow!("position must be 'latest' or 'earliest'")),
    };

    let mut client = SluiceClient::connect(config)
        .await
        .context("failed to connect to server")?;

    let mut subscription = client
        .subscribe(topic, Some(group), None, initial_position, credits)
        .await
        .context("failed to subscribe")?;

    if format == OutputFormat::Text {
        eprintln!(
            "Subscribed to '{}' (group: {}, position: {})",
            topic, group, position
        );
        eprintln!("Press Ctrl+C to stop...\n");
    }

    let mut received: u64 = 0;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                if format == OutputFormat::Text {
                    eprintln!("\nReceived {} message(s)", received);
                }
                break;
            }
            result = subscription.next_message() => {
                match result {
                    Ok(Some(msg)) => {
                        received += 1;

                        let payload_str = String::from_utf8_lossy(&msg.payload).to_string();
                        let output = MessageOutput {
                            message_id: msg.message_id.clone(),
                            topic: topic.to_string(),
                            sequence: msg.sequence,
                            payload: payload_str.clone(),
                            payload_bytes: msg.payload.len(),
                        };

                        match format {
                            OutputFormat::Text => {
                                println!(
                                    "[{}] seq={} id={}: {}",
                                    topic,
                                    msg.sequence,
                                    &msg.message_id[..8.min(msg.message_id.len())],
                                    payload_str
                                );
                            }
                            OutputFormat::Json => {
                                println!("{}", serde_json::to_string(&output)?);
                            }
                        }

                        if auto_ack {
                            subscription.send_ack(&msg.message_id).await?;
                        }

                        // Refill credits if needed
                        subscription.maybe_refill_credits().await?;

                        // Check count limit
                        if count > 0 && received >= count {
                            if format == OutputFormat::Text {
                                eprintln!("\nReached message limit ({})", count);
                            }
                            break;
                        }
                    }
                    Ok(None) => {
                        if format == OutputFormat::Text {
                            eprintln!("Stream ended");
                        }
                        break;
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
