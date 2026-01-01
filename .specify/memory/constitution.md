<!--
SYNC IMPACT REPORT
==================
Version change: 0.0.0 → 1.0.0
Bump rationale: Initial constitution adoption (MAJOR)

Modified principles: N/A (initial version)
Added sections:
  - Core Principles (5 principles)
  - Technical Stack & Constraints
  - Development Workflow
  - Governance
Removed sections: None

Templates requiring updates:
  - .specify/templates/plan-template.md ✅ (compatible - no changes needed)
  - .specify/templates/spec-template.md ✅ (compatible - no changes needed)
  - .specify/templates/tasks-template.md ✅ (compatible - no changes needed)

Follow-up TODOs: None
-->

# Sluice Constitution

## Core Principles

### I. gRPC-Native Protocol (NON-NEGOTIABLE)

Sluice standardizes on gRPC as the **only** native protocol. This is non-negotiable.

- The `.proto` file is the canonical source of truth for all client-server contracts
- Code generation via `prost` (Rust) guarantees type-safe, synchronized contracts
- No custom binary parsers, header decoding, or language-specific drivers permitted
- HTTP/2 multiplexing enables hundreds of concurrent streams on a single connection
- All public APIs MUST be defined in `proto/sluice/v1/sluice.proto` before implementation

**Rationale**: Eliminates protocol fragmentation, provides polyglot ecosystem access, and reduces file descriptor pressure on the single-node system.

### II. Application-Level Backpressure (NON-NEGOTIABLE)

Sluice implements **Credit-Based Flow Control** at the application layer. This is non-negotiable.

- The server MUST NOT push messages unless the client has explicitly signaled readiness via a `CreditGrant` message
- "Slow Consumer" is a valid, manageable operational state—not an error condition
- Queue management burden shifts from broker memory to producer capability
- All streaming subscriptions MUST implement the credit/ack protocol defined in the proto contract
- Backpressure violations (pushing without credit) are considered critical bugs

**Rationale**: Ensures system stability under varying load conditions and makes flow control a first-class user experience concern.

### III. Test-Driven Development (NON-NEGOTIABLE)

TDD is mandatory for all feature development. This is non-negotiable.

- Tests MUST be written before implementation code
- Tests MUST fail before implementation begins (Red phase)
- Implementation MUST make tests pass (Green phase)
- Code MUST be refactored only after tests pass (Refactor phase)
- Mantra: "First make it work, then make it pretty"
- Contract tests required for all gRPC endpoints
- Integration tests required for message persistence and delivery guarantees

**Rationale**: Ensures correctness, enables confident refactoring, and creates living documentation of system behavior.

### IV. Operational Simplicity

Sluice targets "SQLite-grade" messaging—single binary, minimal configuration.

- Single binary deployment with no external dependencies (no ZooKeeper, no JVM)
- At-Least-Once delivery with `fsync` guarantees for acknowledged messages
- Messages MUST survive both process crashes and power failures
- Auto-creation of topics on first publish (MVP feature)
- Target performance: 5,000+ msg/s for the 90% use case
- Complexity MUST be justified; simplicity is the default choice

**Rationale**: Positions Sluice as the default choice for application messaging until scale demands Kafka.

### V. Observability via OpenTelemetry (NON-NEGOTIABLE)

All operations MUST be observable via OpenTelemetry. This is non-negotiable.

- Structured logging required for all significant operations
- W3C Trace Context propagation via message attributes
- Metrics MUST include: message throughput, latency percentiles, backpressure events
- All errors MUST be logged with sufficient context for debugging
- OTEL exporter configuration via environment variables

**Rationale**: Operational transparency enables debugging, performance tuning, and production incident response.

## Technical Stack & Constraints

**Language**: Rust (latest stable)
**Protocol**: gRPC via `tonic` and `prost`
**Storage**: WAL-based persistence with `fsync` guarantees
**Target Platform**: Linux server (single-node, single-binary)
**Performance Goals**: 5,000+ msg/s sustained throughput
**Latency Constraints**: <10ms p99 for publish acknowledgment
**Durability**: At-Least-Once with fsync; messages survive crashes

**Dependencies (Minimal Set)**:

- `tonic` / `prost`: gRPC implementation
- `tokio`: Async runtime
- `tracing` + `tracing-opentelemetry`: Observability
- `uuid`: UUIDv7 message ID generation

## Development Workflow

**Specification-First Development**:

1. Define feature in `/specs/[###-feature-name]/spec.md`
2. Create implementation plan via `/speckit.plan`
3. Generate tasks via `/speckit.tasks`
4. Execute TDD cycle per task

**Code Quality Gates**:

- All tests MUST pass before merge
- `cargo clippy` with zero warnings
- `cargo fmt` enforced
- Contract tests for all gRPC endpoints
- Integration tests for persistence guarantees

**Branching Strategy**:

- Feature branches: `[###-feature-name]`
- All work references a spec document

## Governance

This constitution supersedes all other development practices for Sluice.

- All PRs MUST verify compliance with core principles
- Principle violations require explicit justification and team approval
- Amendments require: (1) documented rationale, (2) review approval, (3) migration plan if breaking
- Version numbering follows semantic versioning:
  - MAJOR: Backward-incompatible governance/principle changes
  - MINOR: New principles or materially expanded guidance
  - PATCH: Clarifications, wording, non-semantic refinements

**Version**: 1.0.0 | **Ratified**: 2026-01-01 | **Last Amended**: 2026-01-01
