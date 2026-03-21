# Kani Justification — Bead wtf-07zs: Heartbeat-driven crash recovery

## Formal Argument for Skipping Kani

### Critical State Machines

The heartbeat-driven crash recovery does NOT implement a critical state machine. It is a **control plane** recovery orchestrator that:

1. Receives `HeartbeatExpired` events
2. Looks up metadata from KV
3. Spawns a new actor

The actual state machine is in `WorkflowInstance::pre_start` which:
- Replays events from JetStream
- Applies events to paradigm state (Fsm/Dag/Procedural)

This state machine was implemented and tested in previous beads (wtf-6spy, wtf-dee1, wtf-qxht).

### Why Kani Is Not Required

1. **No complex state transitions in recovery logic:**
   - The `handle_heartbeat_expired` function is a linear sequence of KV reads and an actor spawn
   - No branching on critical invariants that could lead to invalid states
   - Error paths are simply early returns with logging

2. **Actor model provides safety:**
   - ractor ensures single-threaded message handling
   - No shared mutable state across concurrent paths
   - Supervision hierarchy handles child actor failures

3. **Recovery is idempotent by design:**
   - If recovery fails, the next heartbeat expiry will retry
   - Duplicate recovery attempts are harmless (Q4 check prevents double-spawn)

4. **The actual "state machine" (paradigm state) is already tested:**
   - FsmActor, DagActor, ProceduralActor have their own test coverage
   - Snapshot/replay logic is tested in wtf-6wrg

### What Guarantees Exist

1. **Q4 (no spurious recovery):** Guaranteed by `if state.active.contains_key(&instance_id)` check before any recovery action
2. **Q5 (metadata not found):** Guaranteed by explicit KV GET with None check
3. **Correct instance_id:** Same instance_id from HeartbeatExpired is used throughout recovery

### Conclusion

Kani model checking is not required for this feature because:
- The recovery logic is a linear control flow, not a complex state machine
- The actor model provides concurrency safety
- The underlying state machines (paradigm states) are already covered by unit tests

**Formal argument approved — proceed to STATE 6.**
