//! sluicectl: Command-line interface for Sluice message broker.
//!
//! Provides commands for managing topics, publishing messages, and subscribing
//! to message streams from the terminal.

mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Command-line interface for Sluice message broker.
#[derive(Parser)]
#[command(name = "sluicectl")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Sluice server endpoint (e.g., http://localhost:50051)
    #[arg(short, long, env = "SLUICE_ENDPOINT", default_value = "http://localhost:50051")]
    endpoint: String,

    /// Path to TLS CA certificate (required for https:// endpoints)
    #[arg(long, env = "SLUICE_TLS_CA")]
    tls_ca: Option<String>,

    /// TLS domain name override
    #[arg(long, env = "SLUICE_TLS_DOMAIN")]
    tls_domain: Option<String>,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!("unknown output format: {}", s)),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// List topics or show topic details
    Topics {
        #[command(subcommand)]
        action: TopicsAction,
    },
    /// Publish a message to a topic
    Publish {
        /// Topic name
        topic: String,
        /// Message payload (or use --file)
        payload: Option<String>,
        /// Read payload from file
        #[arg(short, long)]
        file: Option<String>,
    },
    /// Subscribe to a topic and print messages
    Subscribe {
        /// Topic name
        topic: String,
        /// Consumer group name
        #[arg(short, long, default_value = "sluicectl")]
        group: String,
        /// Start position: latest or earliest
        #[arg(short, long, default_value = "latest")]
        position: String,
        /// Credits window size
        #[arg(long, default_value = "100")]
        credits: u32,
        /// Maximum number of messages to receive (0 = unlimited)
        #[arg(short, long, default_value = "0")]
        count: u64,
        /// Auto-acknowledge messages
        #[arg(long, default_value = "true")]
        auto_ack: bool,
    },
}

#[derive(Subcommand)]
enum TopicsAction {
    /// List all topics
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for debug output
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let cli = Cli::parse();

    // Build connection config
    let config = sluice::client::ConnectConfig {
        endpoint: cli.endpoint,
        tls_ca: cli.tls_ca,
        tls_domain: cli.tls_domain,
    };

    match cli.command {
        Commands::Topics { action } => match action {
            TopicsAction::List => commands::topics::list(config, cli.output).await?,
        },
        Commands::Publish {
            topic,
            payload,
            file,
        } => {
            commands::publish::run(config, &topic, payload, file, cli.output).await?;
        }
        Commands::Subscribe {
            topic,
            group,
            position,
            credits,
            count,
            auto_ack,
        } => {
            commands::subscribe::run(
                config,
                &topic,
                &group,
                &position,
                credits,
                count,
                auto_ack,
                cli.output,
            )
            .await?;
        }
    }

    Ok(())
}
