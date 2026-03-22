bead_id: wtf-5so3
bead_title: epic: Phase 7 — CLI + Integration (wtf-cli + E2E tests)
phase: 7
updated_at: 2026-03-21T23:50:00Z

# Martin Fowler Test Plan: Phase 7 CLI + Integration

## Test Strategy
Using Given-When-Then BDD format per Dave Farley's ATDD principles. Each child bead has explicit test scenarios.

---

## Bead: wtf-cli-serve

### Scenario 1: Server starts successfully
**Given**: No server is running on port 8080
**When**: I run `wtf serve --host 127.0.0.1 --port 8080`
**Then**: Server starts and binds to 127.0.0.1:8080
**And**: Process exits with code 0 when stopped

### Scenario 2: Health endpoint responds
**Given**: Server is running with health endpoint at `/health`
**When**: I send GET request to `http://127.0.0.1:8080/health`
**Then**: Response is 200 OK with `{"status":"ok"}`

### Scenario 3: Graceful shutdown on SIGINT
**Given**: Server is running
**When**: I send SIGINT to the server process
**Then**: Server stops gracefully within 10 seconds
**And**: No zombie processes remain

### Scenario 4: Server fails on port conflict
**Given**: Port 8080 is already in use
**When**: I run `wtf serve --port 8080`
**Then**: Error message indicates port conflict
**And**: Exit code is non-zero

---

## Bead: wtf-cli-lint

### Scenario 1: Valid workflow passes linting
**Given**: A valid workflow definition file at `workflows/valid.yaml`
**When**: I run `wtf lint workflows/valid.yaml`
**Then**: Exit code is 0
**And**: Output shows no errors

### Scenario 2: Invalid workflow reports errors
**Given**: An invalid workflow with missing required fields
**When**: I run `wtf lint workflows/invalid.yaml`
**Then**: Exit code is non-zero
**And**: Each lint error is reported with file:line:column
**And**: Error count is accurate

### Scenario 3: Lint reports all violations
**Given**: A workflow with 3 separate lint violations
**When**: I run `wtf lint workflows/multi-error.yaml`
**Then**: All 3 violations are reported
**And**: No violations are silently ignored

### Scenario 4: JSON output format
**Given**: A workflow with errors
**When**: I run `wtf lint --format json workflows/invalid.yaml`
**Then**: Output is valid JSON
**And**: JSON contains array of diagnostic objects

### Scenario 5: Lint directory recursively
**Given**: A directory `workflows/` with multiple workflow files
**When**: I run `wtf lint workflows/`
**Then**: All .yaml and .yml files are linted
**And**: Aggregate exit code reflects any errors found

---

## Bead: wtf-cli-admin-rebuild

### Scenario 1: Rebuild views successfully
**Given**: Views exist in storage
**When**: I run `wtf admin rebuild-views`
**Then**: All views are rebuilt
**And**: Progress is reported
**And**: Exit code is 0

### Scenario 2: Rebuild is idempotent
**Given**: Views have been rebuilt
**When**: I run `wtf admin rebuild-views` again
**Then**: Same result as first run
**And**: No duplicate or corrupted data

### Scenario 3: Rebuild with specific view
**Given**: Multiple views exist
**When**: I run `wtf admin rebuild-views --view execution_view`
**Then**: Only execution_view is rebuilt
**And**: Other views are unchanged

---

## Bead: wtf-e2e-integration

### Scenario 1: Create and execute workflow
**Given**: NATS and storage are running
**When**: I create a new workflow instance
**And**: Execute it to completion
**Then**: Final state is persisted correctly
**And**: All activities completed with correct results

### Scenario 2: Crash mid-execution preserves state
**Given**: A workflow is mid-execution with partial results
**When**: Simulated crash occurs (process kill)
**Then**: Workflow state is durable
**And**: No data loss on restart

### Scenario 3: Replay produces identical results
**Given**: A workflow executed partially then crashed
**When**: System recovers and replays the workflow
**Then**: Re-executed activities produce bit-identical results
**And**: Final state matches expected state
**And**: No duplicate side effects

### Scenario 4: Activity retries on failure
**Given**: An activity fails transiently
**When**: Activity is retried
**Then**: Activity eventually succeeds
**And**: Workflow continues to completion

### Scenario 5: Timer fires correctly after crash
**Given**: A workflow with a wait timer set
**When**: Timer fires after system restart
**Then**: Workflow resumes from correct state
**And**: No timer events are lost or duplicated

### Scenario 6: Concurrent workflow execution
**Given**: Multiple workflows are created simultaneously
**When**: They execute in parallel
**Then**: Each completes independently
**And**: No race conditions or data corruption

---

## Test Execution Order
1. Unit tests for each CLI command (mocked dependencies)
2. Integration tests with real storage (no NATS)
3. Full E2E tests with NATS (CI environment)
