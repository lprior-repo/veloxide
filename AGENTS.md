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

1. **Run quality gates** (if code changed):
   ```bash
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo check --workspace
   ```

2. **Commit and push via jj**:
   ```bash
   jj describe -m "description"
   jj git push
   ```

3. **Verify**:
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

Use the go-skill pipeline with contract synthesis from existing code:

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
2. **wtf-cli has 0 tests** — no test coverage
3. **wtf-worker has 0 tests** — no test coverage

<!-- BEGIN BEADS INTEGRATION v:1 profile:full hash:d4f96305 -->
## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Dolt-powered version control with native sync
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**

```bash
bd ready --json
```

**Create new issues:**

```bash
bd create "Issue title" --description="Detailed context" -t bug|feature|task -p 0-4 --json
bd create "Issue title" --description="What this issue is about" -p 1 --deps discovered-from:bd-123 --json
```

**Claim and update:**

```bash
bd update <id> --claim --json
bd update bd-42 --priority 1 --json
```

**Complete work:**

```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task atomically**: `bd update <id> --claim`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" --description="Details about what was found" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`

### Auto-Sync

bd automatically syncs via Dolt:

- Each write auto-commits to Dolt history
- Use `bd dolt push`/`bd dolt pull` for remote sync
- No manual export/import needed!

### Important Rules

- ✅ Use bd for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `bd ready` before asking "what should I work on?"
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems

For more details, see README.md and docs/QUICKSTART.md.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

<!-- END BEADS INTEGRATION -->
