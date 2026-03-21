# Black Hat Review — Bead wtf-07zs: Heartbeat-driven crash recovery

## Security Review

### Input Validation
- `instance_id` comes from NATS KV watch stream - validated by `InstanceId::new()` which validates NATS subject safety
- `namespace`, `workflow_type` come from KV - no direct user input

### Denial of Service
- Could an attacker trigger many `HeartbeatExpired` events to overwhelm the system?
- Each recovery spawns an actor which is bounded by `max_instances` capacity check
- If at capacity, `StartError::AtCapacity` is returned

### Resource Leaks
- Recovery spawns actors supervised by orchestrator
- Actors are properly registered/deregistered via supervisor events
- No unbounded resource creation

### Error Handling
- All NATS operations that could fail are wrapped in `if let Err` with logging
- No unwrap/expect on fallible operations
- No panic on error paths

## Code Quality Review

### Rust Safety
- `forbid(unsafe_code)` in effect
- No `unsafe` blocks in recovery implementation
- All error handling via `thiserror` enums

### Error Propagation
- Errors logged at appropriate levels (error for infrastructure issues, warn for expected cases)
- No silent failures

### Concurrency
- Actor message handling is single-threaded - no data races
- State mutations only in actor's `&mut self`

## Defects Found

None. The implementation follows secure coding practices and has no obvious security vulnerabilities.

## Status

**STATUS: APPROVED**
