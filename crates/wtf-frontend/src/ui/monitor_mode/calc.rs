//! Pure calculation functions for TimeTravelScrubber
//!
//! All functions in this module are pure (no I/O, no mutation).
//! They enforce preconditions, postconditions, and invariants from the contract.

use super::data::{MonitorMode, ScrubberError, ScrubberState, Seq};

/// Validates replay sequence against preconditions
///
/// # Contract
/// - P1: seq must be >= 0 and <= max_seq
/// - P2: instance_id must be non-empty (checked at call site)
///
/// # Arguments
/// * `seq` - Target sequence number
/// * `max_seq` - Maximum valid sequence number
///
/// # Returns
/// * `Ok(Seq)` if valid
/// * `Err(ScrubberError::InvalidSequence)` if seq > max_seq or max_seq == 0
#[must_use]
pub fn validate_replay_seq(seq: u64, max_seq: u64) -> Result<Seq, ScrubberError> {
    Seq::try_new(seq, max_seq)
}

/// Computes the next sequence in playback direction
///
/// # Arguments
/// * `current` - Current sequence
/// * `max_seq` - Maximum valid sequence
///
/// # Returns
/// * `Some(next_seq)` if not at boundary
/// * `None` if at max_seq (stop playback)
#[must_use]
pub fn calculate_playback_tick(current: u64, max_seq: u64) -> Option<u64> {
    current
        .checked_add(1)
        .and_then(|next| if next > max_seq { None } else { Some(next) })
}

/// Determines monitor mode from scrubber state (postcondition Q3)
#[must_use]
pub fn compute_monitor_mode(state: &Option<ScrubberState>) -> MonitorMode {
    MonitorMode::from_option(state)
}

/// Creates ScrubberState after successful replay (postcondition Q1, Q2)
#[must_use]
pub fn create_scrubber_state(seq: u64, frozen_state: super::data::FrozenState) -> ScrubberState {
    ScrubberState::new(seq, frozen_state, false)
}
