# Martin Fowler Test Plan: Graceful Worker Shutdown (wtf-wuvv)

## Happy Path Tests
- test_drain_config_default_timeout_is_30_seconds
- test_drain_config_allow_custom_timeout
- test_worker_run_returns_shutdown_result_with_completed_count
- test_worker_run_reports_drain_duration_ms

## Error Path Tests
- test_drain_config_rejects_zero_duration
- test_drain_timeout_returns_error_when_exceeded

## Edge Case Tests
- test_graceful_shutdown_with_no_inflight_tasks
- test_graceful_shutdown_with_empty_queue
- test_graceful_shutdown_timeout_exactly_at_boundary

## Contract Verification Tests
- test_drain_begins_when_shutdown_signal_received
- test_no_new_tasks_fetched_after_drain_begins
- test_inflight_tasks_complete_during_drain
- test_drain_respects_timeout_duration

## Contract Violation Tests

### VIOLATION: Zero drain timeout
```
test_drain_config_zero_timeout_returns_error
  Given: DrainConfig::new(Duration::ZERO)
  When: constructor is called
  Then: returns Err(Error::InvalidDrainTimeout) — NOT Ok(DrainConfig)
```

### VIOLATION: Shutdown before any tasks
```
test_shutdown_with_no_inflight_tasks_reports_zero_completed
  Given: shutdown signal fires immediately, no tasks fetched
  When: run() completes drain phase
  Then: ShutdownResult.completed_count == 0
```

## Given-When-Then Scenarios

### Scenario 1: Graceful shutdown with in-flight tasks completing within timeout
Given: Worker has fetched 2 tasks, shutdown signal fires
When: Both tasks complete within drain_timeout
Then:
- completed_count == 2
- interrupted_count == 0
- drain_duration_ms < drain_timeout

### Scenario 2: Graceful shutdown with timeout expiring
Given: Worker has fetched 3 tasks, shutdown signal fires, tasks take longer than drain_timeout
When: drain_timeout expires after 30s
Then:
- interrupted tasks are nak'd back to queue
- completed_count == N (tasks completed before timeout)
- interrupted_count == 3 - N
- drain_duration_ms >= drain_timeout

### Scenario 3: Drain config validation
Given: A DrainConfig with Duration::ZERO
When: DrainConfig::new() is called
Then: Returns Err(Error::InvalidDrainTimeout)
And: Does NOT panic or unwrap

### Scenario 4: Queue closes during drain
Given: Worker is draining in-flight tasks, queue returns None (closed)
When: next_task() returns Ok(None) during drain
Then: Drain completes immediately with completed_count intact
And: interrupted_count reflects tasks not yet processed
