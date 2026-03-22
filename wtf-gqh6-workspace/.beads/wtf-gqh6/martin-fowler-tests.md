# Martin Fowler Test Plan: TimeTravelScrubber

## Happy Path Tests
- test_replay_to_returns_scrubber_state_with_correct_seq
- test_scrubber_state_signal_reflects_historical_mode
- test_reset_clears_state_to_live_mode
- test_play_animates_through_events_at_correct_interval

## Error Path Tests
- test_replay_to_returns_invalid_sequence_error_when_seq_negative
- test_replay_to_returns_invalid_sequence_error_when_seq_exceeds_max
- test_replay_to_returns_instance_not_found_when_id_empty
- test_replay_to_returns_api_connection_failed_when_client_disconnected

## Edge Case Tests
- test_replay_to_at_seq_zero_returns_valid_state
- test_replay_to_at_max_seq_returns_valid_state
- test_play_at_max_seq_does_not_exceed_bounds
- test_multiple_replay_to_calls_overwrite_previous_state

## Contract Verification Tests
- test_precondition_seq_bounds_enforced
- test_postcondition_scrubber_state_contains_seq
- test_invariant_slider_bounds_match_max_seq
- test_invariant_playback_never_exceeds_max_seq

## Contract Violation Tests
- test_seq_negative_violation_returns_invalid_sequence_error
  Given: replay_to(-1) on valid instance
  When: function is called with violating input
  Then: returns `Err(Error::InvalidSequence)` -- NOT a panic

- test_seq_exceeds_max_violation_returns_invalid_sequence_error
  Given: max_seq=100, replay_to(101) on valid instance
  When: function is called with violating input
  Then: returns `Err(Error::InvalidSequence)` -- NOT a panic

- test_empty_instance_id_violation_returns_instance_not_found
  Given: empty instance_id with seq=0
  When: replay_to is called
  Then: returns `Err(Error::InstanceNotFound)` -- NOT a panic

## Given-When-Then Scenarios

### Scenario 1: User drags scrubber to historical position
Given: User is viewing live monitor mode (Signal = None)
When: User drags scrubber to seq=50
Then: API call GET /api/v1/instances/:id/replay-to/50 succeeds
And: Signal updates to Some(ScrubberState{seq: 50, frozen_state: ...})
And: SSE updates are disabled (historical mode active)

### Scenario 2: User clicks play button in historical mode
Given: Scrubber is at seq=50 in historical mode
When: User clicks play button
Then: Playback advances by 1 seq every 500ms
And: Signal updates each tick
And: Stops at max_seq

### Scenario 3: User clicks reset to return to live
Given: Scrubber is at seq=50 in historical mode
When: User clicks reset button
Then: Signal returns to None (live mode)
And: SSE subscription resumes
And: Current live state is displayed

### Scenario 4: Scrubber shows timestamps at tick marks
Given: max_seq = 1000
When: Slider renders
Then: Tick marks appear at seq positions with corresponding timestamps
And: Labels are human-readable (e.g., "2m ago", "5m ago")
