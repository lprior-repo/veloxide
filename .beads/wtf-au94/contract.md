# Contract: wtf-au94 — Phase 1 Foundation

## Epic Overview
**Epic ID:** wtf-au94  
**Title:** epic: Phase 1 — Foundation (wtf-common + wtf-storage)  
**Type:** Epic (container)  
**Priority:** 0 (Critical)

## Scope
Establish the shared type system and NATS persistence layer. All other phases depend on this.
- WorkflowEvent closed enum
- Shared error types (WtfError)
- NATS connection manager
- write-ahead append_event()
- JetStream stream provisioning
- KV bucket provisioning
- sled snapshot store
- heartbeat manager

**ADRs:** 013, 014, 015, 019

## Architecture

### Component Diagram
```
wtf-common/
├── types.rs        → InstanceId, NamespaceId, ActivityId, TimerId, WtfError
├── events.rs       → WorkflowEvent (closed enum), EffectDeclaration, RetryPolicy

wtf-storage/
├── nats.rs         → NatsClient, NatsConfig, connect()
├── journal.rs      → append_event(), build_subject()
├── provision.rs    → provision_streams(), verify_streams(), stream_names, subjects
├── kv.rs           → KvStores, provision_kv_buckets(), heartbeat_key(), write_heartbeat()
├── snapshots.rs    → SnapshotRecord, open_snapshot_db(), write_snapshot(), read_snapshot()
├── replay.rs       → ReplayConsumer, create_replay_consumer(), replay_start_seq()
```

### Key Invariants

#### ADR-013 (JetStream Event Log)
- [x] Only `WorkflowEvent` types are appended to `wtf-events` stream
- [x] Subject format: `wtf.log.<namespace>.<instance_id>`
- [x] Four streams: wtf-events, wtf-work, wtf-signals, wtf-archive

#### ADR-014 (NATS KV Materialized View)
- [x] Four KV buckets: wtf-instances, wtf-timers, wtf-definitions, wtf-heartbeats
- [x] heartbeat TTL = 10s (expiry = crash detected)

#### ADR-015 (Write-Ahead Guarantee)
- [x] `append_event()` awaits PublishAck before executing side effects
- [x] Effects embedded in TransitionApplied for replay skipping

#### ADR-019 (Snapshot Recovery)
- [x] sled stores snapshots under `snapshots` tree
- [x] CRC32 checksum validation on read
- [x] Recovery: load snapshot, replay from seq+1

## Preconditions

### For wtf-common
```rust
// Before publishing events, NATS must be connected
let nats_client = connect(&NatsConfig::default()).await?;
```

### For wtf-storage
```rust
// Streams and KV buckets must be provisioned before use
provision_streams(&jetstream).await?;
let kv_stores = provision_kv_buckets(&jetstream).await?;
```

## Postconditions

### append_event()
- [x] Returns `Ok(seq)` only after PublishAck received
- [x] Event serialized as msgpack
- [x] Subject is `wtf.log.<namespace>.<instance_id>`

### write_heartbeat()
- [x] Entry written to `wtf-heartbeats` KV
- [x] Key format: `hb/<instance_id>`
- [x] 10s TTL triggers expiry detection

### SnapshotRecord
- [x] CRC32 checksum computed on creation
- [x] is_valid() returns true when checksum matches

## Child Beads

| Bead ID | Title | Priority | Description |
|---------|-------|----------|-------------|
| wtf-au95 | Foundation integration tests | 1 | Full integration tests covering append_event, provision_streams, replay |
| wtf-au96 | Foundation chaos testing | 0 | Red Queen adversarial testing for foundation components |

## Exit Criteria
- [x] WorkflowEvent closed enum with all variants
- [x] WtfError with all error variants  
- [x] NatsClient connects with retry logic
- [x] append_event awaits PublishAck
- [x] provision_streams creates 4 streams idempotently
- [x] provision_kv_buckets creates 4 buckets idempotently
- [x] SnapshotRecord with CRC32 validation
- [x] ReplayConsumer with tail detection
- [ ] Integration tests pass
- [ ] Chaos tests pass
