//! `WorkflowEvent` — the only type written to NATS `JetStream` (ADR-013).

use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// A declared side effect attached to an FSM transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectDeclaration {
    /// Name of the effect (e.g. `"CallAuthorizationService"`).
    pub effect_type: String,
    /// Msgpack-encoded effect payload.
    pub payload: Bytes,
}

/// How many times to retry a failed activity before giving up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of attempts (including the first). 0 = no retries.
    pub max_attempts: u32,
    /// Initial delay in milliseconds before the first retry.
    pub initial_interval_ms: u64,
    /// Exponential backoff multiplier (e.g. 2.0 = double delay each retry).
    pub backoff_coefficient: f64,
    /// Maximum delay cap in milliseconds.
    pub max_interval_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_interval_ms: 1_000,
            backoff_coefficient: 2.0,
            max_interval_ms: 60_000,
        }
    }
}
