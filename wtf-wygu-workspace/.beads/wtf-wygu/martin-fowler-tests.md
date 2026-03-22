bead_id: wtf-wygu
bead_title: Long-running activity heartbeat mechanism
phase: test-plan
updated_at: 2026-03-22T00:30:00Z

# Martin Fowler Test Plan

## Happy Path Tests
- `test_send_heartbeat_returns_ok_with_sequence_number`
- `test_heartbeat_sender_send_returns_ok`
- `test_multiple_heartbeats_returns_increasing_sequence_numbers`
- `test_stop_allows_subsequent_stop_calls` (idempotency)

## Error Path Tests
- `test_send_heartbeat_with_empty_activity_id_returns_error`
- `test_send_heartbeat_with_oversized_progress_returns_invalid_input_error`
- `test_heartbeat_send_after_stop_returns_heartbeat_stopped_error`

## Edge Case Tests
- `test_empty_progress_string_is_allowed`
- `test_exactly_1kb_progress_string_is_allowed`
- `test_heartbeat_sender_clone_points_to_same_state`
- `test_stop_called_multiple_times_is_safe` (idempotency)

## Contract Verification Tests
- `test_heartbeat_event_variant_exists_in_workflow_event`
- `test_heartbeat_event_roundtrips_msgpack`
- `test_heartbeat_event_serde_json_tag_is_snake_case`

## Contract Violation Tests
- `test_activity_id_empty_violation_returns_invalid_input`
  Given: `ActivityId::new("")` (empty string)
  When: `send_heartbeat()` is called with empty activity ID
  Then: returns `Err(WtfError::InvalidInput)` — NOT a panic

- `test_progress_exceeds_1kb_violation_returns_invalid_input`
  Given: `progress = "x".repeat(2000)` (2000 bytes, exceeds 1KB limit)
  When: `send_heartbeat()` is called with oversized progress
  Then: returns `Err(WtfError::InvalidInput)` — NOT a panic

- `test_send_after_stop_violation_returns_heartbeat_stopped`
  Given: `HeartbeatSender` with `stop()` already called
  When: `send()` is called on the stopped handle
  Then: returns `Err(WtfError::HeartbeatStopped)` — NOT a panic

## Given-When-Then Scenarios

### Scenario 1: Successful heartbeat emission
Given: A valid JetStream context, namespace "payments", instance "inst-001", activity "act-001"
When: `send_heartbeat()` is called with progress "Processing phase 1"
Then:
- Returns `Ok(seq)` where seq > 0
- The event is appendable to JetStream
- No error is returned

### Scenario 2: Progress string too large
Given: A valid JetStream context and a 2KB progress string
When: `send_heartbeat()` is called
Then:
- Returns `Err(WtfError::InvalidInput)`
- No event is appended to JetStream

### Scenario 3: Sending after stop
Given: A `HeartbeatSender` where `stop()` was called
When: `send()` is called on the stopped sender
Then:
- Returns `Err(WtfError::HeartbeatStopped)`
- No event is appended to JetStream

### Scenario 4: HeartbeatSender cloning
Given: A `HeartbeatSender` instance
When: The handle is cloned using `Clone`
Then:
- Both handles can send heartbeats independently
- Stopping one does not prevent the other from sending
- Both point to the same underlying activity heartbeat state

## End-to-End Scenario

### Scenario: Activity handler sends heartbeats during long execution
Given: A worker processing a long-running "process_file" activity with 1MB file
When: The activity handler:
  1. Calls `heartbeat.send("Reading file header").await`
  2. Calls `heartbeat.send("Processing chunk 1/100").await`
  3. Processes the chunk
  4. Calls `heartbeat.send("Processing chunk 2/100").await`
  5. ... continues for all chunks
  6. Calls `heartbeat.send("Complete").await`
Then:
- All heartbeat events are appended to JetStream with increasing sequence numbers
- The activity can complete successfully via `complete_activity()`
- All heartbeats are visible in the event log for monitoring
