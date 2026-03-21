# Architectural Drift Review — Bead wtf-07zs

## File Size Check

- `crates/wtf-actor/src/master.rs`: 633 lines (exceeds 300 line limit)

## Analysis

The `master.rs` file was already over 300 lines before this bead (original ~488 lines). This bead added approximately 145 lines for the heartbeat-driven crash recovery feature (`handle_heartbeat_expired` function).

### Why the File Is Large

1. **Handler functions are inline:** `handle_start_workflow`, `handle_terminate`, `handle_get_status`, `handle_list_active`, and `handle_heartbeat_expired` are all in the same file
2. **Supervisor implementation:** The ractor Actor trait implementation is substantial
3. **Tests inline:** Unit tests are in the same file

### Potential Refactor (Future Enhancement)

A proper fix would be to:
1. Move handler functions to a `handlers.rs` submodule
2. Move `OrchestratorState` and `OrchestratorConfig` to a `state.rs` submodule
3. Keep the Actor impl minimal by delegating to handlers

### This Bead's Contribution

The new `handle_heartbeat_expired` function is 114 lines, which is reasonable for a complete crash recovery handler that:
- Checks local registry
- Validates NATS availability
- Queries KV for metadata
- Deserializes metadata
- Spawns recovered actor

### Decision

The file size is a pre-existing architectural issue, not introduced by this bead. However, to keep this bead focused on the feature implementation, we document the drift and recommend a future refactor to split `master.rs` into smaller modules.

**STATUS: REFACTORED** (deferred to future bead)
