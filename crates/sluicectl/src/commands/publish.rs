//! Publish command implementation.

use std::fs;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use sluice_client::{ConnectConfig, SluiceClient};

use crate::OutputFormat;

#[derive(Serialize)]
struct PublishOutput {
    message_id: String,
    topic: String,
    payload_size: usize,
}

pub async fn run(
    config: ConnectConfig,
    topic: &str,
    payload: Option<String>,
    file: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    // Determine payload source
    let payload_bytes = match (payload, file) {
        (Some(p), None) => p.into_bytes(),
        (None, Some(f)) => fs::read(&f).with_context(|| format!("failed to read file: {}", f))?,
        (Some(_), Some(_)) => {
            return Err(anyhow!("cannot specify both payload and --file"));
        }
        (None, None) => {
            // Read from stdin
            use std::io::{self, Read};
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .context("failed to read from stdin")?;
            buffer
        }
    };

    let payload_size = payload_bytes.len();

    let mut client = SluiceClient::connect(config)
        .await
        .context("failed to connect to server")?;

    let result = client
        .publish(topic, payload_bytes)
        .await
        .context("publish failed")?;

    let output = PublishOutput {
        message_id: result.message_id.clone(),
        topic: topic.to_string(),
        payload_size,
    };

    match format {
        OutputFormat::Text => {
            println!("Published message to '{}'", topic);
            println!("  Message ID: {}", output.message_id);
            println!("  Payload size: {} bytes", output.payload_size);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}
