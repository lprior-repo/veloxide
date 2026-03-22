bead_id: wtf-xoeh
bead_title: CLI command: wtf admin rebuild-views
phase: 1
updated_at: 2026-03-21T23:58:00Z

# Contract Specification: wtf admin rebuild-views

## Context
- **Feature**: CLI command `wtf admin rebuild-views` for administrative view maintenance
- **Domain terms**:
  - KV stores: Materialized query-side views derived from JetStream event log (ADR-014)
  - Four KV buckets: `wtf-instances`, `wtf-timers`, `wtf-definitions`, `wtf-heartbeats`
  - Rebuild: Full replay of event log to reconstruct KV bucket state
- **Assumptions**:
  - NATS JetStream is running and accessible
  - Event streams exist in NATS
  - KV buckets are provisioned
- **Open questions**: None

## Preconditions
- [ ] NATS JetStream connection is established before rebuild begins
- [ ] All four KV buckets exist (idempotent provision is safe to call)
- [ ] No other writer is actively modifying KV buckets during rebuild (external coordination)

## Postconditions
- [ ] All instances in JetStream have corresponding entries in `wtf-instances` KV
- [ ] All pending timers have corresponding entries in `wtf-timers` KV
- [ ] All workflow definitions have corresponding entries in `wtf-definitions` KV
- [ ] No stale heartbeat entries remain in `wtf-heartbeats` KV
- [ ] Command exits with code 0 on success
- [ ] Progress is reported to stdout during rebuild
- [ ] Idempotent: Running twice produces identical final state

## Invariants
- [ ] KV stores are never the source of truth — JetStream is
- [ ] Rebuild does not delete events from JetStream
- [ ] Concurrent reads during rebuild return consistent (possibly stale) data

## Error Taxonomy
- `WtfError::NatsConnect` — Cannot connect to NATS server
- `WtfError::NatsPublish` — KV read/write operations fail
- `WtfError::StreamNotFound` — Required JetStream stream does not exist
- `WtfError::BucketNotFound` — Required KV bucket does not exist
- `WtfError::InvalidArgument` — Invalid CLI arguments (e.g., non-existent view name)

## Contract Signatures
```rust
// CLI command signature (clap)
async fn rebuild_views(matches: &ArgMatches) -> Result<(), WtfError>

// Core rebuild logic
async fn rebuild_all_views(nats: &NatsConfig) -> Result<RebuildStats, WtfError>
async fn rebuild_single_view(view: &str, nats: &NatsConfig) -> Result<RebuildStats, WtfError>

// Supporting
async fn provision_kv_buckets(js: &Context) -> Result<KvStores, WtfError>
async fn replay_instance(ns: &NamespaceId, id: &InstanceId, stores: &KvStores) -> Result<u64, WtfError>
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| NATS connected | Runtime-checked | `NatsConfig::connect() -> Result<NatsClient, WtfError>` |
| KV buckets exist | Runtime idempotent provision | `provision_kv_buckets()` safe to call repeatedly |
| Valid view name | Runtime argument validation | `clap validator` or explicit match on enum |
| Not during active write | External coordination | Document in help text |

## Violation Examples (REQUIRED)
- VIOLATES <P1>: `rebuild_views()` called without NATS connection → `Err(WtfError::NatsConnect("connection refused"))`
- VIOLATES <P2>: Stream `wtf-events` does not exist → `Err(WtfError::StreamNotFound("wtf-events"))`
- VIOLATES <Q1>: Rebuild completes but some instances missing from `wtf-instances` → Data loss bug
- VIOLATES <Q2>: `rebuild_views()` returns success but exit code is non-zero → Inconsistent signaling

## Ownership Contracts (Rust-specific)
- `NatsConfig`: Owns connection handle; cloned to workers
- `KvStores`: Clone per worker; each worker writes to same buckets (NATS KV is atomic)
- `InstanceId`, `NamespaceId`: Shared, read-only borrowing preferred
- No mutation of input arguments; all fallible operations return `Result`

## Non-goals
- [ ] Snapshot management (sled cache) — separate concern
- [ ] Real-time synchronization — batch rebuild only
- [ ] Selective instance rebuild — full namespace replay only

---

## RebuildStats Response Type

```rust
pub struct RebuildStats {
    pub instances_rebuilt: u64,
    pub timers_rebuilt: u64,
    pub definitions_rebuilt: u64,
    pub events_processed: u64,
    pub duration_ms: u64,
}
```

## CLI Interface

```
wtf admin rebuild-views [OPTIONS]

OPTIONS:
    --view <VIEW>    Rebuild only specific view (instances|timers|definitions|heartbeats)
    --namespace <NS> Rebuild only specific namespace
    --progress       Show progress bar (default: true)
    --dry-run        Print what would be rebuilt without rebuilding
```

## View-Specific Behavior

### wtf-instances
- Replay all `InstanceStarted`, `InstanceCompleted`, `InstanceFailed` events
- Final state = latest event per instance_id

### wtf-timers
- Replay all `TimerCreated`, `TimerFired`, `TimerCancelled` events
- Pending timers = latest non-terminal state per timer_id

### wtf-definitions
- Replay all `DefinitionRegistered` events
- Latest version per namespace/workflow_type

### wtf-heartbeats
- Do NOT rebuild heartbeats — they are ephemeral (10s TTL)
- Clear bucket or skip during rebuild
