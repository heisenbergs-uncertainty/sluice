use anyhow::Result;
use clap::Parser;

mod app;
mod controller;
mod events;
mod grpc;
mod proto;
mod ui;

#[derive(Parser, Debug)]
#[command(name = "lazysluice")]
#[command(about = "Terminal UI client for Sluice", long_about = None)]
struct Args {
    /// gRPC endpoint URL, e.g. http://localhost:50051
    #[arg(
        long,
        env = "SLUICE_ENDPOINT",
        default_value = "http://localhost:50051"
    )]
    endpoint: String,

    /// Optional CA certificate PEM path for TLS.
    #[arg(long, env = "SLUICE_TLS_CA")]
    tls_ca: Option<std::path::PathBuf>,

    /// Optional TLS domain name (SNI).
    #[arg(long, env = "SLUICE_TLS_DOMAIN")]
    tls_domain: Option<String>,

    /// Subscription credits window size.
    #[arg(long, env = "SLUICE_CREDITS_WINDOW", default_value_t = 128)]
    credits_window: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .try_init();

    let _args = Args::parse();

    // TUI implementation will be wired in subsequent tasks.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_defaults_to_localhost() {
        let args = Args::try_parse_from(["lazysluice"]).expect("parse should succeed");
        assert_eq!(args.endpoint, "http://localhost:50051");
        assert_eq!(args.credits_window, 128);
        assert!(args.tls_ca.is_none());
        assert!(args.tls_domain.is_none());
    }

    #[test]
    fn endpoint_flag_overrides_default() {
        let args = Args::try_parse_from([
            "lazysluice",
            "--endpoint",
            "http://127.0.0.1:12345",
            "--credits-window",
            "7",
        ])
        .expect("parse should succeed");
        assert_eq!(args.endpoint, "http://127.0.0.1:12345");
        assert_eq!(args.credits_window, 7);
    }
}
