# Martin Fowler Test Plan: FSM Crash-and-Replay Integration Test

bead_id: wtf-rakc
bead_title: integration test: FSM crash-and-replay — crash after JetStream ACK before KV write
phase: test-plan
updated_at: 2026-03-22T00:00:00Z

## Happy Path Tests

- test_fsm_crash_replay_success
  Given: A running FSM engine with NATS and Sled initialized
  When: I start a checkout workflow and advance to Authorized state, then crash after ACK but before KV write
  Then: After restart, the instance appears in KV with state == 'Authorized' and JetStream has exactly 2 events

## Error Path Tests

- test_nats_not_available_returns_error
  Given: NATS server is not running
  When: start_fsm_workflow("checkout") is called
  Then: returns Err(Error::NatsNotAvailable)

- test_instance_not_found_after_restart_returns_error
  Given: Engine crashed and restarted
  When: wait_for_instance_kv times out
  Then: returns Err(Error::InstanceNotFound)

- test_unexpected_event_count_returns_error
  Given: Engine crashed and restarted
  When: JetStream log has more or fewer than 2 events
  Then: returns Err(Error::UnexpectedEventCount)

- test_state_mismatch_returns_error
  Given: Engine crashed and restarted
  When: KV current_state is not 'Authorized'
  Then: returns Err(Error::StateMismatch)

- test_duplicate_transition_returns_error
  Given: Engine crashed and restarted
  When: JetStream log contains duplicate TransitionApplied
  Then: returns Err(Error::DuplicateTransition)

## Edge Case Tests

- test_handles_rapid_crash_restart_cycle
  Given: Engine is running
  When: Crash and restart happen within 100ms
  Then: Instance is recovered correctly

- test_handles_concurrent_kv_access
  Given: Multiple engine instances writing to KV
  When: Crash occurs on one instance
  Then: Other instances are unaffected

## Contract Verification Tests

- test_precondition_nats_available
  Given: NATS server is not running
  When: start_fsm_workflow is called
  Then: Returns Err(Error::NatsNotAvailable)

- test_precondition_kv_empty_at_start
  Given: KV has stale entries
  When: Test runs
  Then: Test cleans up or uses unique instance ID

- test_postcondition_instance_in_kv
  Given: Engine crashed and restarted
  When: wait_for_instance_kv completes
  Then: Instance exists in KV with correct state

- test_postcondition_jetstream_events_correct
  Given: Engine crashed and restarted
  When: JetStream log is read
  Then: Exactly 2 events exist with correct types

- test_invariant_no_duplicate_transitions
  Given: JetStream events are read
  When: Events are analyzed
  Then: No duplicate TransitionApplied events exist

## Contract Violation Tests

- test_violates_p1_nats_not_available
  Given: NATS server is not running
  When: start_fsm_workflow("checkout") is called
  Then: returns Err(Error::NatsNotAvailable)

- test_violates_q1_instance_not_found
  Given: Engine crashed and restarted but KV write never completed
  When: wait_for_instance_kv(instance_id, 5s) is called
  Then: returns Err(Error::InstanceNotFound)

- test_violates_q2_unexpected_event_count
  Given: Engine crashed and extra events were written to JetStream
  When: assert_event_count(events, 2) is called with 3 events
  Then: returns Err(Error::UnexpectedEventCount(3, 2))

- test_violates_q3_state_mismatch
  Given: Engine restored to wrong state
  When: assert_state_eq(actual, "Authorized") is called with actual = "Processing"
  Then: returns Err(Error::StateMismatch("Processing", "Authorized"))

- test_violates_q4_duplicate_transition
  Given: Two TransitionApplied events exist for same transition
  When: assert_no_duplicate_transitions(events) is called
  Then: returns Err(Error::DuplicateTransition)

## Given-When-Then Scenarios

### Scenario 1: FSM survives crash in crash window
Given: FSM checkout workflow is running, engine is in Authorized state, TransitionApplied event was ACKed to JetStream but KV write is pending
When: I send SIGSTOP to pause, then SIGKILL to kill the engine, then restart the engine
Then: 
- Instance appears in wtf-instances KV within 5 seconds
- current_state == 'Authorized'
- JetStream log shows exactly InstanceStarted and one TransitionApplied
- No duplicate TransitionApplied events exist

### Scenario 2: Engine replays correctly from JetStream
Given: Engine is restarting after crash
When: Engine reads JetStream log for instance
Then:
- Engine reconstructs exact state at time of crash
- No activities are re-dispatched
- Event log is append-only (no new events written)

### Scenario 3: Crash before any KV write
Given: FSM workflow started but no state transitions completed
When: Engine crashes before first KV write
Then:
- After restart, no instance appears in KV
- Test passes trivially (nothing to recover)
