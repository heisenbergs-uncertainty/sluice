# Research: lazysluice TUI client (MVP)

**Feature**: 002-lazysluice-tui  
**Date**: 2026-01-04  
**Status**: Complete

This document resolves technical unknowns identified during planning. Decisions are intentionally scoped to MVP and biased toward minimizing impact on the existing `sluice` server crate.

---

## 1. Repo Integration Pattern (Server ↔ Client)

**Question**: What is the current integration pattern for Sluice gRPC clients within this repo?

**Decision**: Follow existing tonic/prost pattern: `.proto` is canonical, compiled via `tonic-build`, and used via generated `SluiceClient`/`SluiceServer`.

**Rationale**:
- The repo already compiles `proto/sluice/v1/sluice.proto` via `build.rs` and re-exports generated code via `src/proto.rs`.
- Tests already create an in-process tonic server and talk to it using the generated `SluiceClient`.

**Alternatives considered**:
- Hand-written client bindings: rejected (violates gRPC-native contract principle).

---

## 2. Minimal gRPC Extensions Needed for lazysluice MVP

**Question**: What additional server functionality is required to support the MVP UX (topic selection from a server-provided list) without breaking existing clients?

**Decision**: Add a single additive unary RPC: `ListTopics`.

**Rationale**:
- The MVP needs topic discovery for a selectable list.
- Sluice already persists topics in SQLite (`topics` table), so server can list them without new storage concepts.
- Adding a new RPC is backward-compatible (no changes to `Publish`/`Subscribe` semantics).

**Alternatives considered**:
- Client-only manual entry (no new RPC): rejected (explicit product decision: server-provided topic list).
- Streaming topic list / watch topics: rejected (non-MVP).

---

## 3. Optimal Repo File Structure for a Standalone TUI

**Question**: How to add lazysluice without impacting the server binary size/complexity or forcing server deps to include ratatui/crossterm?

**Decision**: Add a new Cargo workspace member crate at `crates/lazysluice`, keeping the existing root `sluice` crate intact.

**Rationale**:
- The root manifest can remain the server package; adding a workspace member avoids adding UI dependencies to the server crate’s dependency graph.
- lazysluice can compile the shared `.proto` independently from `proto/` via its own `build.rs`.

**Alternatives considered**:
- Add `src/bin/lazysluice.rs` to existing `sluice` crate: rejected (would add UI deps to server crate manifest).
- Create shared `sluice-proto` crate: viable future improvement, but not required for MVP.

---

## 4. ratatui-first Architecture (Async + UI)

**Question**: What application structure best fits ratatui and the need to integrate async gRPC streams?

**Decision**: Use a simple MVC-like split:
- Model/App state (`app`)
- View rendering (`ui`)
- Controller/event loop (`controller`)

Use async event sources (crossterm `EventStream`) plus a periodic tick and gRPC update channels.

**Rationale**:
- ratatui is immediate-mode; a main loop that (a) consumes events, (b) updates state, (c) draws is idiomatic.
- `crossterm::event::EventStream` provides a `Stream` suitable for tokio-based `select!` loops.

**Alternatives considered**:
- Blocking `crossterm::event::read()` in main loop: workable, but complicates integrating background gRPC tasks cleanly.

---

## 5. Optional TLS for tonic Client

**Question**: How to support optional TLS via flags while preserving plaintext defaults for localhost?

**Decision**: Use `tonic::transport::Endpoint` and configure TLS via `Endpoint::tls_config(ClientTlsConfig)` when flags are provided; otherwise connect over plaintext.

**Rationale**:
- tonic provides first-class client TLS configuration (`ClientTlsConfig`) and endpoint builder settings.
- This keeps the default UX simple while enabling secure remote connections.

**Alternatives considered**:
- TLS required always: rejected (conflicts with “zero config” local default).
- No TLS support: rejected (explicit product decision).
