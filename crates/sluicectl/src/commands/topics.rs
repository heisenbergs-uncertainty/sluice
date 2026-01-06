//! Topics command implementation.

use anyhow::{Context, Result};
use serde::Serialize;
use sluice_client::{ConnectConfig, SluiceClient};

use crate::OutputFormat;

#[derive(Serialize)]
struct TopicInfo {
    name: String,
    created_at: i64,
}

#[derive(Serialize)]
struct TopicsOutput {
    topics: Vec<TopicInfo>,
    total: usize,
}

pub async fn list(config: ConnectConfig, format: OutputFormat) -> Result<()> {
    let mut client = SluiceClient::connect(config)
        .await
        .context("failed to connect to server")?;

    let topics = client.list_topics().await?;

    let output = TopicsOutput {
        total: topics.len(),
        topics: topics
            .into_iter()
            .map(|t| TopicInfo {
                name: t.name,
                created_at: t.created_at,
            })
            .collect(),
    };

    match format {
        OutputFormat::Text => {
            if output.topics.is_empty() {
                println!("No topics found.");
            } else {
                println!("{:<40} {:>20}", "TOPIC", "CREATED AT");
                println!("{}", "-".repeat(62));
                for topic in &output.topics {
                    // Format timestamp as human readable if possible
                    let ts = chrono_format(topic.created_at);
                    println!("{:<40} {:>20}", topic.name, ts);
                }
                println!();
                println!("Total: {} topic(s)", output.total);
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}

fn chrono_format(millis: i64) -> String {
    // Simple formatting: just show the unix timestamp if we don't have chrono
    if millis == 0 {
        return "-".to_string();
    }
    // Format as seconds since epoch
    format!("{}", millis / 1000)
}
