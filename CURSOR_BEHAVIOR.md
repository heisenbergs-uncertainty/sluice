# Sluice Consumer Group Cursor Behavior

## How Cursors Work in Sluice

Sluice uses **consumer group cursors** to track message processing progress. Understanding this behavior is critical for using the TUI effectively.

### Basic Concepts

1. **Consumer Group**: A logical grouping of consumers that share message processing
2. **Cursor**: A per-consumer-group pointer that tracks which messages have been acknowledged
3. **Sequence Number**: A monotonically increasing counter for messages in a topic (1, 2, 3, ...)

### Cursor Lifecycle

```
Topic: orders
Messages: [msg-1 (seq=1), msg-2 (seq=2), msg-3 (seq=3)]
Consumer Group: "default"
```

#### Initial State
```
Cursor position: 0 (before any messages)
Available messages: 1, 2, 3
```

#### After Subscribing with Earliest
```
Client receives: msg-1, msg-2, msg-3
Cursor position: Still 0 (no acks yet)
```

#### After Acking msg-1
```
Server updates cursor to: sequence 1
Next subscription from Earliest will start at: msg-2
```

#### After Acking msg-2
```
Server updates cursor to: sequence 2
Next subscription from Earliest will start at: msg-3
```

### Current Behavior in lazysluice

#### InitialPosition::Earliest
- **Subscribes from the consumer group's current cursor**
- Shows messages **after** the last acknowledged message
- Does NOT show messages before the cursor
- This is standard consumer group semantics

#### InitialPosition::Latest
- Subscribes from the latest sequence number
- Only shows NEW messages published after subscription starts
- Does not show any historical messages

### Why Acknowledged Messages "Disappear"

This is **expected behavior** based on consumer group semantics:

1. User subscribes with Earliest, sees messages 1-3
2. User acks message 1
3. Server advances cursor to sequence 1
4. If user resubscribes with Earliest, they start from cursor (seq 1)
5. They now only see messages 2-3 (message 1 is before the cursor)

**This is by design** - consumer groups track processed messages so they aren't redelivered.

---

## Current Limitations

### Issue: Cannot View All Persisted Messages

Currently, there is **no way** to view all messages in a topic regardless of acknowledgment status.

#### What Users Might Expect
- A "view all messages" mode showing everything in the topic
- Ability to inspect message history even after acking
- Debugging capability to see all persisted messages

#### Why It Doesn't Exist
Sluice's Subscribe RPC is **consumer-group-aware**:
- It always respects the cursor position
- There's no "raw topic scan" mode
- This follows standard message broker patterns (Kafka, Pulsar, etc.)

---

## Future Enhancement: "View All Persisted Messages"

### Proposed Solution Options

#### Option A: Add "Reset Cursor" Functionality (Simplest)

**Client-Side Change**:
- Add `r` keybinding in Tail view: "Reset cursor to beginning"
- Sends an ack for sequence 0 (or similar) to reset the cursor
- Then resubscribes with Earliest

**Server-Side Change**:
- None required if we can ack backwards
- OR add a new `ResetCursor` RPC

**Pros**:
- Minimal changes
- Uses existing consumer group semantics

**Cons**:
- Destructive (actually moves cursor back)
- Affects all consumers in the group
- Not safe for production use

---

#### Option B: Add "Browse" Subscription Mode (Recommended)

**Proto Addition**:
```protobuf
enum SubscriptionMode {
  CONSUMER_GROUP = 0;  // Default, respects cursor
  BROWSE = 1;          // Ignores cursor, reads all messages
}

message SubscriptionInit {
  string topic = 1;
  string consumer_group = 2;
  string consumer_id = 3;
  InitialPosition initial_position = 4;
  SubscriptionMode mode = 5;  // NEW FIELD
}
```

**Behavior**:
- `CONSUMER_GROUP` mode: Current behavior (respects cursor)
- `BROWSE` mode: Always starts from sequence 0, never updates cursor

**Implementation**:
- Server checks mode in subscription handler
- If BROWSE, starts from seq 0 and doesn't update cursor on ack
- If CONSUMER_GROUP, uses existing logic

**Client Change**:
- Add `b` keybinding: "Browse all messages"
- Sets subscription mode to BROWSE
- Shows indicator: `Tail [BROWSE | EARLIEST]`

**Pros**:
- Non-destructive (doesn't affect cursor)
- Multiple consumers can browse independently
- Safe for production debugging

**Cons**:
- Requires proto change (backward compatibility concern)
- More complex server implementation

---

#### Option C: Add Separate "ListMessages" RPC (Most Flexible)

**Proto Addition**:
```protobuf
message ListMessagesRequest {
  string topic = 1;
  uint64 start_sequence = 2;  // 0 = from beginning
  uint64 limit = 3;            // Max messages to return
}

message ListMessagesResponse {
  repeated MessageDelivery messages = 1;
  bool has_more = 2;
}

// Add to service
rpc ListMessages(ListMessagesRequest) returns (ListMessagesResponse) {}
```

**Behavior**:
- Unary RPC (not streaming)
- Returns a batch of messages
- Does not use consumer groups or cursors
- Paginated for large topics

**Client Change**:
- Add new "Browse" screen separate from Tail
- Keybinding `b` switches to Browse mode
- Shows all messages with pagination
- Cannot ack from Browse mode (read-only)

**Pros**:
- Clean separation of concerns
- No interaction with consumer groups
- Easy to add pagination, filtering
- Can add later without breaking changes

**Cons**:
- Requires server implementation
- More UI work (pagination, separate screen)
- Messages aren't live-updated

---

### Recommended Approach

**Phase 1** (Immediate):
- ✅ Document cursor behavior (this file)
- ✅ Fix message duplication bug
- ✅ Add visual indicators for acked messages

**Phase 2** (Near-term):
- Implement **Option B: Browse Mode**
- Adds `mode` field to SubscriptionInit (optional, defaults to CONSUMER_GROUP)
- Backward compatible (old clients don't send mode, get default)
- Provides non-destructive message viewing

**Phase 3** (Long-term):
- Implement **Option C: ListMessages RPC**
- Provides better pagination and filtering
- Enables advanced features (search, export, etc.)

---

## Implementation Plan for Browse Mode (Option B)

### Server Changes

**File**: `crates/sluice-proto/proto/sluice/v1/sluice.proto`

```protobuf
enum SubscriptionMode {
  CONSUMER_GROUP = 0;  // Respects cursor, updates on ack
  BROWSE = 1;          // Starts from seq 0, never updates cursor
}

message SubscriptionInit {
  string topic = 1;
  string consumer_group = 2;
  string consumer_id = 3;
  InitialPosition initial_position = 4;
  SubscriptionMode mode = 5;  // Default: CONSUMER_GROUP
}
```

**File**: `crates/sluice-server/src/service/subscribe.rs`

```rust
// In subscription handler
let mode = init.mode();  // Default to CONSUMER_GROUP if not specified

let start_seq = if mode == SubscriptionMode::Browse {
    0  // Always start from beginning in browse mode
} else {
    match init.initial_position {
        InitialPosition::Earliest => get_cursor_from_db(topic, consumer_group),
        InitialPosition::Latest => get_latest_sequence(topic),
    }
};

// When processing acks
if mode == SubscriptionMode::Browse {
    // Don't update cursor in browse mode
    tracing::debug!("Browse mode: skipping cursor update");
} else {
    // Update cursor as normal
    update_cursor(topic, consumer_group, acked_sequence);
}
```

### Client Changes

**File**: `crates/sluice-client/src/connection.rs`

```rust
pub async fn subscribe_browse(
    &mut self,
    topic: &str,
) -> Result<Subscription, Box<dyn std::error::Error>> {
    // Subscribe in BROWSE mode
    let init = SubscriptionInit {
        topic: topic.to_string(),
        consumer_group: "".to_string(),  // Not used in browse mode
        consumer_id: None,
        initial_position: InitialPosition::Earliest,
        mode: SubscriptionMode::Browse,  // NEW
    };
    // ... rest of subscription logic
}
```

**File**: `crates/lazysluice/src/controller.rs`

```rust
// Add browse mode flag
pub struct Controller {
    // ... existing fields ...
    browse_mode: bool,  // NEW
}

// Handle 'b' key to toggle browse mode
KeyCode::Char('b') => {
    if self.state.screen == Screen::Tail {
        self.browse_mode = !self.browse_mode;
        // Restart subscription with new mode
        if let Some(topic) = self.state.current_topic.clone() {
            if self.browse_mode {
                // Subscribe in browse mode
                let sub = client.subscribe_browse(&topic).await?;
                self.subscription = Some(sub);
            } else {
                // Normal subscription
                self.start_subscription(topic).await;
            }
        }
    }
}
```

**File**: `crates/lazysluice/src/ui.rs`

```rust
// Update title to show browse mode
let title = if state.browse_mode {
    format!("Tail [BROWSE | {}]", position_indicator)
} else if state.paused {
    format!("Tail [PAUSED | {}]", position_indicator)
} else {
    format!("Tail [{}]", position_indicator)
};
```

### Testing

1. **Unit Tests**:
   - Test browse mode doesn't update cursor
   - Test multiple browse subscriptions don't interfere

2. **Integration Tests**:
   ```bash
   # Publish messages
   sluicectl publish test "msg1"
   sluicectl publish test "msg2"
   sluicectl publish test "msg3"

   # Subscribe and ack (advances cursor)
   sluicectl subscribe test  # Receives msg1, msg2, msg3
   # Ack msg1 and msg2

   # Normal subscribe (starts from cursor)
   sluicectl subscribe test  # Only receives msg3

   # Browse subscribe (ignores cursor)
   lazysluice browse test    # Receives msg1, msg2, msg3
   ```

3. **Backward Compatibility**:
   - Old clients (don't send mode) work as before
   - New clients with old servers (ignore mode field)

---

## Summary

### Current State
- ✅ Acknowledged messages "disappear" from Earliest view (expected behavior)
- ✅ This is standard consumer group semantics
- ✅ Working as designed

### Near-Term Solution
- Add Browse Mode (Option B)
- Non-destructive message viewing
- Requires proto change but backward compatible
- Estimated effort: 4-6 hours (server + client + tests)

### Long-Term Solution
- Add ListMessages RPC (Option C)
- Better pagination, filtering, search
- Separate from subscription semantics
- Estimated effort: 8-10 hours (full feature)

### References
- Kafka consumer groups: https://kafka.apache.org/documentation/#consumergroups
- Pulsar cursor management: https://pulsar.apache.org/docs/en/concepts-messaging/#acknowledgment
- RabbitMQ acknowledgment: https://www.rabbitmq.com/confirms.html
