# Implementation Summary

## Bead: wtf-07zs
## Title: Heartbeat-driven crash recovery for workflow instances

## Implemented

- Hardened heartbeat recovery handler in `crates/wtf-actor/src/master/handlers/heartbeat.rs`:
  - Active-instance guard now logs and skips spurious recovery trigger.
  - Missing metadata path now logs warning and exits recovery path.
  - Added in-flight dedupe guard (`OnceLock<Mutex<HashSet<String>>>`) to avoid duplicate concurrent recovery attempts for same instance.
  - In-flight token is removed after recovery attempt completion.

## Contract alignment notes

- Q4 behavior (skip when instance still active) enforced explicitly.
- Q5 behavior (skip when metadata missing) enforced with warning path.
- I1 behavior (single in-flight recovery per instance) implemented via dedupe set.

## Verification

- `cargo check -p wtf-actor`

## Files changed

- `crates/wtf-actor/src/master/handlers/heartbeat.rs`
