bead_id: wtf-wygu
bead_title: Long-running activity heartbeat mechanism
phase: red-queen
updated_at: 2026-03-22T01:20:00Z

# Red Queen Report

## Adversarial Testing Analysis

### Attack Vectors Considered

1. **Oversized Progress Payload**
   - Attempt: Send progress string > 1KB
   - Expected: Returns `Err(WtfError::InvalidInput)`
   - Implementation: Correctly checks `progress_bytes.len() > MAX_HEARTBEAT_PROGRESS_BYTES`
   - Status: **DEFENDED**

2. **Send After Stop**
   - Attempt: Call `heartbeat.send()` after `heartbeat.stop()`
   - Expected: Returns `Err(WtfError::HeartbeatStopped)`
   - Implementation: Checks `stopped.load(SeqCst)` before sending
   - Status: **DEFENDED**

3. **Concurrent Send/Stop**
   - Attempt: Call `send()` and `stop()` simultaneously from different tasks
   - Analysis: `AtomicBool` with `SeqCst` ordering provides safe synchronization
   - Status: **DEFENDED**

4. **HeartbeatSender Clone Semantics**
   - Attempt: Clone sender, stop one, verify other still works
   - Implementation: Both clones share same `Arc<AtomicBool>`, stop state is shared
   - Analysis: This is the intended behavior — stopping one handle stops the "session"
   - Status: **BY DESIGN**

5. **Empty Progress String**
   - Attempt: Send empty progress string
   - Expected: Should be allowed
   - Implementation: Empty string has 0 bytes < 1KB, passes validation
   - Status: **DEFENDED**

6. **Exactly 1KB Progress String**
   - Attempt: Send progress string of exactly 1024 bytes
   - Expected: Should be allowed
   - Implementation: Check is `> MAX_HEARTBEAT_PROGRESS_BYTES`, so exactly 1KB is OK
   - Status: **DEFENDED**

7. **NATS Publish Failure**
   - Attempt: JetStream publish fails
   - Expected: Returns `Err(WtfError::NatsPublish)`
   - Implementation: Propagates error from `append_event()`
   - Status: **DEFENDED**

### Test Cases Run
- `cargo test -p wtf-worker --lib` — 34 tests passed
- Contract violation tests verified in code review

### Defects Found
None.

### Conclusion
**ALL ATTACKS DEFENDED** — Implementation is robust against adversarial conditions.
