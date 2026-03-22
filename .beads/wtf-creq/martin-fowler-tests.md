# Martin Fowler Test Plan: wtf admin rebuild-views

## Bead ID
wtf-creq

## Feature
`wtf admin rebuild-views` — disaster recovery tool to reconstruct NATS KV from JetStream

---

## Happy Path Tests

### test_run_rebuild_views_returns_success_exit_code
**Given**: Valid NATS connection and provisioned KV buckets  
**When**: `run_rebuild_views(RebuildViewsConfig { view: None, namespace: None, show_progress: false, dry_run: false })` is called  
**Then**: Returns `Ok(ExitCode::SUCCESS)`

### test_run_rebuild_views_with_namespace_filter
**Given**: NATS with events in "payments" namespace  
**When**: `run_rebuild_views(RebuildViewsConfig { namespace: Some("payments".into()), ... })` is called  
**Then**: Only instances with subject `wtf.log.payments.*` are processed

### test_run_rebuild_views_with_view_filter_instances
**Given**: NATS with mixed view data  
**When**: `run_rebuild_views(RebuildViewsConfig { view: Some("instances".into()), ... })` is called  
**Then**: Only `wtf-instances` KV bucket is written

### test_run_rebuild_views_with_view_filter_timers
**Given**: NATS with mixed view data  
**When**: `run_rebuild_views(RebuildViewsConfig { view: Some("timers".into()), ... })` is called  
**Then**: Only `wtf-timers` KV bucket is written

### test_run_rebuild_views_with_view_filter_definitions
**Given**: NATS with mixed view data  
**When**: `run_rebuild_views(RebuildViewsConfig { view: Some("definitions".into()), ... })` is called  
**Then**: Only `wtf-definitions` KV bucket is written

### test_run_rebuild_views_reports_correct_stats
**Given**: JetStream with 3 instances, 10 events total  
**When**: `run_rebuild_views(config)` completes  
**Then**: `RebuildStats.events_processed == 10` and `RebuildStats.instances_rebuilt == 3`

### test_run_rebuild_views_shows_progress_when_enabled
**Given**: `show_progress: true`  
**When**: `run_rebuild_views(config)` is called  
**Then**: Progress messages are printed to stdout

---

## Error Path Tests

### test_run_rebuild_views_returns_error_on_nats_connection_failure
**Given**: NATS server unavailable at `NatsConfig::default()` address  
**When**: `run_rebuild_views(config)` is called  
**Then**: Returns `Err(WtfError::NatsPublish { message: "..." })`

### test_run_rebuild_views_returns_error_on_kv_provision_failure
**Given**: NATS connected but JetStream unavailable  
**When**: `run_rebuild_views(config)` is called  
**Then**: Returns `Err(WtfError::NatsPublish { message: "..." })` from `provision_kv_buckets`

### test_run_rebuild_views_returns_error_on_stream_fetch_failure
**Given**: KV provisioned but stream `wtf-events` does not exist  
**When**: `rebuild_views()` is called  
**Then**: Returns `Err(WtfError::NatsPublish { message: "..." })`

---

## Edge Case Tests

### test_run_rebuild_views_handles_empty_namespace
**Given**: JetStream with no events in "nonexistent" namespace  
**When**: `run_rebuild_views(RebuildViewsConfig { namespace: Some("nonexistent".into()), ... })` is called  
**Then**: Returns `Ok(ExitCode::SUCCESS)` with `instances_rebuilt == 0`

### test_run_rebuild_views_handles_instance_with_no_events
**Given**: Subject exists in stream but no events (shouldn't happen in valid state)  
**When**: Instance is processed  
**Then**: InstanceView not written (or written with initial state)

### test_run_rebuild_views_handles_instance_with_single_event
**Given**: Instance with exactly 1 event (InstanceStarted)  
**When**: Instance is replayed  
**Then**: InstanceView shows status="started", last_event_seq=1

### test_run_rebuild_views_handles_instance_with_many_events
**Given**: Instance with 1000+ events  
**When**: Instance is replayed  
**Then**: All events applied in order, final state reflects last event

### test_run_rebuild_views_derives_final_state_from_terminal_event
**Given**: Instance with InstanceStarted → TransitionApplied → InstanceCompleted events  
**When**: Instance is replayed  
**Then**: InstanceView status="completed", current_state from final event

### test_view_name_parse_accepts_valid_names
**Given**: Valid view names  
**When**: `ViewName::parse("instances")`, `ViewName::parse("timers")`, etc.  
**Then**: Returns `Some(ViewName::Instances)`, `Some(ViewName::Timers)`, etc.

### test_view_name_parse_rejects_invalid_names
**Given**: Invalid view name  
**When**: `ViewName::parse("invalid")`  
**Then**: Returns `None`

### test_view_name_parse_case_insensitive
**Given**: Uppercase view names  
**When**: `ViewName::parse("INSTANCES")`, `ViewName::parse("Timers")`  
**Then**: Returns `Some(ViewName::Instances)`, `Some(ViewName::Timers)`

---

## Contract Verification Tests

### test_precondition_nats_connection_required
**Given**: Invalid NATS config  
**When**: `connect(&invalid_config)` is called  
**Then**: Returns `Err(WtfError::NatsPublish { message: "..." })`

### test_precondition_kv_buckets_must_be_provisioned
**Given**: Provisioning with invalid JetStream context  
**When**: `provision_kv_buckets(&invalid_js)` is called  
**Then**: Returns `Err(WtfError::NatsPublish { message: "..." })`

### test_postcondition_all_instances_have_kv_entry
**Given**: 5 instances in JetStream under namespace "payments"  
**When**: `rebuild_views()` completes  
**Then**: 5 entries written to `wtf-instances` KV with keys `payments/<instance_id>`

### test_postcondition_stats_reflect_actual_work
**Given**: Known event stream  
**When**: `rebuild_views()` returns `stats`  
**Then**: `stats.events_processed` equals actual events replayed

### test_invariant_kv_never_ahead_of_jetstream
**Given**: Valid rebuild  
**When**: Each KV write occurs  
**Then**: Sequence number in InstanceView ≤ last event sequence in stream for that instance

---

## Dry-Run Mode Tests

### test_dry_run_does_not_connect_to_nats
**Given**: `dry_run: true`  
**When**: `run_rebuild_views(config)` is called  
**Then**: `connect()` is never called (verified via mock or early return)

### test_dry_run_does_not_provision_kv
**Given**: `dry_run: true`  
**When**: `run_rebuild_views(config)` is called  
**Then**: `provision_kv_buckets()` is never called

### test_dry_run_prints_intended_actions
**Given**: `dry_run: true` with view and namespace filters  
**When**: `run_rebuild_views(config)` is called  
**Then**: Prints "[dry-run] Would rebuild views" and filter values

### test_dry_run_returns_success
**Given**: `dry_run: true`  
**When**: `run_rebuild_views(config)` is called  
**Then**: Returns `Ok(ExitCode::SUCCESS)`

---

## Given-When-Then Scenarios

### Scenario 1: Full rebuild of all namespaces and views
**Given**: JetStream with events across multiple namespaces (payments, onboarding)  
**And**: `wtf-instances`, `wtf-timers`, `wtf-definitions` KV buckets exist  
**When**: `wtf admin rebuild-views` runs without filters  
**Then**:
- All instances from all namespaces are discovered via subject scan `wtf.log.>`
- Each instance's events are replayed from seq=1
- Final state written to appropriate KV bucket
- Summary printed: "X instances processed, Y KV entries written"

### Scenario 2: Targeted rebuild of single namespace
**Given**: JetStream with events in "payments" and "onboarding" namespaces  
**When**: `wtf admin rebuild-views --namespace payments` runs  
**Then**:
- Only subjects matching `wtf.log.payments.*` are scanned
- "onboarding" namespace is not touched
- Stats show only payments instances processed

### Scenario 3: Disaster recovery after KV wipe
**Given**: NATS JetStream intact but KV buckets empty/corrupted  
**When**: `wtf admin rebuild-views --namespace payments` runs  
**Then**:
- All instances in payments namespace are discovered
- Events replayed to derive current state
- KV entries reconstructed
- Command reports success with count of entries written

### Scenario 4: Dry-run to inspect before rebuilding
**Given**: Existing KV and JetStream  
**When**: `wtf admin rebuild-views --namespace payments --dry-run` runs  
**Then**:
- No writes occur to KV
- Output shows what WOULD be written
- Safe to run multiple times for inspection
