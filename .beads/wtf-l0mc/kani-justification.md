# Kani Justification: wtf-l0mc — Foundation Integration Tests

## Bead Overview
**Bead ID:** wtf-l0mc  
**Title:** Foundation integration tests (wtf-common + wtf-storage)  
**Phase:** State 5.7 (Kani Model Checking)

## Kani Analysis

### Option B: Formal Argument to Skip Kani

This bead (`wtf-l0mc`) contains **integration tests** for foundation components, not the core implementation itself.

### What This Bead Contains

- Integration test file: `crates/wtf-storage/tests/foundation_integration_tests.rs`
- Test functions that verify behavior of existing implementation
- No new state machines or complex state transitions

### Why Kani Is Not Needed

1. **No Critical State Machines in Tests**: Integration tests exercise the API surface but don't implement state machines themselves. The state machines exist in `wtf-actor`, not in these tests.

2. **State Machines Are in Parent Implementation**: The core state machine logic (FSM, DAG, Procedural paradigms) is implemented in `wtf-actor` crate, not in `wtf-storage` tests.

3. **Contract Tests Verify Behavior**: The tests verify:
   - ID validation (pure function, no state)
   - Serialization roundtrips (pure function)
   - Checksum validation (pure function)
   - Snapshot read/write (sled operations)
   - NATS connection (external I/O)

4. **Parent Epic Already Has Kani Coverage**: The parent epic `wtf-au94` (Phase 1 Foundation) would be the appropriate place for Kani verification of the core storage implementation.

### What State Machines Exist in wtf-storage

The `wtf-storage` crate contains:
- `append_event()` — Single function, no state machine
- `provision_streams()` — Idempotent setup, no state machine
- `provision_kv_buckets()` — Idempotent setup, no state machine
- `SnapshotRecord` — Simple struct with validation method, not a state machine
- `replay_start_seq()` — Pure calculation function

### Formal Reasoning

The integration tests verify **external behavior** of already-implemented components. They don't introduce new state machines that could reach invalid states. Kani verification should be applied to:
1. Core actor implementations (`wtf-actor`)
2. State machine transitions
3. Workflow instance lifecycle management

Not to integration tests that verify the API contract.

## Kani Justification Decision

**Status:** FORMAL JUSTIFICATION PROVIDED

Kani model checking is not required for this bead because it contains integration tests, not state machine implementations. The core implementation is in other beads and would be verified there if Kani is needed.
