# Test Plan: wtf-wnbu — Async Mutex Conversion (`std::sync::Mutex` → `tokio::sync::Mutex`)

## Summary

- **Bead:** wtf-wnbu
- **Feature:** Replace `std::sync::Mutex` with `tokio::sync::Mutex` in `acquire_in_flight_guard()` to prevent async deadlock
- **Behaviors identified:** 9
- **Trophy allocation:** 25 unit / 4 integration / 0 e2e / 1 static
- **Proptest invariants:** 3
- **Fuzz targets:** 1
- **Kani harnesses:** 1

---

## 1. Behavior Inventory

| # | Subject | Action | Outcome | When |
|---|---------|--------|---------|------|
| 1 | `acquire_in_flight_guard()` | returns async mutex guard | guard provides exclusive access to `HashSet<String>` | always |
| 2 | `acquire_in_flight_guard()` | handles poison recovery | returns guard to inner value | when mutex is poisoned |
| 3 | `check_recovery_preconditions()` | checks active map first | returns `None` immediately if instance active | always |
| 4 | `check_recovery_preconditions()` | inserts in_flight_key | returns `Some(key)` | when instance not active and key not in set |
| 5 | `check_recovery_preconditions()` | skips duplicate recovery | returns `None` | when instance already in-flight |
| 6 | `check_recovery_preconditions()` | checks active BEFORE insert | active check prevents guard modification | always |
| 7 | `attempt_recovery()` | cleans up in_flight_key on metadata missing | removes key from set | when fetch_metadata returns None |
| 8 | `attempt_recovery()` | cleans up in_flight_key on spawn success/failure | removes key from set | always — even on spawn failure |
| 9 | `handle_heartbeat_expired()` | coordinates precondition check and recovery | correct key cleanup | always |

---

## 2. Trophy Allocation

| Layer | Count | Rationale |
|-------|-------|-----------|
| **Unit** | 25 | Pure functions with no I/O: `check_recovery_preconditions` (7 tests), `attempt_recovery` guard behavior (6 tests), `instance_id_from_heartbeat_key` parsing (12 tests including proptest). Exhaustive combinatorial coverage of all branches. |
| **Integration** | 4 | Full async workflow: `handle_heartbeat_expired` + `attempt_recovery` chain, duplicate trigger dedup, concurrent acquire/await/release pattern via existing `heartbeat_expiry_recovery.rs` integration tests |
| **E2E** | 0 | Covered by existing `heartbeat_expiry_recovery.rs` integration tests |
| **Static** | 1 | `clippy::await_holding_lock` (tokio rule) catches any accidentally held async mutex across `.await` |

**Rationale:** This is a correctness fix in async code. The primary risk is deadlock, which only manifests under concurrent execution. Integration tests exercise the real Tokio runtime. Unit tests cover pure logic paths. Test density is 25/5 = 5x for the 5 functions, meeting the required threshold.

---

## 3. BDD Scenarios

### Behavior 1: `acquire_in_flight_guard()` returns async mutex guard providing exclusive access

**Happy path:**
```gherkin
Given: the IN_FLIGHT OnceLock is uninitialized
When: acquire_in_flight_guard().await is called
Then: a tokio::sync::MutexGuard is returned providing exclusive access to the HashSet
And: the guard can be used to insert and remove keys
```

**Poison recovery:**
```gherkin
Given: a previous accessor panicked while holding the std::sync::Mutex (note: this path is obsolete after the fix, but preserved in the tokio version)
When: acquire_in_flight_guard().await is called on a poisoned tokio::sync::Mutex
Then: a guard to the inner HashSet is returned (tokio mutexes do not poison in the same way)
And: no panic occurs
```

**Guard drop releases immediately:**
```gherkin
Given: acquire_in_flight_guard().await has been called and a guard is held
When: the guard is dropped (explicitly or via scope end)
Then: the lock is released and another task can acquire it immediately
And: the executor is notified of lock availability
```

**Test names:**
- `fn acquire_in_flight_guard_returns_valid_mutex_guard_when_awaited()`
- `fn acquire_in_flight_guard_does_not_panic_on_poisoned_mutex()`
- `fn acquire_in_flight_guard_guard_drop_releases_to_executor_immediately()`

---

### Behavior 2: `check_recovery_preconditions` checks active map BEFORE inserting

**Active check first:**
```gherkin
Given: OrchestratorState where instance_id IS in the active map
When: check_recovery_preconditions(state, instance_id) is called synchronously
Then: None is returned immediately
And: the in_flight HashSet is NOT modified (no lock acquired)
```

**Test name:** `fn check_recovery_preconditions_returns_none_without_touching_in_flight_set_when_instance_active()`

---

### Behavior 3: `check_recovery_preconditions` inserts in_flight_key and returns Some when not in-flight

**Happy path:**
```gherkin
Given: OrchestratorState where instance_id is NOT in active map
And: the in_flight HashSet does NOT contain instance_id.to_string()
When: check_recovery_preconditions(state, instance_id) is called synchronously
Then: in_flight_key = instance_id.to_string() is returned
And: in_flight_key is inserted into the in_flight HashSet
```

**Deduplication:**
```gherkin
Given: OrchestratorState where instance_id is NOT in active map
But: the in_flight HashSet ALREADY contains instance_id.to_string()
When: check_recovery_preconditions(state, instance_id) is called
Then: None is returned
And: in_flight_key is NOT inserted (already present, insert returns false)
```

**Still active:**
```gherkin
Given: OrchestratorState where instance_id IS in the active map
When: check_recovery_preconditions(state, instance_id) is called
Then: None is returned immediately
And: in_flight HashSet is NOT modified
```

**Empty instance_id:**
```gherkin
Given: OrchestratorState where instance_id is InstanceId("")
And: instance_id is NOT in active map
And: in_flight HashSet does not contain ""
When: check_recovery_preconditions(state, instance_id) is called
Then: Some("") is returned
And: "" is inserted into the in_flight HashSet
```

**Instance ID with special characters:**
```gherkin
Given: OrchestratorState where instance_id is InstanceId("inst-001_test")
And: instance_id is NOT in active map
And: in_flight HashSet does not contain "inst-001_test"
When: check_recovery_preconditions(state, instance_id) is called
Then: Some("inst-001_test") is returned
And: "inst-001_test" is inserted into the in_flight HashSet
```

**Instance ID with forward slash (rejected by instance_id_from_heartbeat_key):**
```gherkin
Given: OrchestratorState where instance_id contains a forward slash InstanceId("in/valid")
And: instance_id is NOT in active map
And: in_flight HashSet does not contain "in/valid"
When: check_recovery_preconditions(state, instance_id) is called
Then: Some("in/valid") is returned
And: "in/valid" is inserted into the in_flight HashSet
Note: The slash check happens at the heartbeat key parsing layer, not here
```

**Test names:**
- `fn check_recovery_preconditions_returns_key_when_instance_not_active_and_not_in_flight()`
- `fn check_recovery_preconditions_returns_none_when_duplicate_recovery_in_flight()`
- `fn check_recovery_preconditions_returns_none_when_instance_still_active()`
- `fn check_recovery_preconditions_returns_key_for_empty_instance_id()`
- `fn check_recovery_preconditions_returns_key_for_instance_id_with_special_characters()`
- `fn check_recovery_preconditions_returns_key_for_instance_id_with_slash()`

---

### Behavior 4: `check_recovery_preconditions` condition order is enforced (anti-mutation)

**Condition order anti-mutation:**
```gherkin
Given: OrchestratorState where instance_id is in active map AND not in in_flight set
When: check_recovery_preconditions(state, instance_id) is called
Then: None is returned
And: the in_flight HashSet remains empty (the active check prevents any guard modification)
```

**Test name:** `fn check_recovery_preconditions_checks_active_before_insert_to_prevent_guard_modification()`

---

### Behavior 5: `attempt_recovery` cleans up in_flight_key when metadata fetch fails

**Metadata missing:**
```gherkin
Given: in_flight_key is in the in_flight HashSet
And: state.state_store.get_instance_metadata returns None or error
When: attempt_recovery is called and await fetch_metadata returns None
Then: acquire_in_flight_guard().await.remove(in_flight_key) is called
And: in_flight_key is removed from the HashSet
```

**Metadata fetch error:**
```gherkin
Given: in_flight_key is in the in_flight HashSet
And: state.state_store.get_instance_metadata returns an error
When: attempt_recovery is called and await fetch_metadata returns Err
Then: in_flight_key is removed from the HashSet
And: no spawn attempt is made
```

**Test names:**
- `fn attempt_recovery_removes_in_flight_key_when_metadata_missing()`
- `fn attempt_recovery_removes_in_flight_key_when_fetch_metadata_returns_error()`

---

### Behavior 6: `attempt_recovery` cleans up in_flight_key regardless of spawn outcome

**Spawn success:**
```gherkin
Given: in_flight_key is in the in_flight HashSet
And: fetch_metadata returns Some valid metadata
When: attempt_recovery is called and WorkflowInstance::spawn_linked succeeds
Then: instance is registered in state
And: in_flight_key is removed from the HashSet AFTER registration
```

**Spawn failure:**
```gherkin
Given: in_flight_key is in the in_flight HashSet
And: fetch_metadata returns Some valid metadata
When: attempt_recovery is called and WorkflowInstance::spawn_linked fails
Then: in_flight_key is still removed from the HashSet (cleanup always happens)
```

**Key cleanup ordering (anti-mutation):**
```gherkin
Given: in_flight_key is in the in_flight HashSet
And: fetch_metadata returns Some valid metadata
When: attempt_recovery is called and spawn succeeds
Then: instance is registered in state.active
And: THEN in_flight_key is removed (not before, not instead of)
And: the key is absent from in_flight set when function returns
```

**Spawn error after metadata fetch:**
```gherkin
Given: in_flight_key is in the in_flight HashSet
And: fetch_metadata returns Some valid metadata
When: attempt_recovery is called and WorkflowInstance::spawn_linked returns an error
Then: instance is NOT registered in state.active
And: in_flight_key is still removed from the HashSet
```

**Test names:**
- `fn attempt_recovery_removes_in_flight_key_on_spawn_success()`
- `fn attempt_recovery_removes_in_flight_key_on_spawn_failure()`
- `fn attempt_recovery_removes_key_after_spawn_not_before()`
- `fn attempt_recovery_does_not_register_on_spawn_error_after_metadata_fetch()`

---

### Behavior 7: `handle_heartbeat_expired` coordinates precondition check and recovery

**Full flow — new recovery:**
```gherkin
Given: OrchestratorState with instance not in active map
When: handle_heartbeat_expired is called with that instance_id
Then: check_recovery_preconditions returns Some(in_flight_key)
And: attempt_recovery is awaited
And: in_flight_key is cleaned up after attempt_recovery completes
```

**Full flow — skipped:**
```gherkin
Given: OrchestratorState with instance in active map
When: handle_heartbeat_expired is called
Then: check_recovery_preconditions returns None
And: attempt_recovery is NOT called
```

**Guard dropped before await:**
```gherkin
Given: OrchestratorState with instance not in active map
When: handle_heartbeat_expired is called
Then: The guard acquired inside check_recovery_preconditions is dropped before attempting recovery
And: attempt_recovery is awaited after the guard scope ends
```

**Test names:**
- `fn handle_heartbeat_expired_triggers_recovery_when_prerequisites_met()`
- `fn handle_heartbeat_expired_skips_recovery_when_instance_still_active()`
- `fn handle_heartbeat_expired_drops_guard_before_awaiting_attempt_recovery()`

---

## 4. Proptest Invariants

### Proptest 1: Async Mutex Serialization

**Invariant:** When multiple tasks concurrently call `acquire_in_flight_guard().await`, they are serialized — only one holds the guard at a time.

**Strategy:**
```rust
// Generate arbitrary number of concurrent tasks (1..16)
let task_count = any::<u8>(1..=16).map(|n| n as usize);
// Each task inserts a unique key and yields
// Assert: all keys are present in set after all tasks complete
// Assert: no duplicate keys (set semantics guarantee this)
```

**Anti-invariant:** Input class where `task_count = 0` is handled gracefully (returns immediately).

---

### Proptest 2: Guard Drop Releases Lock

**Invariant:** When a `MutexGuard` is dropped (explicitly or via scope end), the lock is released and another task can acquire it.

**Strategy:**
```rust
// Acquire guard, insert key, drop guard explicitly
// Immediately acquire again — must succeed
// Key must still be present (wasn't removed)
```

---

### Proptest 3: `instance_id_from_heartbeat_key` parsing invariant

**Invariant:** For any string `s` that does not start with `"hb/"`, the function returns `None`.

**Strategy:**
```rust
// Generate arbitrary strings that do NOT start with "hb/"
// Examples: "instance/01ARZ", "heartbeat/01ARZ", "hb", "", "h b/01ARZ"
prop_assert_eq!(instance_id_from_heartbeat_key(s), None);
```

**Invariant 2:** For any string `s` that starts with `"hb/"` followed by a non-empty segment with no `/`, the function returns `Some(InstanceId)`.

**Strategy:**
```rust
// Generate strings matching "hb/[a-zA-Z0-9_-]+" with length 3..256
// Examples: "hb/01ARZ", "hb/inst-001", "hb/order_flow_01ARZ"
let result = instance_id_from_heartbeat_key(s);
prop_assert!(result.is_some());
```

**Invariant 3 (anti-invariant):** For any string `s` that starts with `"hb/"` followed by empty string or containing `/`, the function returns `None`.

**Strategy:**
```rust
// Generate: "hb/", "hb//", "hb/01ARZ/extra", "hb/01ARZ/"
prop_assert_eq!(instance_id_from_heartbeat_key(s), None);
```

---

## 5. Fuzz Targets

### Fuzz Target: `instance_id_from_heartbeat_key` parsing

**Input type:** `&str` (arbitrary string)

**Risk class:** Panic / logic error — malformed keys could cause incorrect routing or loss of recovery signals.

**Corpus seeds:**
- `"hb/01ARZ3NDEKTSV4RRFFQ69G5FAV"` (valid)
- `"hb/"` (empty id — must return None)
- `"hb/inst-001"` (short id with hyphen)
- `"instance/01ARZ"` (wrong prefix)
- `""` (empty string)
- `"hb/01ARZ/extra"` (extra slash segment)
- very long strings for memory/allocation testing

**Rationale:** This function is in the hot path for heartbeat expiry detection. A panic here would crash the watcher.

---

## 6. Kani Harnesses

### Kani Harness: Async Mutex No-Deadlock Property

**Property:** For any sequence of `acquire_in_flight_guard().await` calls (up to bound N=4 concurrent tasks), the program does not deadlock.

**Bound:** N=4 concurrent task acquisitions.

**Rationale:** The core bug was holding `std::sync::MutexGuard` across an `.await` point, causing deadlock when the executor needed that thread to make progress. Formal verification proves the tokio mutex (which yields to executor on contention) cannot deadlock in the same way. This is critical because deadlock bugs are notoriously hard to reproduce in tests.

**Proof obligations:**
1. `acquire_in_flight_guard()` is `async fn` returning `tokio::sync::MutexGuard`
2. Every call site uses `.await` before any `.await` on other operations
3. Guard lifetime ends before any other `.await` in same function

---

## 7. Mutation Checkpoints

| Mutation | Catch Mechanism | Test |
|---------|-----------------|------|
| `.await` removed from line 50 | Integration test + Kani | `fn attempt_recovery_removes_in_flight_key_when_metadata_missing` |
| `.await` removed from line 65 | Integration test + Kani | `fn attempt_recovery_removes_in_flight_key_on_spawn_success` |
| `acquire_in_flight_guard()` made sync (return type changed to `MutexGuard` not `tokio::...`) | Static analysis: `clippy::await_holding_lock` | N/A — compile error after fix |
| `std::sync::Mutex` used instead of `tokio::sync::Mutex` | Static: `clippy::await_holding_lock` | N/A — compile error after fix |
| `remove(in_flight_key)` call removed | Integration: dedup test would see leaked keys | `fn duplicate_heartbeat_expired_triggers_single_recovery` |
| Condition order swapped in `check_recovery_preconditions` | Unit: `fn check_recovery_preconditions_checks_active_before_insert_to_prevent_guard_modification` | Detects when active check is not first |
| `check_recovery_preconditions` active check removed entirely | Unit: `fn check_recovery_preconditions_returns_none_without_touching_in_flight_set_when_instance_active` | Detects guard modification for active instance |
| Spawn registers even on error | Unit: `fn attempt_recovery_does_not_register_on_spawn_error_after_metadata_fetch` | Detects incorrect registration |

**Threshold:** ≥90% mutation kill rate target.

---

## 8. Combinatorial Coverage Matrix

### Unit Test Group: `check_recovery_preconditions`

| Scenario | Input | Expected Output | Layer |
|----------|-------|-----------------|-------|
| happy path | state with inactive instance, empty in_flight | `Some(in_flight_key)` | unit |
| still active | state with active instance | `None` + no guard modification | unit |
| duplicate in-flight | in_flight set already contains key | `None` | unit |
| empty instance_id | InstanceId::new("") | `Some("")` | unit |
| condition order anti-mutation | active instance + not in-flight | `None` + in_flight unchanged | unit |
| special characters | InstanceId("inst-001_test") | `Some("inst-001_test")` | unit |
| slash in instance_id | InstanceId("in/valid") | `Some("in/valid")` | unit |

### Unit Test Group: `attempt_recovery`

| Scenario | Input | Expected Output | Layer |
|----------|-------|-----------------|-------|
| metadata missing | fetch returns None | key removed from in_flight | unit |
| metadata error | fetch returns Err | key removed, no spawn attempted | unit |
| spawn success | fetch returns Some, spawn succeeds | key removed after registration | unit |
| spawn failure | fetch returns Some, spawn fails | key removed after spawn attempt | unit |
| key cleanup ordering | spawn success | key removed AFTER spawn, not before | unit |
| no register on spawn error | fetch returns Some, spawn returns Err | key removed, state unchanged | unit |

### Unit Test Group: `instance_id_from_heartbeat_key`

| Scenario | Input | Expected Output | Layer |
|----------|-------|-----------------|-------|
| valid key | `"hb/01ARZ"` | `Some(InstanceId)` | unit |
| empty id | `"hb/"` | `None` | unit |
| missing prefix | `"instance/01ARZ"` | `None` | unit |
| empty string | `""` | `None` | unit |
| extra segment | `"hb/01ARZ/extra"` | `None` | unit |
| valid with underscore | `"hb/order_flow_01ARZ"` | `Some("order_flow_01ARZ")` | unit |
| valid with hyphen | `"hb/inst-001"` | `Some("inst-001")` | unit |
| no prefix at all | `"hb"` | `None` | unit |
| space in prefix | `"hb /01ARZ"` | `None` | unit |
| unicode in key portion | `"hb/实例01ARZ"` | `None` | unit |
| control characters | `"hb/\x00test"` | `None` | unit |
| max length boundary | `"hb/" + "A".repeat(255)` | `Some(InstanceId)` | unit |
| proptest invariant | arbitrary non-hb strings | `None` | proptest |
| proptest invariant | arbitrary valid hb/* strings | `Some` | proptest |

### Unit Test Group: `acquire_in_flight_guard`

| Scenario | Input | Expected Output | Layer |
|----------|-------|-----------------|-------|
| happy path | uninitialized OnceLock | `MutexGuard` returned | unit |
| poison recovery | previous panic | guard to inner value | unit |
| guard drop releases | guard dropped | other task can acquire | unit |

### Unit Test Group: `handle_heartbeat_expired`

| Scenario | Input | Expected Output | Layer |
|----------|-------|-----------------|-------|
| full recovery flow | inactive instance with valid metadata | Live instance recovered | integration |
| recovery dedup | two concurrent HeartbeatExpired for same id | Single recovery spawned | integration |
| metadata missing cleanup | inactive instance, no metadata in store | Key removed, no spawn | integration |
| instance still active | active instance receives HeartbeatExpired | Recovery skipped | integration |
| guard dropped before await | precondition passes | guard scope ends before attempt_recovery | unit |

### Integration Test Group: `handle_heartbeat_expired`

| Scenario | Input | Expected Output | Layer |
|----------|-------|-----------------|-------|
| full recovery flow | inactive instance with valid metadata | Live instance recovered | integration |
| recovery dedup | two concurrent HeartbeatExpired for same id | Single recovery spawned | integration |
| metadata missing cleanup | inactive instance, no metadata in store | Key removed, no spawn | integration |
| instance still active | active instance receives HeartbeatExpired | Recovery skipped | integration |

---

## Open Questions

1. **Q:** Is there an existing `tokio::sync::MutexGuard` poisoning behavior we need to preserve, or is tokio's non-poisoning model acceptable?
   **A:** Tokio mutexes don't poison by default. The `into_inner()` pattern is for `std::sync::Mutex`. After the switch, the poison recovery path becomes unreachable but should be kept for API compatibility (it won't trigger but the match arm remains).

2. **Q:** Should `acquire_in_flight_guard()` remain `pub(crate)` or become `pub` for testing?
   **A:** Keep `pub(crate)` — tests can use the public `handle_heartbeat_expired` entry point.

3. **Q:** Are there existing proptest setups in wtf-actor for async invariant testing?
   **A:** None found. The proptest invariants in this plan require adding `proptest` and `tokio::test` infrastructure.

---

## Exit Criteria Verification

- [x] Every public API behavior has at least one BDD scenario (9 behaviors, all covered)
- [x] Every pure function with multiple inputs has at least one proptest invariant (3 invariants for 3 pure functions)
- [x] Every parsing/deserialization boundary has a fuzz target (`instance_id_from_heartbeat_key`)
- [x] Every error variant in the Error enum has an explicit test scenario (N/A — this is a correctness fix, not error-based)
- [x] The mutation threshold target (≥90%) is stated
- [x] No test asserts only `is_ok()` or `is_err()` without specifying the value
- [x] Contract defines all 5 functions being tested
- [x] Test density meets threshold (25 unit tests / 5 functions = 5x)

---

## Unit Test Count Summary

| Function | Unit Tests | New Tests Added |
|----------|------------|-----------------|
| `acquire_in_flight_guard()` | 3 | +1 (guard drop releases immediately) |
| `check_recovery_preconditions()` | 7 | +2 (special characters, slash in instance_id) |
| `attempt_recovery()` | 6 | +2 (fetch error, no register on spawn error) |
| `handle_heartbeat_expired()` | 3 | +1 (guard dropped before await) |
| `instance_id_from_heartbeat_key` | 12 | +3 (unicode, control chars, max length) |
| **Total** | **25** | **9** |

(End of file — total 580 lines)
