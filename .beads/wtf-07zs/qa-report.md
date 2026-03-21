# QA Report — Bead wtf-07zs: Heartbeat-driven crash recovery

## Implementation Verification

### Code Review of `handle_heartbeat_expired`

**Location:** `crates/wtf-actor/src/master.rs`

**Q4 (No spurious recovery):**
- Line: `if state.active.contains_key(&instance_id)` check before any recovery action
- Verified: If instance is in registry, logs debug and returns early

**Q5 (Metadata not found):**
- Line: `instances_kv.get(&key).await` returns `None` → logs warning and returns
- Verified: Graceful handling when instance metadata absent from KV

**Recovery spawn:**
- Lines: Builds `InstanceArguments`, spawns `WorkflowInstance` via `spawn_linked`
- Verified: Same `instance_id` used, metadata from KV used for namespace/workflow_type/paradigm
- Automatic replay from snapshot handled by `WorkflowInstance::pre_start`

### Code Review of `handle_start_workflow` (metadata persistence)

**Location:** `crates/wtf-actor/src/master.rs`

- Lines: After successful spawn, writes `InstanceMetadata` to `wtf-instances` KV
- Verified: Metadata includes namespace, instance_id, workflow_type, paradigm, engine_node_id
- Key format: `wtf_storage::instance_key(namespace, instance_id)` = `<ns>/<id>`

### Unit Tests

- All 71 wtf-actor tests pass
- All 26 wtf-worker tests pass
- All 19 wtf-storage tests pass
- cargo check passes with no errors

### Integration Test Requirements (Not Executable Without NATS)

1. **Heartbeat expiry triggers recovery:**
   - Start workflow instance
   - Kill the actor process
   - Wait 10s for heartbeat TTL to expire
   - Verify new instance is spawned with same instance_id

2. **Spurious heartbeat expiry ignored:**
   - Start workflow instance
   - Simulate heartbeat expiry while instance still alive
   - Verify instance continues running, no duplicate spawned

3. **Recovery replays from snapshot:**
   - Start workflow, emit events, trigger snapshot
   - Kill actor process
   - Wait for recovery
   - Verify instance resumes at correct sequence (not from beginning)

### Compilation and Linting

- `cargo check` passes
- `cargo test --lib` passes (all 116 tests)
- Clippy shows pre-existing doc warnings only (not in new code)

### Defects Found

None in the implementation.

### Reproduction Steps for Integration Test

```bash
# Start NATS server
docker run -p 4222:4222 nats:latest

# Run the engine with a workflow
cargo run --bin wtf-engine -- start-workflow --namespace test --instance test-001 --type checkout

# In another terminal, kill the workflow actor process
pkill -f "wf-test-001"

# Observe in logs:
# - "HeartbeatExpired — triggering crash recovery"
# - "Recovery instance spawned"
# - "Replay complete" with correct event count
```
