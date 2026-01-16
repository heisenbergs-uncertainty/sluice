//! OpenTelemetry observability infrastructure.
//!
//! Provides:
//! - Structured tracing with OpenTelemetry export
//! - Prometheus/OTLP metrics for key operations
//! - HTTP endpoints for Prometheus scraping

pub mod metrics;
pub mod prometheus;
pub mod tracing;
