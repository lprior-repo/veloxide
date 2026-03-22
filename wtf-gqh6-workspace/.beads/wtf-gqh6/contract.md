# Contract Specification: TimeTravelScrubber

## Context
- Feature: Monitor Mode time-travel scrubber for wtf-frontend
- Domain terms: seq (sequence number), max_seq (latest sequence), replay, frozen state, historical mode, live mode, ScrubberState
- Location: crates/wtf-frontend/src/ui/monitor_mode.rs
- Assumptions: WtfClient has get_event_log and replay_to methods; SSE stream provides live updates

## Preconditions
- [P1] seq must be >= 0 and <= max_seq when calling replay_to
- [P2] instance_id must be non-empty string
- [P3] API client must be in connected state before replay

## Postconditions
- [Q1] replay_to returns Some(FrozenState) with seq preserved
- [Q2] ScrubberState stores correct seq value
- [Q3] Signal<Option<ScrubberState>> reflects mode: None=live, Some(_) = historical
- [Q4] Historical mode disables SSE subscription

## Invariants
- [I1] Slider bounds: min=0, max=max_seq (dynamic)
- [I2] Playback position never exceeds max_seq
- [I3] Reset always returns to live mode (Signal = None)

## Error Taxonomy
- Error::InvalidSequence - when seq < 0 or seq > max_seq
- Error::InstanceNotFound - when instance_id doesn't exist
- Error::ApiConnectionFailed - when client cannot reach API
- Error::ReplayInProgress - when replay already running

## Contract Signatures
```rust
// In wtf-frontend/src/ui/monitor_mode.rs
struct TimeTravelScrubber {
    state: Signal<Option<ScrubberState>>,
    max_seq: Signal<u64>,
}

struct ScrubberState {
    seq: u64,
    frozen_state: FrozenState,
    is_playing: bool,
}

impl TimeTravelScrubber {
    fn replay_to(seq: u64) -> Result<ScrubberState, Error>;
    fn play() -> Result<(), Error>;
    fn reset() -> ();
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| seq >= 0 | Compile-time | u64 (unsigned) |
| seq <= max_seq | Runtime-checked | Result<ScrubberState, Error::InvalidSequence> |
| instance_id non-empty | Runtime-checked constructor | WtfClient::replay_to validates |
| API connected | Runtime | Result type + Error::ApiConnectionFailed |

## Violation Examples (REQUIRED)
- VIOLATES P1: replay_to(-1) -- should produce `Err(Error::InvalidSequence)`
- VIOLATES P1: replay_to(max_seq + 1) -- should produce `Err(Error::InvalidSequence)`
- VIOLATES P2: replay_to(0) with empty instance_id -- should produce `Err(Error::InstanceNotFound)`
- VIOLATES Q3: After replay_to(5), state should be Some(ScrubberState{seq: 5, ...}), NOT None

## Ownership Contracts
- state Signal: owned by component, mutations through setter methods
- max_seq Signal: read-only derived from API response
- frozen_state: owned by ScrubberState, cloned on each replay_to call

## Non-goals
- [N1] Does not implement graph rendering (delegated to GraphOverlay component)
- [N2] Does not manage SSE subscription directly (handled by parent MonitorMode)
- [N3] Does not persist scrubber position across sessions
