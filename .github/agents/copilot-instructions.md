# sluice Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-01-01

## Active Technologies
- Rust (edition 2021; latest stable) (002-lazysluice-tui)
- Client: N/A (in-memory only). Server: SQLite (existing) (002-lazysluice-tui)

- Rust (latest stable, 1.75+) + tonic (gRPC), prost (protobuf), tokio (async runtime), rusqlite (SQLite), tracing + opentelemetry (001-mvp-core)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust (latest stable, 1.75+): Follow standard conventions

## Recent Changes
- 002-lazysluice-tui: Added Rust (edition 2021; latest stable)

- 001-mvp-core: Added Rust (latest stable, 1.75+) + tonic (gRPC), prost (protobuf), tokio (async runtime), rusqlite (SQLite), tracing + opentelemetry

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
