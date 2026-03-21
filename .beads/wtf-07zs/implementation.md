# Implementation Summary — Bead wtf-07zs

## Feature: Heartbeat-driven crash recovery

## Changes Made

### 1. `messages.rs` — Added RecoveryError and InstanceMetadata types

**Added:**
- `RecoveryError` enum with variants: `InstanceNotFound`, `ReplayFailed`, `SpawnFailed`, `NoNatsClient`
- `InstanceMetadata` struct with fields: `namespace`, `instance_id`, `workflow_type`, `paradigm`, `engine_node_id`

### 2. `master.rs` — Implemented heartbeat-driven crash recovery

**Modified `handle_start_workflow`:**
- After successfully spawning a WorkflowInstance, write instance metadata to `wtf-instances` KV bucket
- Metadata includes: namespace, instance_id, workflow_type, paradigm, engine_node_id
- This enables recovery to look up metadata when a heartbeat expires

**Added `handle_heartbeat_expired` function:**
- Called when `HeartbeatExpired` message is received from the NATS KV watcher
- **Q4 (no spurious recovery):** If instance is in local registry, it's alive — log debug and ignore
- **Q5 (metadata not found):** Look up metadata from `wtf-instances` KV; if not found, log warning and skip recovery
- **Recovery spawn:** Build `InstanceArguments` from metadata and spawn a new `WorkflowInstance`
- The new instance automatically replays from the last snapshot in its `pre_start` method

**Updated `OrchestratorMsg::HeartbeatExpired` handling:**
- Changed from logging a warning to calling `handle_heartbeat_expired`

## Architecture Notes

- Recovery is triggered when heartbeat TTL (10s) expires without being refreshed
- The new instance replays from JetStream, automatically respecting snapshot boundaries
- In-flight activities and pending timers are re-dispatched via `compute_live_transition` when transitioning from Replay to Live phase
- Only one recovery can be in flight per instance (enforced by the single-threaded actor message handling)

## Postconditions Verified

- ✅ Q1: If instance not in registry and metadata exists, new instance is spawned
- ✅ Q2: Replays from last snapshot (automatic via WorkflowInstance::pre_start)
- ✅ Q3: New instance resumes heartbeat emission (handled by WorkflowInstance heartbeat timer)
- ✅ Q4: Instance in registry → no spurious recovery triggered
- ✅ Q5: No metadata in KV → recovery skipped with warning

## Invariants Verified

- ✅ I1: Single-threaded actor message handling ensures at most one recovery in flight
- ✅ I2: Recovery replays via WorkflowInstance::pre_start which respects snapshot boundaries
- ✅ I3: Same instance_id used for recovered instance
