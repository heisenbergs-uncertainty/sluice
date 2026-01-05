# Feature Specification: lazysluice TUI client

**Feature Branch**: `002-lazysluice-tui`  
**Created**: 2026-01-04  
**Status**: Draft  
**Input**: User description: "lazysluice - Terminal User Interface (TUI) Client for Sluice Message Broker; standalone client; keyboard-driven/vim-like navigation; real-time monitoring; fast startup; default connect to a local server endpoint."

## Clarifications

### Session 2026-01-04

- Q: Should lazysluice automatically reconnect when the gRPC connection/subscribe stream drops? → A: Auto-reconnect: keep retrying with backoff until connected.
- Q: For the MVP, how does the user choose the topic to tail/publish? → A: Show a selectable list of topics retrieved from the server.
- Q: Should users be able to configure the subscribe “credits” window size? → A: Yes: configurable via CLI/UI.
- Q: How should ACK work in the MVP? → A: Manual per-message ack (user selects a message and presses a key).
- Q: What’s the MVP connection security posture? → A: TLS optional via flags/config (plaintext default for localhost).

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Connect and live-tail a topic (Priority: P1)

As an operator/developer, I want to connect to a running Sluice server and live-tail messages for a chosen topic so that I can quickly observe traffic and verify system behavior.

**Why this priority**: This is the core “monitor/explore” value. If this works, the tool is immediately useful even without any advanced features.

**Independent Test**: Start a local Sluice server, publish a few messages with any existing client, then use lazysluice to connect and confirm the messages appear in the terminal in real time.

**Acceptance Scenarios**:

1. **Given** a running Sluice server, **When** the user starts lazysluice with no configuration, **Then** it attempts to connect using sensible defaults and shows a clear connection status.
2. **Given** a successful connection, **When** the user selects a topic from the server-provided topic list and starts tailing from “latest”, **Then** new messages for that topic appear as they arrive.
3. **Given** the server is unreachable, **When** the user starts lazysluice, **Then** the UI stays responsive and shows a clear error plus a retry path.

---

### User Story 2 - Publish a test message (Priority: P2)

As an operator/developer, I want to publish a message to a topic from within the terminal so that I can quickly test a pipeline or reproduce issues without switching tools.

**Why this priority**: Publishing is the simplest “management” action and complements live tailing for a tight feedback loop.

**Independent Test**: With a running Sluice server, publish a message from lazysluice and verify it appears in the tail view (either in lazysluice itself or another subscriber).

**Acceptance Scenarios**:

1. **Given** a successful connection, **When** the user selects a topic from the server-provided topic list, enters message content, and confirms publish, **Then** the UI shows a success confirmation including a unique message identifier.
2. **Given** message validation fails (e.g., invalid topic name or oversized payload), **When** the user attempts to publish, **Then** the UI shows the error and preserves the draft for correction.

---

### User Story 3 - Control subscription behavior (Priority: P3)

As an operator/developer, I want to control how I subscribe (start position, consumer group, pausing/resuming) and acknowledge messages so that I can debug consumption behavior and avoid losing my place.

**Why this priority**: These controls turn the tool from a “tail -f” into a practical debugging client for real-world flows.

**Independent Test**: Subscribe from “earliest” and from “latest” and verify behavior differs. Run two clients in the same consumer group and confirm takeover behavior is communicated.

**Acceptance Scenarios**:

1. **Given** an active tail, **When** the user pauses the stream, **Then** messages stop rendering until the user resumes.
2. **Given** the user chooses “earliest” start, **When** tailing begins, **Then** the oldest available messages are shown before newer ones.
3. **Given** another client takes over the same consumer group, **When** the server terminates the stream, **Then** lazysluice shows a clear notification and offers a one-step reconnect.
4. **Given** a message is selected in the UI, **When** the user triggers acknowledge, **Then** lazysluice acknowledges that specific message and visibly marks it as acknowledged in-session.

---

### Edge Cases

- Server endpoint is unreachable at startup (offline, wrong address, refused connection).
- Server becomes unreachable while tailing (temporary network loss) and later recovers (client should recover without manual restart).
- Subscription stream is terminated by server due to consumer group takeover.
- Message payload is not human-readable (binary) or is very large.
- Message rate is extremely high (UI must remain responsive and usable).
- Topic list is empty or fails to load.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: lazysluice MUST run as a standalone terminal application. Any required server-side support MUST be additive (no breaking changes) and limited to enabling the MVP UX (e.g., topic discovery).
- **FR-002**: lazysluice MUST connect to a running Sluice server using sensible defaults when the user provides no configuration.
- **FR-003**: lazysluice MUST provide a keyboard-driven interface optimized for fast navigation (including a discoverable help/shortcuts view).
- **FR-004**: lazysluice MUST retrieve and display a selectable list of available topics from the server.
- **FR-005**: Users MUST be able to select a topic from the topic list and start a live tail from “latest”.
- **FR-006**: Users MUST be able to start a subscription from “earliest” for debugging/backfill inspection.
- **FR-007**: Users MUST be able to publish a message to a selected topic and receive an explicit success/failure result.
- **FR-008**: lazysluice MUST remain responsive during transient failures (startup connection failures, mid-stream disconnects) and MUST automatically retry reconnecting until successful (while showing current connection status).
- **FR-009**: lazysluice MUST support pausing/resuming rendering of the live tail without exiting the program.
- **FR-010**: lazysluice MUST allow selecting a consumer group identifier for subscriptions.
- **FR-011**: lazysluice MUST allow configuring the subscription credit window size.
- **FR-012**: lazysluice MUST support manual per-message acknowledgement (the user selects a message and explicitly acknowledges it).
- **FR-013**: lazysluice MUST display message metadata needed for debugging at a glance (identifier, sequence/order, timestamp, and attributes when present).
- **FR-014**: For non-text message payloads, lazysluice MUST provide a safe display fallback that does not corrupt the terminal (e.g., a readable preview and an indication that content is binary).
- **FR-015**: lazysluice MUST support optional TLS configuration for connecting to a remote Sluice server while keeping plaintext as the default for localhost.

### Acceptance Criteria

- **AC-001 (FR-002)**: Launching lazysluice with no configuration results in either a successful connection or a clear, actionable error state.
- **AC-002 (FR-004)**: The UI shows a topic list retrieved from the server, or a clear error state if it cannot be loaded.
- **AC-003 (FR-005)**: Starting a tail from “latest” shows new messages as they arrive, without requiring a restart.
- **AC-004 (FR-006)**: Starting a tail from “earliest” shows the oldest available messages first.
- **AC-005 (FR-007)**: Publishing a message reports either success (including a unique message identifier) or a clear failure reason.
- **AC-006 (FR-008)**: If the connection drops during tailing, the UI remains usable and automatically retries reconnecting until it succeeds (with user-visible status).
- **AC-007 (FR-011)**: The user can set a credit window size and the value is used for subsequent subscriptions in the session.
- **AC-008 (FR-012)**: The user can acknowledge a selected message, and the UI shows an in-session acknowledgement indicator for that message.
- **AC-009 (FR-015)**: When TLS configuration is provided, lazysluice connects using TLS; when not provided, lazysluice uses plaintext defaults suitable for localhost.

### Key Entities *(include if feature involves data)*

- **Server Endpoint**: Where lazysluice connects (address and optional connection settings).
- **Topic**: Named stream that messages are published to and consumed from.
- **Subscription Session**: A live tail session defined by topic, consumer group, and start position.
- **Message**: Delivered unit containing identifier, order/sequence, timestamp, attributes, and payload.
- **Operator Preferences (Session-local)**: Current view state such as selected topic, paused state, and chosen start position.

## Assumptions

- The server exposes a stable client-facing interface that supports publishing and subscribing to topics.
- Authentication/authorization is not required for the MVP client experience unless the server is configured to require it.
- The server provides a way for clients to discover available topics (a topic list).

## Dependencies

- A running, reachable Sluice server instance.
- Server support for topic discovery.
- A terminal environment suitable for interactive keyboard-driven applications.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: First usable screen appears in under 1 second on a typical developer laptop.
- **SC-002**: A user can connect and begin tailing a topic in under 30 seconds from a cold start on localhost.
- **SC-003**: When messages are being published to a tailed topic, they appear in the UI within 250 ms of arrival under normal local-network conditions.
- **SC-004**: In an informal “hallway test” of 10 first-time users (developers), at least 9/10 can publish a test message and confirm it was accepted on their first attempt, without assistance, using only the in-app help.

## Out of Scope (for this feature)

- Changes unrelated to enabling topic discovery (topic list) for the client.
- Persisting long-term user configuration beyond basic defaults (unless it is trivial and does not create operational overhead).
- Deep protocol debugging tools (packet-level tracing) beyond what is needed for operator visibility.
