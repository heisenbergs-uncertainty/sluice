//! Prometheus/OTLP metrics implementation.
//!
//! Key metrics:
//! - sluice_publish_total: Counter for publish operations
//! - sluice_publish_latency_seconds: Histogram for publish latency
//! - sluice_backpressure_active: Gauge for backpressure state
//! - sluice_subscription_lag: Gauge for consumer lag

use opentelemetry::metrics::{Counter, Gauge, Histogram, Meter};
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider};
use std::sync::OnceLock;

/// Global metrics instance.
static METRICS: OnceLock<Metrics> = OnceLock::new();

/// Sluice metrics registry.
#[derive(Debug)]
pub struct Metrics {
    /// Total number of publish operations.
    pub publish_total: Counter<u64>,
    /// Histogram of publish latency in seconds.
    pub publish_latency: Histogram<f64>,
    /// Gauge indicating backpressure state (1 = active, 0 = inactive).
    pub backpressure_active: Gauge<i64>,
    /// Gauge indicating subscription lag (messages behind latest).
    pub subscription_lag: Gauge<i64>,
}

impl Metrics {
    /// Create a new metrics registry from a meter.
    fn new(meter: &Meter) -> Self {
        Self {
            publish_total: meter
                .u64_counter("sluice_publish_total")
                .with_description("Total number of publish operations")
                .with_unit("1")
                .init(),
            publish_latency: meter
                .f64_histogram("sluice_publish_latency_seconds")
                .with_description("Publish latency from request to fsync")
                .with_unit("s")
                .init(),
            backpressure_active: meter
                .i64_gauge("sluice_backpressure_active")
                .with_description("1 if consumer has 0 credits and lag > 0")
                .with_unit("1")
                .init(),
            subscription_lag: meter
                .i64_gauge("sluice_subscription_lag")
                .with_description("Consumer lag (max_seq - cursor)")
                .with_unit("1")
                .init(),
        }
    }
}

/// Initialize the metrics system.
///
/// This should be called once at startup. Subsequent calls are ignored.
///
/// # Arguments
///
/// * `otel_endpoint` - Optional OTLP endpoint for metrics export
pub fn init_metrics_with_endpoint(otel_endpoint: Option<&str>) {
    METRICS.get_or_init(|| {
        if let Some(endpoint) = otel_endpoint {
            // Use OTLP exporter when endpoint is configured
            use opentelemetry_otlp::{Protocol, WithExportConfig};

            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint)
                .with_protocol(Protocol::Grpc);

            match opentelemetry_otlp::new_pipeline()
                .metrics(opentelemetry_sdk::runtime::Tokio)
                .with_exporter(exporter)
                .with_period(std::time::Duration::from_secs(10))
                .build()
            {
                Ok(provider) => {
                    global::set_meter_provider(provider);
                    tracing::info!(endpoint, "OTLP metrics exporter configured");
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to create OTLP exporter, using no-op metrics");
                    let reader = ManualReader::builder().build();
                    let provider = SdkMeterProvider::builder().with_reader(reader).build();
                    global::set_meter_provider(provider);
                }
            }
        } else {
            // No endpoint configured, use manual reader (metrics are recorded but not exported)
            let reader = ManualReader::builder().build();
            let provider = SdkMeterProvider::builder().with_reader(reader).build();
            global::set_meter_provider(provider);
        }

        let meter = global::meter("sluice");
        Metrics::new(&meter)
    });
}

/// Initialize the metrics system without OTLP export.
///
/// This should be called once at startup. Subsequent calls are ignored.
pub fn init_metrics() {
    init_metrics_with_endpoint(None);
}

/// Get the global metrics instance.
///
/// Panics if metrics have not been initialized.
pub fn metrics() -> &'static Metrics {
    METRICS
        .get()
        .expect("metrics not initialized - call init_metrics() first")
}

/// Record a successful publish operation.
pub fn record_publish(topic: &str, latency_seconds: f64) {
    if let Some(m) = METRICS.get() {
        let attrs = [KeyValue::new("topic", topic.to_string())];
        m.publish_total.add(1, &attrs);
        m.publish_latency.record(latency_seconds, &attrs);
    }
}

/// Record backpressure state for a subscription.
pub fn record_backpressure(topic: &str, consumer_group: &str, active: bool) {
    if let Some(m) = METRICS.get() {
        let attrs = [
            KeyValue::new("topic", topic.to_string()),
            KeyValue::new("consumer_group", consumer_group.to_string()),
        ];
        m.backpressure_active
            .record(if active { 1 } else { 0 }, &attrs);
    }
}

/// Record subscription lag.
pub fn record_subscription_lag(topic: &str, consumer_group: &str, lag: i64) {
    if let Some(m) = METRICS.get() {
        let attrs = [
            KeyValue::new("topic", topic.to_string()),
            KeyValue::new("consumer_group", consumer_group.to_string()),
        ];
        m.subscription_lag.record(lag, &attrs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_metrics_is_idempotent() {
        // First init should work
        init_metrics();
        // Second init should not panic
        init_metrics();
        // Metrics should be available
        let _ = metrics();
    }

    #[test]
    fn test_record_publish() {
        init_metrics();
        // Should not panic
        record_publish("test-topic", 0.001);
    }

    #[test]
    fn test_record_backpressure() {
        init_metrics();
        // Should not panic
        record_backpressure("test-topic", "test-group", true);
        record_backpressure("test-topic", "test-group", false);
    }

    #[test]
    fn test_record_subscription_lag() {
        init_metrics();
        // Should not panic
        record_subscription_lag("test-topic", "test-group", 100);
    }
}
