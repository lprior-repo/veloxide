# Implementation Summary: wtf-creq

## Bead ID
wtf-creq

## Feature
`wtf admin rebuild-views` — disaster recovery tool to reconstruct NATS KV from JetStream

## Files Changed

### `crates/wtf-cli/src/commands/admin.rs`
- Full implementation of `rebuild_views()` function
- Added `InstanceView` struct for KV entries
- Added `DiscoveredInstance` struct for instance tracking
- Implemented `discover_instances()` — scans JetStream stream for unique (namespace, instance_id) pairs
- Implemented `rebuild_instance()` — replays events from seq=1 for each instance and derives state
- Implemented `apply_event_to_state()` — pure function to update derived state from events
- Added `parse_instance_from_subject()` — parses subject string into namespace/instance_id
- All existing tests preserved and new tests added for parsing and state derivation

### `crates/wtf-cli/Cargo.toml`
- Added `indicatif = "0.17"` for progress bar support
- Added `futures-util = "0.3"` for StreamExt combinators
- Added `async-nats` for direct JetStream consumer creation
- Added `rmp-serde` for event deserialization
- Added `bytes` for payload handling

### `crates/wtf-linter/src/lib.rs`
- Fixed pre-existing bug: removed call to non-existent `rules::check_random_in_workflow()`

### `Cargo.toml` (workspace)
- Added `futures-util = "0.3"` to workspace dependencies

## Contract Clause Mapping

| Contract Clause | Implementation |
|---|---|
| P1: NATS connected | `connect()` via `wtf_storage::nats` |
| P2: KV provisioned | `provision_kv_buckets()` via `wtf_storage::kv` |
| P3: Namespace filter | `namespace_filter: Option<String>` passed to `discover_instances()` |
| P4: View filter valid | `ViewName::parse()` with CLI exit on invalid |
| Q1: InstanceView written | `stores.instances.put()` in `rebuild_instance()` |
| Q2: Stats accurate | `RebuildStats` accumulated during replay |
| Q3: Dry-run no writes | Early return in `run_dry_run()` before any I/O |
| Q4: Final state from replay | Sequential event replay with state accumulation |

## Architecture

```
rebuild_views()
  ├── discover_instances()    # Scan stream, collect unique (ns, id) pairs
  │   └── Push consumer over wtf.log.>
  └── rebuild_instance()      # For each instance
      ├── Create push consumer for subject
      ├── Replay events sequentially
      ├── apply_event_to_state() # Pure state derivation
      └── Write InstanceView to KV
```

## Test Results
- 9 unit tests passing
- All new tests for parsing and state derivation
- Existing tests preserved
