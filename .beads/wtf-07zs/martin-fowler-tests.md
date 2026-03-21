# Martin Fowler Test Plan

## Happy Path Tests
- `test_recovery_spawns_new_instance_when_heartbeat_expires_for_unknown_instance`
- `test_recovery_replays_from_last_snapshot_sequence`

## Error Path Tests
- `test_recovery_skips_when_instance_metadata_not_found_in_kv`
- `test_recovery_fails_gracefully_when_nats_unavailable`
- `test_recovery_fails_gracefully_when_replay_consumer_creation_fails`

## Edge Case Tests
- `test_heartbeat_expired_ignored_when_instance_still_in_active_registry`
- `test_concurrent_heartbeat_expired_for_same_instance_only_spawns_once`
- `test_recovery_uses_correct_instance_id_from_expired_heartbeat`

## Contract Verification Tests
- `test_invariant_one_recovery_in_flight_per_instance`
- `test_invariant_recovery_never_replays_before_snapshot_seq`

## Contract Violation Tests
- `test_heartbeat_expired_for_active_instance_does_not_trigger_spurious_recovery`
  Given: `HeartbeatExpired { instance_id: "inst-abc" }` received but `inst-abc` IS in `state.active`
  When: `handle_heartbeat_expired` is called
  Then: returns `Ok(None)` — no recovery spawned

- `test_heartbeat_expired_for_unknown_instance_without_metadata_skips_recovery`
  Given: `HeartbeatExpired { instance_id: "inst-xyz" }` received, `inst-xyz` NOT in `state.active`, KV GET returns `None`
  When: `handle_heartbeat_expired` is called
  Then: returns `Ok(None)` with warning log — not an error

## Given-When-Then Scenarios

### Scenario 1: Instance crashes and recovers via heartbeat expiry
Given: A WorkflowInstance `inst-abc` was running, emitting heartbeats every 5s
And: The instance process crashes (stops emitting heartbeats)
When: 10s pass and the heartbeat TTL expires
Then: The heartbeat watcher sends `HeartbeatExpired { instance_id: "inst-abc" }` to the orchestrator
And: The orchestrator queries `wtf-instances` KV for metadata
And: Metadata exists (namespace, workflow_type, paradigm)
And: A new WorkflowInstance is spawned with the same instance_id
And: The new instance replays from the last `SnapshotTaken` sequence
And: The new instance resumes heartbeat emission

### Scenario 2: Heartbeat expires for healthy instance (timing glitch)
Given: A WorkflowInstance `inst-abc` is running and healthy
But: The instance is momentarily slow and misses one heartbeat interval
When: The heartbeat watcher sees the TTL expire before the next heartbeat
Then: The orchestrator receives `HeartbeatExpired { instance_id: "inst-abc" }`
And: The orchestrator sees `inst-abc` IS in `state.active`
And: No recovery is triggered (the instance is still alive)
And: The next heartbeat from the instance refreshes the KV entry

### Scenario 3: Heartbeat expires for completed instance
Given: A WorkflowInstance `inst-abc` completed successfully and deregistered itself
And: The heartbeat entry was deleted
When: A stale `HeartbeatExpired` event arrives (should not happen but is defensively handled)
Then: The orchestrator sees `inst-abc` is NOT in `state.active`
And: The orchestrator queries KV and finds no metadata
And: Recovery is skipped with a warning
