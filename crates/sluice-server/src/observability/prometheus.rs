//! Prometheus HTTP endpoint for metrics scraping.
//!
//! Provides:
//! - `/metrics` - Prometheus metrics endpoint
//! - `/health` - Basic health check
//! - `/ready` - Readiness check

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use prometheus::{Encoder, Registry, TextEncoder};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::watch;

/// Prometheus server state.
#[derive(Clone)]
pub struct PrometheusState {
    registry: Arc<Registry>,
}

impl PrometheusState {
    /// Create a new Prometheus state with the given registry.
    pub fn new(registry: Registry) -> Self {
        Self {
            registry: Arc::new(registry),
        }
    }
}

/// Create the Prometheus HTTP router.
pub fn create_router(state: PrometheusState) -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .with_state(state)
}

/// Handle GET /metrics - Prometheus metrics endpoint.
async fn metrics_handler(State(state): State<PrometheusState>) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = state.registry.gather();

    let mut buffer = Vec::new();
    match encoder.encode(&metric_families, &mut buffer) {
        Ok(()) => (
            StatusCode::OK,
            [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
            buffer,
        ),
        Err(e) => {
            tracing::error!(error = %e, "Failed to encode metrics");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/plain; charset=utf-8")],
                format!("Failed to encode metrics: {e}").into_bytes(),
            )
        }
    }
}

/// Handle GET /health - Basic health check.
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Handle GET /ready - Readiness check.
async fn ready_handler() -> impl IntoResponse {
    (StatusCode::OK, "READY")
}

/// Run the Prometheus HTTP server.
///
/// # Arguments
///
/// * `addr` - Address to bind to
/// * `registry` - Prometheus registry to serve
/// * `shutdown_rx` - Receiver for shutdown signal
pub async fn run_prometheus_server(
    addr: SocketAddr,
    registry: Registry,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = PrometheusState::new(registry);
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(address = %addr, "Starting Prometheus metrics server");

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.changed().await;
            tracing::info!("Prometheus server shutting down");
        })
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let registry = Registry::new();
        let state = PrometheusState::new(registry);
        let app = create_router(state);

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ready_endpoint() {
        let registry = Registry::new();
        let state = PrometheusState::new(registry);
        let app = create_router(state);

        let response = app
            .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let registry = Registry::new();
        let state = PrometheusState::new(registry);
        let app = create_router(state);

        let response = app
            .oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
