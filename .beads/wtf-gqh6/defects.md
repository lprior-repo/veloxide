# Black Hat Review Defects: wtf-gqh6 (STATE 5.5)

## Defect Summary

| ID | Severity | Location | Category | Description |
|----|----------|----------|----------|-------------|
| BHR-001 | **P1** | `data.rs:141-147` | Invariant Violation | `ScrubberState::with_seq()` allows seq > max_seq, violates contract invariant I2 |

---

## BHR-001: Invariant Violation in `with_seq()`

### Location
`crates/wtf-frontend/src/ui/monitor_mode/data.rs:141-147`

### Violation
**Contract Invariant I2**: "Playback position never exceeds max_seq"

### Evidence
```rust
/// Returns a new ScrubberState at a new sequence position
#[must_use]
pub fn with_seq(&self, new_seq: u64) -> Self {
    Self {
        seq: new_seq,  // NO VALIDATION!
        frozen_state: self.frozen_state.clone(),
        is_playing: self.is_playing,
    }
}
```

### Attack Vector
```rust
let state = ScrubberState::new(50, FrozenState::new(50, "{}".to_string(), 0), false);
let invalid_state = state.with_seq(999);  // Violates I2: seq 999 exceeds any reasonable max_seq
```

### Root Cause
`with_seq()` provides an escape hatch from `Seq::try_new()` validation. The codebase properly validates sequence bounds in `validate_replay_seq()`, but `with_seq()` allows bypassing this validation entirely.

### Impact
- Contract violation: invariant I2 can be violated at runtime
- Makes illegal states representable
- Undermines the type-driven design that `Seq::try_new()` provides

### Required Fix (choose one)
1. **Option A**: Change signature to return `Result<ScrubberState, ScrubberError>` and validate
2. **Option B**: Remove `with_seq()` entirely and force callers to use validated constructors
3. **Option C**: Add `# SAFETY` comment documenting that callers MUST ensure `new_seq <= max_seq`

---

## Phase Compliance

| Phase | Status |
|-------|--------|
| Phase 1: Contract & Bead Parity | ❌ FAIL - I2 violated |
| Phase 2: Farley Engineering Rigor | ✅ PASS |
| Phase 3: NASA-Level Functional Rust | ❌ FAIL - illegal states representable |
| Phase 4: Ruthless Simplicity & DDD | ✅ PASS |
| Phase 5: Bitter Truth (Velocity) | ⚠️ MARGINAL - code is legible but with_seq is suspicious |

---

*Black Hat Review completed: 2026-03-22T10:35:00Z*
