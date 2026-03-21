# Martin Fowler Test Plan: wtf-worker Integration Tests with Live NATS

## Happy Path Tests

### test_work_queue_consumer_create_succeeds_with_valid_nats_context
Given: Live NATS server with JetStream, `wtf-work` stream exists
When: `WorkQueueConsumer::create(&js, "test-worker", None)` is called
Then: Returns `Ok(WorkQueueConsumer)`, consumer is durable

### test_work_queue_consumer_create_with_filter_subject
Given: Live NATS server with JetStream, `wtf-work` stream exists
When: `WorkQueueConsumer::create(&js, "email-worker", Some("wtf.work.send_email".into()))` is called
Then: Returns `Ok(WorkQueueConsumer)`, consumer filters to `wtf.work.send_email` only

### test_next_task_returns_task_when_message_available
Given: `WorkQueueConsumer` created, `enqueue_activity` published a task
When: `next_task()` is called
Then: Returns `Some(AckableTask { task: ActivityTask { activity_type, ... }, message })`

### test_next_task_returns_none_when_stream_closed
Given: `WorkQueueConsumer` created, stream is closed (engine shutdown)
When: `next_task()` is called
Then: Returns `Ok(None)`

### test_ack_removes_message_from_queue
Given: `WorkQueueConsumer` created, task pulled via `next_task`
When: `ackable.ack()` is called after `complete_activity` succeeds
Then: Message is removed from NATS queue, subsequent `next_task` does not return that message

### test_worker_run_processes_task_and_acks
Given: `Worker::new` with NATS context, handler registered for "send_email", task enqueued
When: `worker.run(shutdown_rx)` processes the task
Then: Handler called with task, `complete_activity` appended to JetStream, message acked

### test_enqueue_activity_publishes_to_correct_subject
Given: `ActivityTask` with `activity_type: "charge_card"`
When: `enqueue_activity(&js, &task)` is called
Then: Publishes to subject `wtf.work.charge_card`, returns `Ok(sequence)`

## Error Path Tests

### test_create_returns_error_when_stream_not_found
Given: NATS server running but `wtf-work` stream does not exist
When: `WorkQueueConsumer::create(&js, "worker", None)` is called
Then: Returns `Err(WtfError::NatsPublish(...))` with stream-not-found message

### test_create_returns_error_when_nats_disconnected
Given: Invalid/disconnected NATS context
When: `WorkQueueConsumer::create(&js, "worker", None)` is called
Then: Returns `Err(WtfError::NatsPublish(...))`

### test_next_task_returns_error_on_receive_failure
Given: `WorkQueueConsumer` created, NATS connection drops
When: `next_task()` is called
Then: Returns `Err(WtfError::NatsPublish(...))`

### test_ack_returns_error_on_delivery_failure
Given: `AckableTask` pulled, NATS connection lost before ack
When: `ackable.ack()` is called
Then: Returns `Err(WtfError::NatsPublish(...))`

### test_nak_requeues_message_for_redelivery
Given: `AckableTask` pulled, activity handler returns error
When: `ackable.nak()` is called
Then: Message is re-delivered to worker on next `next_task` call

### test_worker_calls_fail_activity_on_handler_error
Given: `Worker::new` with NATS context, handler registered that returns `Err`
When: `worker.run(shutdown_rx)` processes failing task
Then: `fail_activity` appended to JetStream, message acked

### test_worker_naks_on_append_event_failure
Given: `Worker::new` with NATS context, handler succeeds but `complete_activity` fails
When: `worker.run(shutdown_rx)` processes task
Then: `nak()` called on message, not `ack()`

### test_unknown_activity_type_logs_warning_and_acks
Given: `Worker::new` with no handlers registered, task with unknown activity type
When: `worker.run(shutdown_rx)` processes task
Then: Logs warning, acks message (avoid poison-pill queue stall)

## Edge Case Tests

### test_next_task_handles_empty_queue_gracefully
Given: `WorkQueueConsumer` created, no messages in queue
When: `next_task()` is called with timeout
Then: Returns `None` or times out gracefully (not an error)

### test_worker_respects_shutdown_signal
Given: `Worker::run` is active, shutdown signal sent
When: `shutdown_rx` receives `true`
Then: Worker stops processing, exits loop cleanly

### test_multiple_workers_share_queue_correctly
Given: Two `WorkQueueConsumer` instances with different `worker_name`
When: Tasks are enqueued
Then: Each worker receives its own durable cursor, no message lost or duplicated

### test_activity_task_msgpack_roundtrip_preserves_all_fields
Given: `ActivityTask` with all fields populated
When: Serialized via `to_msgpack()` and deserialized via `from_msgpack()`
Then: All fields identical (activity_id, activity_type, payload, namespace, instance_id, attempt, retry_policy)

### test_retry_policy_passed_correctly_to_handler
Given: `ActivityTask` with specific `retry_policy`
When: Handler is called
Then: Handler receives task with same `retry_policy` values

## Contract Verification Tests

### test_precondition_p3_stream_exists_verified_by_create
Given: Live NATS without `wtf-work` stream
When: `WorkQueueConsumer::create` called
Then: Returns error (precondition P3 violated → error returned)

### test_postcondition_q5_ack_removes_message
Given: Task pulled and acked
When: Different worker calls `next_task`
Then: Does not receive the acked message (Q5 verified)

### test_postcondition_q9_write_ahead_sequence_verified
Given: Task being processed by worker
When: `worker.run` calls `complete_activity` then `ack`
Then: JetStream append completes BEFORE ack is sent (verified via NATS metadata)

### test_invariant_i1_no_ack_before_append
Given: Worker processing task, `complete_activity` is async
When: Append is in flight but not yet acknowledged
Then: `ack()` is NOT called until `append_event` returns `Ok`

### test_invariant_i3_attempt_is_1_based
Given: `ActivityTask` with `attempt: 1`
When: Received via `next_task`
Then: `attempt == 1` means first attempt (not zero-based index)

## Contract Violation Tests

### test_violates_p2_empty_worker_name_returns_error
Given: Valid NATS context, `wtf-work` stream exists
When: `WorkQueueConsumer::create(&js, "", None)` is called
Then: Returns `Err(WtfError::NatsPublish("invalid consumer name"))` — NOT panic

### test_violates_q9_activity_completed_before_ack_not_allowed
Given: Worker with slow NATS
When: `complete_activity` returns but `PublishAck` not yet received
Then: `ack()` is NOT called yet — violation if ack were sent early

### test_violates_i1_ack_before_append_is_contract_violation
Given: Incorrect implementation that acks before appending event
When: Crash occurs between ack and append
Then: Event is LOST — this violates ADR-015 write-ahead guarantee

## End-to-End Scenario Tests

### test_full_dispatch_cycle_engine_to_worker_to_completion
Given: Engine dispatches `ActivityTask` via `enqueue_activity`, worker running
When: Complete cycle: enqueue → `next_task` → handler → `complete_activity` → `ack`
Then: Event appears in JetStream log, message removed from queue, workflow can proceed

### test_full_dispatch_cycle_with_failure_and_retry
Given: Engine dispatches `ActivityTask`, handler fails, retries available
When: `fail_activity` called with `retries_exhausted: false`, message nak'd
Then: Message re-delivered, handler retried, eventually `complete_activity` called

### test_write_ahead_guarantee_verified_under_crash_scenario
Given: Worker processing task, `complete_activity` has not yet returned
When: Worker process crashes (kill -9)
Then: Message is NOT acked, will be redelivered on restart (event was never appended)
This verifies ADR-015: no event loss between crash and append
