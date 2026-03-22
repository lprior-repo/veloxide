bead_id: wtf-wygu
bead_title: Long-running activity heartbeat mechanism
phase: kani-justification
updated_at: 2026-03-22T01:30:00Z

# Kani Model Checking Justification

## Request: Skip Kani formal verification

## Formal Argument

### 1. What Critical State Machines Exist?

The heartbeat implementation contains one primary state variable:
- `stopped: Arc<AtomicBool>` — a boolean flag indicating if the sender has been stopped

### 2. Why Those State Machines Cannot Reach Invalid States

The `stopped` field is an `AtomicBool` which can only hold two values: `true` or `false`.

**Invariant:** `stopped` is always a valid boolean value.

Analysis:
- `AtomicBool::new(false)` initializes to valid state `false`
- `store(true, SeqCst)` transitions to valid state `true`
- `load(SeqCst)` reads a valid boolean value
- No arithmetic operations that could overflow/underflow
- No data structure manipulation

### 3. What Guarantees the Contract/Tests Provide

Contract guarantees:
- `send()` checks `stopped` before proceeding
- `stop()` transitions `stopped` from `false` to `true`
- No other transitions are possible

Test coverage:
- `heartbeat_sender_stop_is_idempotent` — verifies multiple `stop()` calls are safe
- `heartbeat_sender_clone_points_to_same_stopped_state` — verifies shared state semantics
- `heartbeat_max_progress_bytes_constant_is_1kb` — verifies constant value

### 4. Formal Reasoning (Not Hand-waving)

**Lemma 1:** The set of possible states for `HeartbeatSender` is `{Active, Stopped}`.

**Proof:** 
- Initial state after `new()`: `stopped = false` (Active)
- Only transition: `stop()` sets `stopped = true` (Stopped)
- No transition exists from Stopped back to Active
- Therefore, exactly two states exist.

**Lemma 2:** `send()` is safe for all states.

**Proof:**
- In Active state: `stopped.load()` returns `false`, proceeds to send
- In Stopped state: `stopped.load()` returns `true`, returns `Err(HeartbeatStopped)`
- Both branches return a defined result; no undefined behavior.

**Theorem:** `HeartbeatSender` cannot reach an invalid state that causes undefined behavior.

**Proof:** By Lemma 1, only two states exist. By Lemma 2, `send()` handles both states correctly. Therefore, no undefined behavior is reachable.

## Conclusion

Kani model checking is not required because:
1. The only state variable (`AtomicBool`) cannot represent invalid states
2. All operations on `AtomicBool` are defined for all values
3. The contract/tests verify the state machine behavior
4. No memory-unsafe operations are performed

**Recommendation:** Skip Kani. Proceed to State 7 (Architectural Drift).
