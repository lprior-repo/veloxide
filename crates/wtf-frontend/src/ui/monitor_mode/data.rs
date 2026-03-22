//! Data types for TimeTravelScrubber
//!
//! ## Types
//! - `Seq`: Newtype wrapper for sequence numbers
//! - `FrozenState`: Immutable snapshot of workflow state
//! - `ScrubberState`: Current scrubber position and playback state
//! - `MonitorMode`: Live vs historical viewing mode
//! - `ScrubberError`: Domain errors

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Sequence number for event log positioning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seq(u64);

impl Seq {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Validates that this sequence is within bounds [0, max_seq]
    #[must_use]
    pub fn is_valid(self, max_seq: u64) -> bool {
        self.0 <= max_seq
    }

    /// Creates Seq if within bounds, returns error otherwise
    pub fn try_new(value: u64, max_seq: u64) -> Result<Self, super::ScrubberError> {
        if max_seq == 0 {
            return Err(super::ScrubberError::InvalidSequence(value, max_seq));
        }
        if value > max_seq {
            return Err(super::ScrubberError::InvalidSequence(value, max_seq));
        }
        Ok(Self::new(value))
    }
}

impl Default for Seq {
    fn default() -> Self {
        Self(0)
    }
}

/// Frozen snapshot of workflow state at a specific sequence
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenState {
    /// Sequence number this frozen state represents
    pub seq: u64,
    /// JSON representation of workflow state
    pub state_json: String,
    /// Unix timestamp when this state was captured
    pub timestamp: i64,
}

impl FrozenState {
    #[must_use]
    pub const fn new(seq: u64, state_json: String, timestamp: i64) -> Self {
        Self {
            seq,
            state_json,
            timestamp,
        }
    }

    /// Creates a FrozenState from API response data
    ///
    /// Note: `chrono::Utc::now()` requires wasm32 target.
    /// On non-wasm targets, uses a placeholder timestamp of 0.
    #[must_use]
    pub fn from_response(seq: u64, response: serde_json::Value) -> Self {
        #[cfg(target_arch = "wasm32")]
        let timestamp = chrono::Utc::now().timestamp();
        #[cfg(not(target_arch = "wasm32"))]
        let timestamp: i64 = 0;
        Self {
            seq,
            state_json: serde_json::to_string(&response).map_or_else(|_| String::new(), |s| s),
            timestamp,
        }
    }
}

/// Current state of the time-travel scrubber
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScrubberState {
    /// Current sequence position
    pub seq: u64,
    /// Frozen state at current sequence
    pub frozen_state: FrozenState,
    /// Whether playback animation is active
    pub is_playing: bool,
}

impl ScrubberState {
    #[must_use]
    pub const fn new(seq: u64, frozen_state: FrozenState, is_playing: bool) -> Self {
        Self {
            seq,
            frozen_state,
            is_playing,
        }
    }

    /// Returns a new ScrubberState with playback started
    #[must_use]
    pub fn start_playing(&self) -> Self {
        Self {
            seq: self.seq,
            frozen_state: self.frozen_state.clone(),
            is_playing: true,
        }
    }

    /// Returns a new ScrubberState with playback stopped
    #[must_use]
    pub fn stop_playing(&self) -> Self {
        Self {
            seq: self.seq,
            frozen_state: self.frozen_state.clone(),
            is_playing: false,
        }
    }

    /// Returns a new ScrubberState at a new sequence position
    ///
    /// # Contract
    /// - Precondition: `new_seq <= max_seq`
    ///
    /// # Returns
    /// - `Ok(Self)` with updated sequence if `new_seq <= max_seq`
    /// - `Err(ScrubberError::InvalidSequence)` if `new_seq > max_seq`
    pub fn with_seq(&self, new_seq: u64, max_seq: u64) -> Result<Self, ScrubberError> {
        if new_seq > max_seq {
            return Err(ScrubberError::InvalidSequence(new_seq, max_seq));
        }
        Ok(Self {
            seq: new_seq,
            frozen_state: self.frozen_state.clone(),
            is_playing: self.is_playing,
        })
    }
}

/// Monitor mode indicating whether viewing live or historical state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorMode {
    /// Viewing live/up-to-date state with SSE updates
    Live,
    /// Viewing frozen historical state
    Historical,
}

impl MonitorMode {
    /// Determines monitor mode from scrubber state
    #[must_use]
    pub fn from_option(state: &Option<ScrubberState>) -> Self {
        match state {
            Some(_) => Self::Historical,
            None => Self::Live,
        }
    }

    #[must_use]
    pub fn is_live(&self) -> bool {
        matches!(self, Self::Live)
    }

    #[must_use]
    pub fn is_historical(&self) -> bool {
        matches!(self, Self::Historical)
    }
}

/// Errors that can occur during time-travel scrubber operations
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ScrubberError {
    #[error("sequence {0} is out of bounds (max: {1})")]
    InvalidSequence(u64, u64),
    #[error("instance not found")]
    InstanceNotFound,
    #[error("API connection failed")]
    ApiConnectionFailed,
    #[error("replay already in progress")]
    ReplayInProgress,
}
