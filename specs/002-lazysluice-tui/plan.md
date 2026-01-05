# Implementation Plan: lazysluice TUI client

**Branch**: `002-lazysluice-tui` | **Date**: 2026-01-05 | **Spec**: [specs/002-lazysluice-tui/spec.md](spec.md)
**Input**: Feature specification from [specs/002-lazysluice-tui/spec.md](spec.md)

## Summary

Implement `lazysluice`: a standalone Rust TUI gRPC client for Sluice, focused on (1) connect + select a topic from a server-provided list, (2) live tail with credit-based flow control and manual per-message ack, and (3) publish test messages. Add a single backward-compatible server RPC for topic discovery (`ListTopics`) and keep all existing `Publish`/`Subscribe` semantics unchanged.

## Technical Context

**Language/Version**: Rust (edition 2021; latest stable)  
**Primary Dependencies**:
- Client: `ratatui`, `crossterm`, `tokio`, `tonic`, `prost`, `clap`, `anyhow`, `tracing`
- Server (existing): `tonic`, `prost`, `tokio`, `rusqlite`, `tracing` (no UI deps added)
**Storage**: Client: N/A (in-memory only). Server: SQLite (existing)  
**Testing**: `cargo test` (contract + integration tests; add contract tests for `ListTopics`)  
**Target Platform**: Terminal UI on macOS/Linux; server remains single-node binary  
**Project Type**: Cargo workspace with two binaries (server root crate + `crates/lazysluice`)  
**Performance Goals**:
- Client cold start: first UI within 1s (SC-001)
- Message display latency: <250ms under normal local conditions (SC-003)
**Constraints**:
- gRPC is the only protocol; `.proto` is canonical
- Must preserve credit-based flow control and manual ack semantics
- Additive-only API changes; no breaking changes to `Publish`/`Subscribe`
- Keep server dependency graph free of TUI deps
**Scale/Scope**: MVP TUI with a small number of screens (topic list, tail, publish, help)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Pass/Fail | Notes |
|---|---|---|
| I. gRPC-Native Protocol | PASS | New functionality (`ListTopics`) is added to `proto/sluice/v1/sluice.proto` and codegen remains prost/tonic-based. |
| II. Application-Level Backpressure | PASS | Client uses `CreditGrant` for streaming and does not assume push without credit. |
| III. Test-Driven Development | PASS | Add contract tests for new RPC and keep existing test style. |
| IV. Operational Simplicity | PASS | Single additive RPC; no extra services; TLS optional (plaintext default localhost). |
| V. Observability via OpenTelemetry | FAIL | Tasks currently only add tracing logs/spans; add OTEL tracer + OTLP exporter + minimal client metrics to satisfy constitution. |

## Project Structure

### Documentation (this feature)

```text
specs/002-lazysluice-tui/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── grpc-api.md
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
.
├── Cargo.toml                 # Root server package + workspace members
├── build.rs                   # Server proto codegen (existing)
├── proto/sluice/v1/sluice.proto
├── src/                       # Server crate (existing)
│   ├── service/
│   │   ├── mod.rs             # Add ListTopics handler wiring
│   │   └── topics.rs          # New: topic listing implementation
│   └── storage/
│       ├── mod.rs
│       └── reader.rs          # Add list-topics query (or schema helper)
├── crates/
│   └── lazysluice/
│       ├── Cargo.toml
│       ├── build.rs           # Client proto codegen (compile from repo proto/)
│       └── src/
│           ├── main.rs
│           ├── app.rs
│           ├── ui.rs
│           ├── controller.rs
│           ├── events.rs
│           └── grpc/
│               ├── mod.rs
│               └── client.rs
└── tests/                     # Existing test suite; add/extend contract tests
```

**Structure Decision**: Cargo workspace + separate `lazysluice` crate to keep TUI dependencies isolated from the server binary.

## Phase 0: Outline & Research

**Outputs**: [specs/002-lazysluice-tui/research.md](research.md)

Research decisions (resolved):
- Additive gRPC extension required for topic discovery (`ListTopics`).
- Repo structure: add `crates/lazysluice` workspace member.
- ratatui architecture: model/view/controller split with async event loop.
- tonic TLS: optional `ClientTlsConfig` on `Endpoint`.

## Phase 1: Design & Contracts

**Outputs**:
- [specs/002-lazysluice-tui/data-model.md](data-model.md)
- [specs/002-lazysluice-tui/contracts/grpc-api.md](contracts/grpc-api.md)
- [specs/002-lazysluice-tui/quickstart.md](quickstart.md)

### API Design (Additive)

1. Extend `proto/sluice/v1/sluice.proto` with:
  - `rpc ListTopics(ListTopicsRequest) returns (ListTopicsResponse)`
  - `Topic` message (name + created_at timestamp)
2. Server implementation:
  - List topics from existing SQLite `topics` table.
  - Return sorted list (lexicographic by name) for stable UI ordering.
3. Contract tests:
  - Start test server, publish to new topics, assert `ListTopics` returns those topics.

### Client UX Design (MVP)

Screens (minimal):
1. Connection + topic list screen
2. Tail screen (message list, paused indicator, selected message)
3. Publish screen (topic + payload editor)
4. Help screen (keybindings)

Key behaviors:
- Auto-reconnect with backoff on disconnect; UI remains responsive.
- Manual ack of selected message sends `Ack(message_id)`.
- Credits window configurable; client sends `CreditGrant(credits_window)` initially and then replenishes.
- TLS optional; plaintext default for localhost.

## Phase 2: Implementation Planning (for /speckit.tasks)

This section enumerates the concrete work items that `/speckit.tasks` should expand into TDD tasks.

1. **Proto + codegen**
  - Add `ListTopics` RPC/messages to `proto/sluice/v1/sluice.proto`.
  - Regenerate server proto code.
2. **Server: ListTopics**
  - Add storage query to list topic names + timestamps.
  - Implement `ListTopics` handler in service layer.
  - Add contract tests.
3. **Workspace + lazysluice crate**
  - Convert repo to workspace while keeping existing server package behavior.
  - Add `crates/lazysluice` crate with its own proto codegen.
4. **Client: gRPC plumbing**
  - Build optional TLS endpoint creation.
  - Implement list-topics call.
  - Implement subscribe loop with credit grants and ack.
5. **Client: TUI app loop**
  - ratatui renderer and app state.
  - Event handling (keys + ticks + gRPC updates).
  - Reconnect/backoff behavior.

## Complexity Tracking

No constitution violations are required for this feature.
