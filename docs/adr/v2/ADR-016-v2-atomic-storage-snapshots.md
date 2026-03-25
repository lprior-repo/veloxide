# ADR 016 (v2): Atomic Storage and Replay Snapshots

## Status
Accepted

## Context
1. **Multi-Partition Corruption:** We are using `fjall` with multiple partitions (`events`, `instances`, `timers`). If the engine writes an event to the `events` partition, but crashes before updating the `instances` index partition, the UI will display inconsistent data (e.g., UI says "Running", event log says "Failed").
2. **The Replay Cliff:** An actor rehydrates its state by replaying its event log. If a workflow runs a massive Map loop, generating 20,000 events, rehydration could take seconds. If the engine restarts and 100 actors need to rehydrate 20,000 events each, the startup delay is catastrophic.

## Decision

### 1. Atomic WriteBatches
The `DbWriterActor` is mandated to use `fjall::Batch` for every single state transition. 
- A transition must write the `WorkflowEvent` to the `events` partition AND update the `InstanceSummary` in the `instances` partition in the exact same atomic transaction. 
- If the batch fails to commit, neither partition is updated. This guarantees 100% eventual consistency between the Event Log and the Materialized Views.

### 2. Periodic State Snapshotting
To solve the Replay Cliff, the Engine implements a `snapshots` partition in `fjall`.
- Every $N$ events (e.g., $N=100$), the actor serializes its fully computed in-memory state.
- It sends a `SnapshotTaken { sequence_number, state_bytes }` instruction to the `DbWriterActor`.
- This snapshot is written to the `snapshots` partition as part of the same atomic batch that writes the 100th event.
- On rehydration, the actor reads the latest snapshot, loads the state into memory, and only replays the events from the `events` partition that have a sequence number greater than the snapshot's sequence number.

## Consequences
- **Positive:** Absolute data consistency between the raw log and the UI dashboard.
- **Positive:** Rehydration time is strictly bounded to $O(N)$ events, guaranteeing fast crash recovery regardless of how long the workflow has been running.
- **Negative:** Snapshots consume additional disk space (mitigated by overwriting the previous snapshot for that instance).