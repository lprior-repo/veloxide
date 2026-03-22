bead_id: wtf-xoeh
bead_title: CLI command: wtf admin rebuild-views
phase: 3
updated_at: 2026-03-22T00:10:00Z

# Implementation Summary: wtf admin rebuild-views

## Files Created/Modified

### crates/wtf-cli/src/main.rs
- Added `Admin` command variant with `RebuildViews` subcommand
- Added `AdminCommands` enum with `RebuildViews` { view, namespace, progress, dry_run }
- Integrated `tracing_subscriber::fmt::init()` for logging
- Uses `#[tokio::main]` for async main

### crates/wtf-cli/src/admin.rs (new)
- `RebuildViewsConfig` struct for CLI argument binding
- `RebuildStats` struct for reporting rebuild statistics
- `ViewName` enum: Instances, Timers, Definitions, Heartbeats
- `run_rebuild_views()` async function
- `rebuild_views()` stub (returns zeros - full implementation pending)

### crates/wtf-cli/src/lib.rs
- Added `pub mod admin;` export

### crates/wtf-cli/Cargo.toml
- Added `tracing-subscriber = { workspace = true }` dependency

## CLI Interface

```
wtf admin rebuild-views [OPTIONS]
    --view <VIEW>        Specific view to rebuild (instances|timers|definitions|heartbeats)
    --namespace <NS>    Rebuild only specific namespace
    --progress           Show progress (default: true)
    --dry-run           Print what would be rebuilt without rebuilding
```

## Build Status
- **Compilation**: ✅ Pass (no warnings)
- **Unit Tests**: ✅ 4 tests pass
  - view_name_from_str_instances
  - view_name_from_str_invalid
  - view_name_all_returns_three
  - rebuild_stats_default_is_zero

## Pending Implementation

The `rebuild_views()` function currently returns stub data (zeros). Full implementation requires:

1. Query JetStream for all streams
2. Iterate instances per namespace
3. Replay events to build KV state
4. Stream events to avoid memory issues with large datasets

## Test Coverage

Current tests cover:
- ViewName parsing (valid/invalid)
- ViewName::all() returns 3 views
- RebuildStats default values

Missing tests (require NATS):
- Full rebuild integration
- Error handling
- Idempotency
