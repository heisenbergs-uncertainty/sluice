//! Configuration parsing for Sluice server.
//!
//! Supports:
//! - CLI arguments via clap
//! - Environment variable overrides
//! - Sensible defaults for quick start

use clap::Parser;
use std::path::PathBuf;

/// Sluice: A gRPC-native message broker with credit-based flow control.
#[derive(Parser, Debug, Clone)]
#[command(name = "sluice")]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Host address to bind to
    #[arg(long, env = "SLUICE_HOST", default_value = "0.0.0.0")]
    pub host: String,

    /// Port to listen on
    #[arg(short, long, env = "SLUICE_PORT", default_value_t = 50051)]
    pub port: u16,

    /// Data directory for SQLite database
    #[arg(short, long, env = "SLUICE_DATA_DIR", default_value = "./data")]
    pub data_dir: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, env = "RUST_LOG", default_value = "info")]
    pub log_level: String,

    /// Size of the write channel (backpressure control)
    #[arg(long, env = "SLUICE_WRITE_CHANNEL_SIZE", default_value_t = 1000)]
    pub write_channel_size: usize,

    /// Size of the reader connection pool
    #[arg(long, env = "SLUICE_READER_POOL_SIZE", default_value_t = 10)]
    pub reader_pool_size: u32,

    /// Size of the notification channel
    #[arg(long, env = "SLUICE_NOTIFY_CHANNEL_SIZE", default_value_t = 1024)]
    pub notify_channel_size: usize,

    /// OpenTelemetry collector endpoint for metrics export (optional)
    #[arg(long, env = "OTEL_EXPORTER_OTLP_ENDPOINT")]
    pub otel_endpoint: Option<String>,
}

impl Config {
    /// Parse configuration from CLI arguments and environment.
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Create a default configuration for testing.
    #[cfg(test)]
    pub fn test_config(data_dir: PathBuf) -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 0, // Random port
            data_dir,
            log_level: "debug".into(),
            write_channel_size: 100,
            reader_pool_size: 5,
            notify_channel_size: 256,
            otel_endpoint: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 50051,
            data_dir: PathBuf::from("./data"),
            log_level: "info".into(),
            write_channel_size: 1000,
            reader_pool_size: 10,
            notify_channel_size: 1024,
            otel_endpoint: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.port, 50051);
        assert_eq!(config.host, "0.0.0.0");
    }
}
