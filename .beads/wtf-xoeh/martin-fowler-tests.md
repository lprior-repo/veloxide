bead_id: wtf-xoeh
bead_title: CLI command: wtf admin rebuild-views
phase: 2
updated_at: 2026-03-21T23:59:30Z

# Martin Fowler Test Plan: wtf admin rebuild-views

## Test Strategy
Using Given-When-Then BDD format per Dave Farley's ATDD principles. Each scenario is an executable specification.

---

## Happy Path Tests

### test_rebuild_all_views_succeeds
**Given**: NATS JetStream is running with event streams and KV buckets provisioned
**When**: I run `wtf admin rebuild-views`
**Then**: Exit code is 0
**And**: All instances from JetStream appear in `wtf-instances`
**And**: All pending timers appear in `wtf-timers`
**And**: All definitions appear in `wtf-definitions`
**And**: Stats are printed to stdout (instances_rebuilt, timers_rebuilt, definitions_rebuilt, events_processed, duration_ms)

### test_rebuild_single_view_succeeds
**Given**: Multiple views exist with data
**When**: I run `wtf admin rebuild-views --view instances`
**Then**: Exit code is 0
**And**: Only `wtf-instances` bucket is rebuilt
**And**: Other buckets retain their existing data

### test_rebuild_with_namespace_filter
**Given**: Multiple namespaces have events in JetStream
**When**: I run `wtf admin rebuild-views --namespace payments`
**Then**: Exit code is 0
**And**: Only `payments` namespace instances are rebuilt
**And**: Other namespaces are unaffected

### test_progress_output_is_shown
**Given**: NATS JetStream has events to replay
**When**: I run `wtf admin rebuild-views`
**Then**: Progress is reported to stdout
**And**: Final stats summary is printed

---

## Error Path Tests

### test_returns_error_when_nats_not_running
**Given**: NATS server is not running on configured host:port
**When**: I run `wtf admin rebuild-views`
**Then**: Exit code is non-zero
**And**: Error message indicates connection failure
**And**: No partial state is written to KV buckets

### test_returns_error_for_invalid_view_name
**Given**: An invalid view name is provided
**When**: I run `wtf admin rebuild-views --view invalid_view`
**Then**: Exit code is non-zero
**And**: Error message lists valid view names (instances, timers, definitions, heartbeats)

### test_returns_error_when_stream_missing
**Given**: NATS is running but `wtf-events` stream does not exist
**When**: I run `wtf admin rebuild-views`
**Then**: Exit code is non-zero
**And**: Error message indicates stream not found

### test_returns_error_when_kv_bucket_missing
**Given**: NATS JetStream is running but KV bucket cannot be accessed
**When**: I run `wtf admin rebuild-views`
**Then**: Exit code is non-zero
**And**: Error message identifies the failing bucket

---

## Idempotency Tests

### test_rebuild_is_idempotent
**Given**: Views have been successfully rebuilt once
**When**: I run `wtf admin rebuild-views` again
**Then**: Exit code is 0
**And**: Same final state as first run
**And**: No duplicate or corrupted entries in KV buckets
**And**: Duration is similar (no exponential slowdown)

### test_rebuild_idempotent_with_concurrent_reads
**Given**: Views are being rebuilt
**When**: Concurrently reading from KV buckets during rebuild
**Then**: Readers see consistent state (no torn reads)
**And**: Rebuild completes successfully
**And**: Final state is correct

---

## Edge Case Tests

### test_handles_empty_stream_gracefully
**Given**: JetStream has no events (fresh deployment)
**When**: I run `wtf admin rebuild-views`
**Then**: Exit code is 0
**And**: Stats show 0 events processed
**And**: KV buckets are provisioned but empty

### test_handles_large_number_of_instances
**Given**: JetStream has 10,000+ instances across many namespaces
**When**: I run `wtf admin rebuild-views`
**Then**: Exit code is 0 within reasonable time (<5 minutes)
**And**: All instances are correctly rebuilt
**And**: Memory usage remains bounded (streaming replay)

### test_heartbeat_bucket_not_rebuilt
**Given**: `wtf-heartbeats` bucket has existing entries
**When**: I run `wtf admin rebuild-views`
**Then**: `wtf-heartbeats` entries remain unchanged
**And**: `wtf-heartbeats` is NOT rebuilt (ephemeral, 10s TTL)
**And**: Other three buckets (`wtf-instances`, `wtf-timers`, `wtf-definitions`) are rebuilt

### test_dry_run_does_not_modify_state
**Given**: KV buckets have existing data
**When**: I run `wtf admin rebuild-views --dry-run`
**Then**: Exit code is 0
**And**: A preview of what would be rebuilt is printed
**And**: KV buckets are unchanged (verified by snapshot comparison)

---

## Contract Verification Tests

### test_precondition_nats_connected
**Given**: No NATS connection
**When**: RebuildViews command is invoked
**Then**: Returns `Err(WtfError::NatsConnect(_))`

### test_postcondition_all_instances_rebuilt
**Given**: JetStream has instance events
**When**: Rebuild completes successfully
**Then**: Every instance has a corresponding `wtf-instances` entry

### test_invariant_kv_never_source_of_truth
**Given**: Rebuild completes successfully
**When**: We compare KV state to JetStream
**Then**: KV is derived from JetStream (KV is always subset/derived)

### test_no_data_loss_postcondition
**Given**: JetStream has N instances with events
**When**: Rebuild completes successfully
**Then**: Exactly N entries exist in `wtf-instances`
**And**: Every instance_id from JetStream has a corresponding KV entry
**And**: No instances are silently dropped

### test_exit_code_matches_return_value
**Given**: A rebuild scenario
**When**: Command completes with return value Ok
**Then**: Exit code is 0
**When**: Command completes with return value Err
**Then**: Exit code is non-zero
**And**: Exit code accurately reflects the error category

### test_concurrent_rebuild_returns_error_or_is_serialized
**Given**: A rebuild is in progress
**When**: A second rebuild is started concurrently
**Then**: Either second rebuild fails with error "rebuild already in progress"
**Or**: Second rebuild waits and completes after first finishes
**And**: No corrupted state in KV buckets in either case

---

## Given-When-Then Scenarios

### Scenario 1: Full rebuild with all views
**Given**: NATS JetStream with events from `payments` and `onboarding` namespaces
**And**: KV buckets are empty or stale
**When**: I run `wtf admin rebuild-views`
**Then**: All events are replayed in order
**And**: Final KV state matches JetStream event log
**And**: Stats report: X instances, Y timers, Z definitions

### Scenario 2: Partial rebuild for single view
**Given**: All four KV buckets have existing data
**And**: `wtf-instances` bucket is stale
**When**: I run `wtf admin rebuild-views --view instances`
**Then**: Only `wtf-instances` is rebuilt
**And**: `wtf-timers`, `wtf-definitions`, `wtf-heartbeats` are unchanged

### Scenario 3: Idempotent second run
**Given**: Rebuild has completed successfully once
**When**: I run `wtf admin rebuild-views` again
**Then**: Same results as first run
**And**: No new events processed (already in sync)
**And**: Exit code is 0

### Scenario 4: Error recovery
**Given**: NATS connection drops mid-rebuild
**When**: I run `wtf admin rebuild-views`
**Then**: Exit code is non-zero
**And**: Partial state is not committed (atomic per-instance replay)
**And**: Error message is descriptive
