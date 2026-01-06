# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sluice is a gRPC-native message broker with credit-based flow control. It occupies the "middle way" between heavy cluster-native systems (Kafka) and lightweight ephemeral ones (Redis Pub/Sub), providing durable SQLite-backed message persistence with operational simplicity.

## Build and Test Commands

### Building
```bash
# Build in debug mode
cargo build

# Build release binary
cargo build --release

# Build all workspace members (sluice, sluicectl, lazysluice)
cargo build --workspace
```

### Running
```bash
# Run server with defaults (port 50051, data in ./data)
cargo run

# Run with custom configuration
cargo run -- --port 9000 --data-dir /var/lib/sluice --log-level debug

# Run sluicectl CLI
cargo run -p sluicectl -- --help

# Run lazysluice TUI
cargo run -p lazysluice
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test integration_test

# Run specific test by name
cargo test test_publish_and_subscribe

# Run with output visible
cargo test -- --nocapture

# Run tests for specific package
cargo test -p sluicectl
```

### Development
```bash
# Check code without building
cargo check

# Run clippy lints
cargo clippy

# Format code
cargo fmt

# Generate protobuf code (automatically done via build.rs)
cargo build
```

## Architecture Overview

### Core Threading Model

Sluice uses a **dedicated writer thread pattern** for SQLite operations:
- Single `std::thread` owns the write connection (src/storage/writer.rs)
- Communication via `tokio::sync::mpsc` channel for publish commands
- Enables group commit batching for 5,000+ msg/s throughput
- Reader pool uses `r2d2` for concurrent read operations (src/storage/reader.rs)

### Component Organization

```
src/
├── storage/        # SQLite persistence layer
│   ├── writer.rs   # Dedicated writer thread with group commit
│   ├── reader.rs   # r2d2 connection pool for reads
│   ├── batch.rs    # Group commit batching logic
│   └── schema.rs   # Schema initialization and queries
├── service/        # gRPC service handlers
│   ├── publish.rs  # Unary Publish RPC handler
│   ├── subscribe.rs # Bidirectional Subscribe RPC handler
│   ├── topics.rs   # ListTopics RPC handler
│   └── registry.rs # Active subscription tracking
├── flow/           # Flow control infrastructure
│   ├── credit.rs   # Credit-based backpressure tracking
│   └── notify.rs   # tokio::broadcast notification bus
├── client/         # Client library for applications
│   ├── connection.rs # gRPC connection management
│   └── subscription.rs # Subscription lifecycle handling
├── observability/  # Metrics and tracing
│   ├── metrics.rs  # OpenTelemetry metrics setup
│   └── tracing.rs  # tracing-subscriber configuration
└── server.rs       # gRPC server setup and ServerState
```

### Workspace Structure

This is a Cargo workspace with three crates:
- **sluice** (root): Core message broker server and client library
- **sluicectl** (crates/sluicectl): CLI tool for publish/subscribe/list operations
- **lazysluice** (crates/lazysluice): Terminal UI (TUI) client using ratatui

### Key Data Flow

1. **Publish Path**: gRPC request → service/publish.rs → mpsc channel → writer thread → group commit → SQLite WAL → notify bus
2. **Subscribe Path**: gRPC stream → service/subscribe.rs → reader pool → credit tracking → message delivery → ack → cursor update
3. **Notification**: Writer thread → broadcast channel → sleeping subscriptions → wake and poll

### Protocol Buffers

The gRPC API is defined in `proto/sluice/v1/sluice.proto`:
- **Publish**: Unary RPC that returns after fsync (durability guarantee)
- **Subscribe**: Bidirectional stream with credit-based flow control
  - Client sends: `SubscriptionInit`, `CreditGrant`, `Ack`
  - Server sends: `MessageDelivery`
- **ListTopics**: Unary RPC for topic discovery

Generated code is in `src/proto.rs` via build.rs.

### Storage Schema

SQLite with WAL mode, synchronous=FULL for crash safety:
- `topics` table: topic_id, name, created_at
- `messages` table: id (UUIDv7), topic_id, sequence, payload, attributes, timestamp
- `subscriptions` table: topic_id, consumer_group, cursor_seq (tracks read position)

Auto-created topics on first publish.

### Configuration

Server configuration via CLI args or environment variables (src/config.rs):
- `--host` / `SLUICE_HOST`: Bind address
- `--port` / `SLUICE_PORT`: Server port
- `--data-dir` / `SLUICE_DATA_DIR`: SQLite database location
- `--log-level` / `RUST_LOG`: Logging verbosity
- `--otel-endpoint` / `OTEL_EXPORTER_OTLP_ENDPOINT`: OpenTelemetry collector

### Client Usage

The sluice crate exports a client library (src/client/):
```rust
use sluice::client::SluiceClient;

// Connect and publish
let client = SluiceClient::connect("http://localhost:50051").await?;
client.publish("my-topic", b"payload").await?;

// Subscribe with credit-based flow control
let subscription = client.subscribe("my-topic", "my-group").await?;
subscription.grant_credits(10).await?;
while let Some(msg) = subscription.next().await {
    // Process message
    subscription.ack(&msg.message_id).await?;
}
```

### Testing Infrastructure

Integration tests in `tests/` use temporary SQLite databases and spawn servers on random ports. See `tests/common/mod.rs` for shared test utilities.

## Development Notes

### Clippy Configuration

The codebase allows several clippy lints (see src/lib.rs):
- `module_name_repetitions`: `service::publish::PublishService` is acceptable
- `needless_raw_string_hashes`: SQL queries use raw strings
- `similar_names`: `seq`, `seq_no`, `sequence` are domain terms

### UUIDv7 for Message IDs

Uses time-sortable UUIDv7 (uuid crate with v7 feature) for natural ordering and efficient range queries.

### Graceful Shutdown

Server handles SIGTERM/SIGINT by:
1. Stopping new connections
2. Flushing writer thread
3. Closing existing streams
4. Exiting cleanly

See src/main.rs signal handling.
