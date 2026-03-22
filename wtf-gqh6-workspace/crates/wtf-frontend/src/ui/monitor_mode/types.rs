#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ScrubberError {
    #[error("sequence {0} out of bounds [0, {1}]")]
    InvalidSequence(u64, u64),
    #[error("instance not found")]
    InstanceNotFound,
    #[error("API connection failed: {0}")]
    ApiConnectionFailed(String),
    #[error("replay already in progress")]
    ReplayInProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrozenState {
    pub seq: u64,
    pub state_json: serde_json::Value,
    pub timestamp: String,
}

impl FrozenState {
    #[must_use]
    pub fn new(seq: u64, state_json: serde_json::Value, timestamp: String) -> Self {
        Self {
            seq,
            state_json,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorMode {
    Live,
    Historical,
}

impl MonitorMode {
    #[must_use]
    pub fn is_live(&self) -> bool {
        matches!(self, MonitorMode::Live)
    }

    #[must_use]
    pub fn is_historical(&self) -> bool {
        matches!(self, MonitorMode::Historical)
    }
}

#[derive(Debug, Clone)]
pub struct ScrubberState {
    pub seq: u64,
    pub frozen_state: FrozenState,
    pub is_playing: bool,
    pub mode: MonitorMode,
}

impl ScrubberState {
    #[must_use]
    pub fn new(seq: u64, frozen_state: FrozenState) -> Self {
        Self {
            seq,
            frozen_state,
            is_playing: false,
            mode: MonitorMode::Historical,
        }
    }

    #[must_use]
    pub fn with_playing(mut self, is_playing: bool) -> Self {
        self.is_playing = is_playing;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrubberBounds {
    pub min_seq: u64,
    pub max_seq: u64,
}

impl ScrubberBounds {
    #[must_use]
    pub fn new(min_seq: u64, max_seq: u64) -> Self {
        Self { min_seq, max_seq }
    }

    #[must_use]
    pub fn contains(&self, seq: u64) -> bool {
        seq >= self.min_seq && seq <= self.max_seq
    }

    #[must_use]
    pub fn clamp(&self, seq: u64) -> u64 {
        seq.clamp(self.min_seq, self.max_seq)
    }
}

#[derive(Debug, Clone)]
pub struct ReplayResponse {
    pub seq: u64,
    pub state: serde_json::Value,
    pub timestamp: String,
}

impl ReplayResponse {
    #[must_use]
    pub fn into_frozen_state(self) -> FrozenState {
        FrozenState::new(self.seq, self.state, self.timestamp)
    }
}
