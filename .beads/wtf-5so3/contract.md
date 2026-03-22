bead_id: wtf-5so3
bead_title: epic: Phase 7 — CLI + Integration (wtf-cli + E2E tests)
phase: 7
updated_at: 2026-03-21T23:50:00Z

# Contract: EPIC Decomposition Plan

## Epic Overview
Phase 7 delivers the wtf-cli binary and end-to-end integration tests. This is an EPIC container bead that spawns three child CLI command beads and one E2E test bead.

## Child Beads Required

### 1. wtf-cli-serve
**Title**: CLI command: `wtf serve` (HTTP API server)
**Description**: Implement `wtf serve` command that starts the Axum HTTP API server. Should bind to configurable host:port, initialize NATS connection, and serve the REST API defined in wtf-api.
**Priority**: 1
**Dependencies**: None (base phase)
**Acceptance Criteria**:
- `wtf serve --host 0.0.0.0 --port 8080` starts the HTTP server
- Health endpoint responds at `/health`
- Graceful shutdown on SIGINT/SIGTERM

### 2. wtf-cli-lint
**Title**: CLI command: `wtf lint` (Workflow linting)
**Description**: Implement `wtf lint` command that invokes the wtf-linter to validate workflow definitions. Should accept file paths or directories, output diagnostics in structured format.
**Priority**: 2
**Dependencies**: None (base phase)
**Acceptance Criteria**:
- `wtf lint <workflow-file>` returns 0 if valid, non-zero if errors
- JSON/ человеко-readable output format
- Reports all lint rule violations

### 3. wtf-cli-admin-rebuild
**Title**: CLI command: `wtf admin rebuild-views` (View rebuild)
**Description**: Implement `wtf admin rebuild-views` command for administrative maintenance. Should rebuild view tables/materializations required for query performance.
**Priority**: 2
**Dependencies**: wtf-cli-serve (needs API to exist)
**Acceptance Criteria**:
- `wtf admin rebuild-views` rebuilds all views
- Reports progress and completion status
- Idempotent operation

### 4. wtf-e2e-integration
**Title**: E2E integration tests (crash-and-replay)
**Description**: End-to-end integration tests that verify crash-and-replay correctness. Should test the full workflow lifecycle: create workflow, execute activities, simulate crashes, and verify replay produces identical results.
**Priority**: 1
**Dependencies**: All three CLI commands implemented
**Acceptance Criteria**:
- Tests create workflows and execute to completion
- Simulated crash mid-execution preserves state
- Replay produces bit-identical results
- All tests pass in CI

## Technical Context

### wtf-cli binary structure
```
wtf-cli/
  src/
    main.rs          # clap CLI entry point
    commands/
      serve.rs       # wtf serve
      lint.rs        # wtf lint  
      admin.rs       # wtf admin rebuild-views
    lib.rs
```

### Dependencies between components
- `wtf serve` → `wtf-api` (Axum routes), `wtf-storage` (NATS/KV)
- `wtf lint` → `wtf-linter` (syn/quote AST analysis)
- `wtf admin rebuild-views` → `wtf-storage` (view materialization)
- E2E tests → All of the above + `wtf-worker`

## Phase Dependencies
This epic depends on all previous phases being complete. Child beads should be executed in dependency order:
1. First: wtf-cli-serve, wtf-cli-lint (independent CLI commands)
2. Second: wtf-cli-admin-rebuild (depends on serve)
3. Third: wtf-e2e-integration (depends on all CLI commands)
