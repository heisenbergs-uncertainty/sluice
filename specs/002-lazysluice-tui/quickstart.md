# Quickstart: lazysluice TUI client (MVP)

**Feature**: 002-lazysluice-tui  
**Date**: 2026-01-05

This quickstart describes how the lazysluice TUI is expected to be built and run alongside the Sluice server once the implementation tasks are completed.

## Prerequisites

- Rust (latest stable)
- `protoc` (Protocol Buffer compiler)
- macOS or Linux terminal

## Build

```bash
# From repository root
cargo build --release
```

Expected binaries:
- `./target/release/sluice` (server)
- `./target/release/lazysluice` (TUI client)

## Run Server

```bash
# Default server (port 50051)
./target/release/sluice serve
```

## Run lazysluice

```bash
# Connect to localhost with plaintext defaults
./target/release/lazysluice

# Or specify an explicit endpoint
./target/release/lazysluice --endpoint http://localhost:50051
```

## TLS (Optional)

When connecting to a remote endpoint, lazysluice supports optional TLS.

```bash
# Minimal flag shape (finalized for MVP tasks)
./target/release/lazysluice \
  --endpoint https://sluice.example.com:443 \
  --tls-ca /path/to/ca.pem \
  --tls-domain sluice.example.com
```

## Basic Usage Flow

1. Start lazysluice.
2. It connects and loads the topic list via `ListTopics`.
3. Select a topic and start tailing.
4. Use the configured credits window for flow control.
5. Select a message and press the ack key to send `Ack(message_id)`.
6. Switch to publish view to send a test message.

## Development

```bash
# Run tests
cargo test

# Format and lint
cargo fmt --check
cargo clippy -- -D warnings
```
