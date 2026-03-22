# Contract Specification: wtf admin rebuild-views

## Context

- **Feature**: `wtf admin rebuild-views` — disaster recovery tool to reconstruct NATS KV materialized views from JetStream event log
- **Bead ID**: wtf-creq
- **Bead Title**: wtf-cli: wtf admin rebuild-views — reconstruct NATS KV from JetStream
- **ADR Reference**: ADR-014 (NATS KV as Materialized View)
- **Domain terms**: JetStream event log (source of truth), NATS KV (materialized view), InstanceView, SnapshotRecord, ReplayConsumer
- **Assumptions**: 
  - NATS is available and JetStream streams exist
  - sled snapshots path is accessible for snapshot reconstruction
  - KV buckets are provisioned (idempotent provision)
- **Open questions**: None

## Preconditions

- **P1**: NATS connection must be established before replay begins
  - Enforcement: `Result<(), WtfError>` return type; `WtfError::NatsPublish` on failure
- **P2**: KV buckets must be successfully provisioned before writes
  - Enforcement: `Result<KvStores, WtfError>`; bubbles up provisioning errors
- **P3**: Namespace filter (if provided) must be non-empty string
  - Enforcement: `Option<String>`; empty string treated as "all namespaces"
- **P4**: View filter (if provided) must be one of: instances, timers, definitions, heartbeats
  - Enforcement: `ViewName::parse()` returns `None` for invalid values; CLI exits with error

## Postconditions

- **Q1**: For every instance found in JetStream, an InstanceView is written to `wtf-instances` KV
  - Key format: `<namespace>/<instance_id>`
  - Value: InstanceView with current state derived from replay
- **Q2**: `RebuildStats` accurately reflects:
  - `instances_rebuilt`: count of unique instance_ids processed
  - `timers_rebuilt`: count of timer entries written (TimerScheduled events)
  - `definitions_rebuilt`: count of definition entries written (InstanceStarted events)
  - `events_processed`: total events replayed
  - `duration_ms`: wall-clock time in milliseconds
- **Q3**: On dry-run mode, no KV writes occur
  - Verification: `stores` are not accessed in dry-run path
- **Q4**: InstanceView reflects the state after the LAST event in the stream for that instance
  - Verification: replay continues until `TailReached` for each instance

## Invariants

- **I1**: KV state never advances beyond JetStream state (KV ≤ JetStream)
- **I2**: Each instance's final state is derived by applying ALL events in sequence order
- **I3**: SnapshotRecord's checksum must be valid when writing to KV; invalid snapshots fall back to full replay

## Error Taxonomy

| Error Variant | Trigger Condition |
|---|---|
| `WtfError::NatsPublish` | NATS connection failure, consumer creation failure, KV write failure |
| `WtfError::NatsTimeout` | JetStream consumer timeout waiting for messages |
| `WtfError::InstanceNotFound` | (Not expected during rebuild — all instances are discovered from stream) |
| `WtfError::ReplayDivergence` | Replayed state differs from expected (for validation runs) |

## Contract Signatures

```rust
// CLI entry point
pub async fn run_rebuild_views(config: RebuildViewsConfig) -> anyhow::Result<std::process::ExitCode>

// Internal rebuild logic  
async fn rebuild_views(
    stores: &KvStores,
    namespace_filter: &Option<String>,
    view_filter: Option<&ViewName>,
    show_progress: bool,
) -> Result<RebuildStats, WtfError>

// Dry-run mode (no writes)
fn run_dry_run(config: &RebuildViewsConfig) -> anyhow::Result<std::process::ExitCode>
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: NATS connected | Runtime | `connect()` returns `Result<Client, WtfError>` |
| P2: KV provisioned | Runtime | `provision_kv_buckets()` returns `Result<KvStores, WtfError>` |
| P3: Namespace filter valid | Runtime | `Option<String>` — empty means all |
| P4: View filter valid | Compile-time via `FromStr` | `ViewName::parse()` → `Option<ViewName>` |

## Violation Examples (REQUIRED)

- **VIOLATES P1**: `connect(&NatsConfig { host: "nonexistent" })` → returns `Err(WtfError::NatsPublish { message: "..." })`
- **VIOLATES P2**: `provision_kv_buckets(invalid_js)` → returns `Err(WtfError::NatsPublish { message: "..." })`
- **VIOLATES Q1**: If replay crashes mid-stream for an instance, that instance's KV entry is stale/incomplete — but rebuild is re-runnable and idempotent
- **VIOLATES Q3**: `run_dry_run` does not access `stores` — enforced by code structure (early return before `provision_kv_buckets`)

## Ownership Contracts

- `stores: &KvStores` — shared borrow, no ownership transfer
- `namespace_filter: &Option<String>` — shared borrow, read-only
- `view_filter: Option<&ViewName>` — shared borrow, read-only
- All mutations happen via async KV writes, not in-place mutation

## Non-goals

- Snapshot reconstruction from sled for `wtf-snapshots` KV (mentioned in bead description but deferred)
- Real-time watch/stream updates (this is offline batch reconstruction)
- Multi-threaded parallel instance processing (sequential for simplicity)
