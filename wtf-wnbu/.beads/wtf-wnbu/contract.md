# Contract Specification: wtf-wnbu

## Context

- **Feature:** Replace `std::sync::Mutex` with `tokio::sync::Mutex` in `acquire_in_flight_guard()` to prevent async deadlock
- **Domain terms:**
  - `in_flight` - a `HashSet<String>` tracking instance IDs currently undergoing heartbeat-expired recovery
  - `in_flight_key` - the `instance_id.to_string()` used as the set key
  - `MutexGuard` / `AsyncMutexGuard` - guard types for exclusive access to the set
- **Assumptions:**
  - The `OnceLock` pattern for lazy-initialized static storage is preserved
  - Poison recovery via `into_inner()` is maintained for safety
- **Open questions:**
  - None

## Preconditions

- [x] `acquire_in_flight_guard()` may be called from any async context
- [x] The function must not block the Tokio runtime thread

## Postconditions

- [x] `acquire_in_flight_guard()` returns `tokio::sync::MutexGuard<'static, HashSet<String>>`
- [x] All call sites use `.await` when acquiring the lock
- [x] The `in_flight` set correctly tracks instance IDs in recovery
- [x] Poison recovery via `into_inner()` is preserved

## Invariants

- [x] No `std::sync::MutexGuard` is ever held across an `.await` point
- [x] Lock acquisition in async code always yields to the executor (via `.await`)
- [x] The `in_flight` set is always cleaned up (keys removed) even on recovery failure

## Error Taxonomy

This fix addresses a **correctness bug** (async deadlock) rather than a runtime error, so traditional error variants do not apply. However:

- `PoisonError` (via `into_inner()`) - mutex poisoned by panicked accessor; guard recovered to prevent key leaks
- Logic errors (if any):
  - **Invariant violation** - holding any mutex guard across `.await` is a programming error
  - **Key leak** - failure to remove `in_flight_key` after recovery completes

## Contract Signatures

### Function 1: `acquire_in_flight_guard()`

```rust
// BEFORE (blocking, incorrect in async context):
fn acquire_in_flight_guard() -> std::sync::MutexGuard<'static, HashSet<String>>

// AFTER (non-blocking, async-safe):
async fn acquire_in_flight_guard() -> tokio::sync::MutexGuard<'static, HashSet<String>>
```

**Preconditions:**
- May be called from any async context

**Postconditions:**
- Returns a `tokio::sync::MutexGuard` providing exclusive access to the static `HashSet<String>`
- If the mutex is poisoned (from a previous panic), recovers the inner value via `into_inner()` and returns a valid guard
- Lock acquisition yields to the executor (does not block the thread)

**Call sites to update:**

```rust
// Line 33: check_recovery_preconditions (synchronous, no await context)
let mut guard = acquire_in_flight_guard(); // NOT .await

// Lines 50, 65: attempt_recovery (async function, after .await points)
acquire_in_flight_guard().remove(in_flight_key);  // CHANGED TO: acquire_in_flight_guard().await.remove(in_flight_key);
```

---

### Function 2: `check_recovery_preconditions(state, instance_id) -> Option<String>`

```rust
fn check_recovery_preconditions(
    state: &OrchestratorState,
    instance_id: &InstanceId,
) -> Option<String>
```

**Preconditions:**
- `state` is a valid `OrchestratorState` with an `active` HashMap
- `instance_id` is a valid `InstanceId`

**Postconditions:**
- Returns `Some(in_flight_key)` where `in_flight_key = instance_id.to_string()` **if and only if**:
  1. `state.active` does NOT contain `instance_id` (instance is not currently active)
  2. The `in_flight` set does NOT already contain `in_flight_key` (no recovery in progress)
- Returns `None` if either condition fails
- **Condition order is guaranteed**: `active.contains_key` is checked BEFORE attempting to insert into `in_flight`
- When returning `None` due to `active.contains_key`, the `in_flight` set is NOT modified
- When returning `None` due to duplicate in-flight, the `in_flight` set is NOT modified (insert returns false)

**Error cases:**
- Returns `None` when `state.active.contains_key(instance_id)` is true — instance still alive
- Returns `None` when `in_flight_key` already exists in the `in_flight` set — recovery already in progress

---

### Function 3: `attempt_recovery(myself, state, instance_id, in_flight_key) -> ()`

```rust
async fn attempt_recovery(
    myself: &ActorRef<OrchestratorMsg>,
    state: &mut OrchestratorState,
    instance_id: &InstanceId,
    in_flight_key: &str,
) -> ()
```

**Preconditions:**
- `in_flight_key` is already present in the `in_flight` set
- `instance_id` is NOT in `state.active`

**Postconditions:**
- If `fetch_metadata` returns `Some(metadata)`:
  - Builds recovery arguments from metadata
  - Calls `WorkflowInstance::spawn_linked` to resurrect the instance
  - If spawn succeeds: registers the new actor ref in `state.active`
  - If spawn fails: does NOT register (instance remains unrecovered)
- **Always removes `in_flight_key` from the `in_flight` set**, regardless of:
  - Whether metadata was found
  - Whether spawn succeeded or failed
- Key removal happens AFTER metadata fetch and spawn attempt (not instead of)

**Key cleanup guarantee:** `in_flight_key` is removed from the set even when:
- `fetch_metadata` returns `None`
- `WorkflowInstance::spawn_linked` returns an error
- Any other error occurs during recovery

---

### Function 4: `handle_heartbeat_expired(myself, state, instance_id) -> ()`

```rust
pub async fn handle_heartbeat_expired(
    myself: ActorRef<OrchestratorMsg>,
    state: &mut OrchestratorState,
    instance_id: InstanceId,
) -> ()
```

**Preconditions:**
- `myself` is a valid `ActorRef<OrchestratorMsg>`
- `state` is a valid `OrchestratorState`
- `instance_id` is a valid `InstanceId`

**Postconditions:**
- Calls `check_recovery_preconditions(state, &instance_id)` **synchronously** (no `.await`)
- If `check_recovery_preconditions` returns `Some(in_flight_key)`:
  - Calls `attempt_recovery(&myself, state, &instance_id, &in_flight_key).await`
- If `check_recovery_preconditions` returns `None`:
  - Returns immediately without calling `attempt_recovery`
- The `in_flight_key` is cleaned up by `attempt_recovery` before it returns

**Guarantee:** No `.await` point exists between acquiring the in-flight guard and inserting the key.

---

### Function 5: `instance_id_from_heartbeat_key(key: &str) -> Option<InstanceId>`

```rust
#[must_use]
pub fn instance_id_from_heartbeat_key(key: &str) -> Option<InstanceId>
```

**Location:** `wtf-actor/src/heartbeat.rs`

**Preconditions:**
- `key` is any `&str`

**Postconditions:**
- Returns `Some(InstanceId)` if `key` matches the format `hb/<instance_id>` where:
  - `instance_id` is non-empty
  - `instance_id` contains no forward slash `/`
- Returns `None` for any other format including:
  - Empty string
  - `hb/` (empty id after prefix)
  - `instance/<id>` (wrong prefix)
  - `hb/<id>/<extra>` (extra segment)
  - Any other format

**Invariant:** The function is pure (no side effects, deterministic).

---

## Non-goals

- [x] Changing the `OnceLock` pattern for lazy initialization
- [x] Modifying the poison recovery behavior
- [x] Adding new functionality beyond fixing the async deadlock

(End of file — total 201 lines)
