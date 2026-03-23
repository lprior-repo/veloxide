# Agent Instructions

## Project Overview

wtf-engine is a durable execution runtime (~39K Rust LOC across 9 crates). It runs long-lived workflows with guaranteed no lost transitions — backed by NATS JetStream event log.

**Tech stack:** Rust (end-to-end), Ractor actors, axum HTTP, Dioxus WASM frontend, NATS JetStream/KV, sled snapshots.

---

## NATS Connection

NATS is running in Docker (`wtf-nats-test` container on port 4222):

```bash
# Verify connection
cargo run -p wtf-storage --bin nats_connect_test

# Run full test suite (requires NATS)
cargo test --workspace
```

---

## Issue Tracking with Beads

**⚠️ Dolt/bd database is NOT available** in this environment (`database "wtf" not found on 127.0.0.1:3308`).

Beads are tracked in `.beads/<bead-id>/` directories. Since bd is unavailable, contracts and test plans must be **synthesized from actual implementation code** — not pulled from a database.

### Current Bead Status

| Category | Count | Notes |
|----------|-------|-------|
| **LANDED (STATE 8)** | 48 | Fully implemented, tested, committed |
| **GHOST (STATE 1, empty)** | 11 | Need cleanup or re-implementation |
| **Total** | 60 | |

**Ghost beads (empty, no artifacts):**
`wtf-2q3d`, `wtf-5eii`, `wtf-772u`, `wtf-bqiq`, `wtf-ibdy`, `wtf-iu4d`, `wtf-lrko`, `wtf-p19r`, `wtf-pc26`, `wtf-wygu`, `wtf-xgxr`

---

## Running Tests

```bash
# All workspace tests (requires NATS running)
cargo test --workspace

# Crate-specific
cargo test -p wtf-actor
cargo test -p wtf-storage
cargo test -p wtf-linter

# With output
cargo test --workspace -- --nocapture

# Clippy
cargo clippy --workspace -- -D warnings
```

---

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below:

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** — Create beads for anything that needs follow-up:
   ```bash
   mkdir -p .beads/wtf-<id>
   echo "STATE 1" > .beads/wtf-<id>/STATE.md
   ```

2. **Run quality gates** (if code changed):
   ```bash
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo check --workspace
   ```

3. **Commit and push**:
   ```bash
   jj describe -m "description"
   jj git push
   ```

4. **Verify**:
   ```bash
   jj log --no-graph -r "main | main@origin"
   # Must show synced
   ```

**CRITICAL RULES:**
- Work is NOT complete until pushed to remote
- NEVER stop before pushing — that leaves work stranded
- If push fails, resolve and retry

---

## Go-skill Pipeline (Implementing New Features)

Since bd is unavailable, use the go-skill pipeline with contract synthesis from existing code:

```
STATE 1 → rust-contract (synthesize contract.md + martin-fowler-tests.md from implementation)
STATE 2 → test-reviewer (verify test plan quality)
STATE 3 → functional-rust (verify implementation matches contract)
STATE 4 → Moon Gate (cargo check, cargo test, cargo clippy)
STATE 4.5 → qa-enforcer (actual command execution, not faked)
STATE 4.6 → QA review
STATE 5 → red-queen (adversarial testing to break implementation)
STATE 5.5 → black-hat-reviewer
STATE 5.7 → kani-justification or kani run
STATE 6 → repair loop (if needed)
STATE 7 → architectural-drift (enforce <300 line files, DDD principles)
STATE 8 → jj git push --bookmark main
```

---

## Non-Interactive Shell Commands

**ALWAYS use non-interactive flags** with file operations:

```bash
# Force overwrite without prompting
cp -f source dest
mv -f source dest
rm -f file

# For recursive operations
rm -rf directory
cp -rf source dest
```

**Other commands that may prompt:**
- `scp` — use `-o BatchMode=yes`
- `ssh` — use `-o BatchMode=yes`
- `apt-get` — use `-y` flag

---

## Key Crates

| Crate | LOC | Purpose |
|-------|-----|---------|
| `wtf-common` | 690 | `WorkflowEvent`, `InstanceId`, `RetryPolicy` |
| `wtf-actor` | 3,896 | Ractor actors, FSM/DAG/Procedural paradigms |
| `wtf-storage` | 1,362 | JetStream journal, KV, sled snapshots |
| `wtf-api` | 1,786 | axum HTTP, SSE, workflow handlers |
| `wtf-cli` | 996 | `wtf serve`, `wtf lint`, `wtf admin` |
| `wtf-linter` | 1,968 | 6 procedural workflow lint rules |
| `wtf-frontend` | 27,145 | Dioxus WASM dashboard |

---

## Known Issues

1. **7 journal_test failures** — assertions don't provide required `Extension<ActorRef<OrchestratorMsg>>`, all return 500 instead of expected status codes
2. **11 ghost beads** — empty STATE 1 directories, no artifacts, no implementation
3. **wtf-cli has 0 tests** — no test coverage
4. **wtf-worker has 0 tests** — no test coverage
