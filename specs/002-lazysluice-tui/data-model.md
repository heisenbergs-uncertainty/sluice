# Data Model: lazysluice TUI client (MVP)

**Feature**: 002-lazysluice-tui  
**Date**: 2026-01-05  
**Status**: Complete

## Overview

This feature introduces a standalone terminal UI client (`lazysluice`) and a single additive server RPC for topic discovery. The lazysluice client does not persist state; it maintains session-local runtime state only.

On the server side, topics are already persisted in SQLite (`topics` table). The new `ListTopics` RPC exposes that data for UI selection.

---

## Entities

### 1. Topic (Server-Persisted)

A named stream of messages. Topics already exist in the server SQLite schema.

| Field | Type | Constraints | Description |
| --- | --- | --- | --- |
| `name` | string | non-empty, unique | Topic name shown in the UI |
| `created_at` | int64 (ms) | non-negative | Unix timestamp (milliseconds) of topic creation |

**Lifecycle**:
- Created: implicitly by the server on first publish to a new topic
- Updated: never (immutable)
- Deleted: not supported (MVP)

---

### 2. MessageDelivery (Wire Model)

A message delivered over `Subscribe`.

| Field | Type | Description |
| --- | --- | --- |
| `message_id` | string | Server-generated UUIDv7 |
| `sequence` | uint64 | Global monotonic sequence number |
| `timestamp` | int64 (ms) | Server-side Unix timestamp (milliseconds) |
| `attributes` | map<string,string> | Metadata headers (including W3C tracing) |
| `payload` | bytes | Opaque payload (text or binary) |

---

### 3. SubscriptionSession (Client Runtime)

Defines a live tail session.

| Field | Type | Description |
| --- | --- | --- |
| `endpoint` | string | gRPC endpoint URL (e.g., `http://localhost:50051`) |
| `topic` | string | Selected topic |
| `consumer_group` | string | Consumer group identifier |
| `initial_position` | enum | `EARLIEST` or `LATEST` |
| `credits_window` | u32 | Credit window size used for flow control |
| `connected` | bool | Connection/stream health indicator |
| `paused` | bool | Whether rendering is paused |

**Notes**:
- Flow control is governed by credit-based backpressure (client must grant credits).
- Acknowledgements are user-initiated (manual per-message ack).

---

### 4. ClientPreferences (Session-Local)

Non-persisted UI configuration and state.

| Field | Type | Description |
| --- | --- | --- |
| `selected_topic_index` | usize | Cursor position in topic list |
| `selected_message_index` | usize | Cursor position in message list |
| `show_help` | bool | Whether help overlay/screen is active |
| `tls_enabled` | bool | Whether TLS is configured for the current session |

---

## Relationships

- `Topic` (1) → `MessageDelivery` (many): messages are published to a topic.
- `SubscriptionSession` (1) → `MessageDelivery` (many): a session receives a stream of deliveries.
- `SubscriptionSession` (1) ↔ `ClientPreferences` (1): preferences are scoped to the active session.

---

## Validation Rules (Client)

- Endpoint: must be a valid tonic endpoint URL (`http://` or `https://`).
- `credits_window`: must be > 0.
- `consumer_group`: defaults to `default` when not provided.

---

## State Transitions (Client)

### Connection State

```
[Disconnected] ── connect ──► [Connected]
     ▲                         │
     │                         └── disconnect/error ──► [Disconnected]
     └────────── retry(backoff) ───────────────────────┘
```

### Credits State (per stream)

```
[credits = 0] ── CreditGrant(N) ──► [credits = N]
[credits = N] ── deliver message ──► [credits = N-1]
```

### Ack State (UI-only)

The server cursor advances on ack; lazysluice should maintain an in-session marker for acked message IDs.
