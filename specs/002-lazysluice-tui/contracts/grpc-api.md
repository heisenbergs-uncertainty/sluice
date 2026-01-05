# gRPC API Contract: lazysluice MVP additions (Sluice v1)

**Feature**: 002-lazysluice-tui  
**Date**: 2026-01-05  
**Proto File**: [proto/sluice/v1/sluice.proto](../../../proto/sluice/v1/sluice.proto)

## Overview

This feature requires server-provided topic discovery for the lazysluice TUI.

To preserve backward compatibility, the contract change is additive:
- Add `ListTopics` unary RPC
- Do not change existing `Publish`/`Subscribe` semantics

---

## Service Definition (Additive)

```protobuf
service Sluice {
  rpc Publish(PublishRequest) returns (PublishResponse);
  rpc Subscribe(stream SubscribeUpstream) returns (stream SubscribeDownstream);

  // NEW (additive): used by lazysluice to populate the topic picker.
  rpc ListTopics(ListTopicsRequest) returns (ListTopicsResponse);
}
```

---

## RPC: ListTopics

Return the set of known topics.

### Request: `ListTopicsRequest`

MVP request has no fields.

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| (none) | | | |

### Response: `ListTopicsResponse`

| Field | Type | Description |
| --- | --- | --- |
| `topics` | repeated `Topic` | Topics known to the server |

### Message: `Topic`

| Field | Type | Description |
| --- | --- | --- |
| `name` | string | Topic name |
| `created_at` | int64 | Unix timestamp (milliseconds) |

### Ordering

Servers SHOULD return topics sorted lexicographically by `name` for stable UI ordering.

### Error Codes

| gRPC Status | Condition |
| --- | --- |
| `OK` | Topics returned successfully (possibly empty list) |
| `UNAVAILABLE` | Server is shutting down or temporarily unavailable |
| `INTERNAL` | Storage error while listing topics |

---

## Backward Compatibility

- Existing clients that only call `Publish`/`Subscribe` are unaffected.
- Adding a new RPC is backward compatible in gRPC/protobuf.

---

## Notes for lazysluice

- A topic list may be empty (fresh server); UI should handle this gracefully.
- lazysluice can trigger topic creation by publishing to a new topic name (server MVP behavior), but topic selection UX for MVP is driven by `ListTopics`.
