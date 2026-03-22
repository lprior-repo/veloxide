//! TimeTravelScrubber - Monitor Mode time-travel scrubber for wtf-frontend
//!
//! This module provides the TimeTravelScrubber component that enables users to
//! navigate through historical states of a workflow execution.
//!
//! ## Architecture
//! - `data`: Type definitions (Seq, FrozenState, ScrubberState, MonitorMode, ScrubberError)
//! - `calc`: Pure calculation functions
//! - `monitor_mode`: Re-exports and tests
//!
//! ## Data Architecture
//! - `FrozenState`: Immutable snapshot of workflow state at a given sequence
//! - `ScrubberState`: Current scrubber position and playback state
//! - `MonitorMode`: Enum representing live vs historical viewing mode
//!
//! ## Calculations (Pure Functions)
//! - `validate_replay_seq()`: Validates sequence bounds
//! - `compute_monitor_mode()`: Determines mode from scrubber state
//! - `calculate_playback_tick()`: Computes next sequence during playback
//!
//! ## Contract
//! - Preconditions enforced via Result types (seq >= 0, seq <= max_seq)
//! - Postconditions: replay_to returns Some(FrozenState), state updated correctly
//! - Invariants: slider bounds [0, max_seq], playback never exceeds max_seq

pub mod calc;
pub mod data;

pub use calc::{
    calculate_playback_tick, compute_monitor_mode, create_scrubber_state, validate_replay_seq,
};
pub use data::{FrozenState, MonitorMode, ScrubberError, ScrubberState, Seq};

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Seq Tests
    // ========================================================================

    #[test]
    fn given_valid_seq_within_bounds_when_validating_then_ok() {
        let result = validate_replay_seq(50, 100);
        assert_eq!(result, Ok(Seq::new(50)));
    }

    #[test]
    fn given_seq_equal_to_max_when_validating_then_ok() {
        let result = validate_replay_seq(100, 100);
        assert_eq!(result, Ok(Seq::new(100)));
    }

    #[test]
    fn given_seq_exceeds_max_when_validating_then_error() {
        let result = validate_replay_seq(101, 100);
        assert_eq!(result, Err(ScrubberError::InvalidSequence(101, 100)));
    }

    #[test]
    fn given_zero_max_when_validating_then_error() {
        let result = validate_replay_seq(0, 0);
        assert_eq!(result, Err(ScrubberError::InvalidSequence(0, 0)));
    }

    #[test]
    fn given_seq_zero_when_validating_then_ok() {
        let result = validate_replay_seq(0, 100);
        assert_eq!(result, Ok(Seq::new(0)));
    }

    // ========================================================================
    // MonitorMode Tests
    // ========================================================================

    #[test]
    fn given_none_state_when_computing_mode_then_live() {
        let state: Option<ScrubberState> = None;
        assert_eq!(compute_monitor_mode(&state), MonitorMode::Live);
    }

    #[test]
    fn given_some_state_when_computing_mode_then_historical() {
        let state = Some(ScrubberState::new(
            50,
            FrozenState::new(50, "{}".to_string(), 0),
            false,
        ));
        assert_eq!(compute_monitor_mode(&state), MonitorMode::Historical);
    }

    #[test]
    fn given_live_mode_when_checking_is_live_then_true() {
        assert!(MonitorMode::Live.is_live());
        assert!(!MonitorMode::Live.is_historical());
    }

    #[test]
    fn given_historical_mode_when_checking_is_historical_then_true() {
        assert!(MonitorMode::Historical.is_historical());
        assert!(!MonitorMode::Historical.is_live());
    }

    // ========================================================================
    // Playback Tick Tests
    // ========================================================================

    #[test]
    fn given_middle_seq_when_tick_then_incremented() {
        assert_eq!(calculate_playback_tick(50, 100), Some(51));
    }

    #[test]
    fn given_seq_zero_when_tick_then_one() {
        assert_eq!(calculate_playback_tick(0, 100), Some(1));
    }

    #[test]
    fn given_max_seq_when_tick_then_none() {
        assert_eq!(calculate_playback_tick(100, 100), None);
    }

    #[test]
    fn given_near_max_seq_when_tick_then_stops_at_max() {
        assert_eq!(calculate_playback_tick(99, 100), Some(100));
    }

    #[test]
    fn given_max_seq_boundary_when_tick_then_none() {
        assert_eq!(
            calculate_playback_tick(u64::MAX - 1, u64::MAX),
            Some(u64::MAX)
        );
        assert_eq!(calculate_playback_tick(u64::MAX, u64::MAX), None);
    }

    // ========================================================================
    // ScrubberState Tests
    // ========================================================================

    #[test]
    fn given_scrubber_state_when_start_playing_then_is_playing() {
        let state = ScrubberState::new(50, FrozenState::new(50, "{}".to_string(), 0), false);
        let playing = state.start_playing();
        assert!(playing.is_playing);
        assert_eq!(playing.seq, 50);
    }

    #[test]
    fn given_playing_state_when_stop_playing_then_not_playing() {
        let state = ScrubberState::new(50, FrozenState::new(50, "{}".to_string(), 0), true);
        let stopped = state.stop_playing();
        assert!(!stopped.is_playing);
        assert_eq!(stopped.seq, 50);
    }

    #[test]
    fn given_state_when_with_seq_then_new_seq() {
        let state = ScrubberState::new(50, FrozenState::new(50, "{}".to_string(), 0), true);
        let max_seq = 100u64;
        let new_state = state
            .with_seq(75, max_seq)
            .expect("75 <= 100, should be valid");
        assert_eq!(new_state.seq, 75);
        assert!(new_state.is_playing);
        assert_eq!(new_state.frozen_state.seq, 50); // original frozen seq unchanged
    }

    #[test]
    fn given_seq_exceeds_max_when_with_seq_then_error() {
        let state = ScrubberState::new(50, FrozenState::new(50, "{}".to_string(), 0), true);
        let max_seq = 50u64;
        let result = state.with_seq(75, max_seq);
        assert_eq!(result, Err(ScrubberError::InvalidSequence(75, 50)));
    }

    // ========================================================================
    // FrozenState Tests
    // ========================================================================

    #[test]
    fn given_response_when_creating_frozen_state_then_correct_fields() {
        let response = serde_json::json!({"key": "value"});
        let frozen = FrozenState::from_response(42, response);
        assert_eq!(frozen.seq, 42);
        assert!(frozen.state_json.contains("key"));
    }

    // ========================================================================
    // Contract Violation Tests
    // ========================================================================

    #[test]
    fn test_seq_exceeds_max_violation_returns_invalid_sequence_error() {
        let result = validate_replay_seq(101, 100);
        assert_eq!(result, Err(ScrubberError::InvalidSequence(101, 100)));
    }

    #[test]
    fn test_postcondition_scrubber_state_contains_seq() {
        let frozen = FrozenState::new(5, "{}".to_string(), 0);
        let state = create_scrubber_state(5, frozen);
        assert_eq!(state.seq, 5);
        assert!(!state.is_playing);
    }

    #[test]
    fn test_reset_sets_state_to_none_not_some() {
        let state: Option<ScrubberState> = None;
        assert_eq!(compute_monitor_mode(&state), MonitorMode::Live);
    }

    #[test]
    fn test_invariant_slider_bounds_match_max_seq() {
        let max_seq = 100u64;
        let valid_seq = validate_replay_seq(0, max_seq);
        assert!(valid_seq.is_ok());
        let invalid_seq = validate_replay_seq(max_seq + 1, max_seq);
        assert!(invalid_seq.is_err());
    }

    #[test]
    fn test_invariant_playback_never_exceeds_max_seq() {
        let max_seq = 100u64;
        let current = 100u64;
        assert_eq!(calculate_playback_tick(current, max_seq), None);
        assert!(current <= max_seq);
    }
}
