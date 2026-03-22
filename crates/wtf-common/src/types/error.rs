//! Shared error types for wtf-engine.

use thiserror::Error;

/// Top-level error type for wtf-engine public API surface.
#[derive(Debug, Error)]
pub enum WtfError {
    /// NATS `JetStream` publish failed before receiving a `PublishAck` (ADR-015).
    #[error("NATS publish failed: {message}")]
    NatsPublish { message: String },

    /// NATS operation timed out.
    #[error("NATS timeout after {timeout_ms}ms: {operation}")]
    NatsTimeout { operation: String, timeout_ms: u64 },

    /// sled snapshot store I/O failure.
    #[error("sled error: {message}")]
    SledError { message: String },

    /// Replayed log produced different state (non-deterministic workflow — ADR-016).
    #[error("replay divergence at seq {seq}: {detail}")]
    ReplayDivergence { seq: u64, detail: String },

    /// Engine at maximum concurrent instance capacity.
    #[error("capacity exceeded: {running}/{max} instances running")]
    CapacityExceeded { running: usize, max: usize },

    /// No instance found for the given ID.
    #[error("instance not found: {instance_id}")]
    InstanceNotFound { instance_id: String },

    /// Workflow name is empty or contains illegal characters.
    #[error("invalid workflow name: {name:?}")]
    InvalidWorkflowName { name: String },

    /// Input validation failed (e.g., oversized progress string).
    #[error("invalid input: {detail}")]
    InvalidInput { detail: String },

    /// Attempted to use a heartbeat sender after it was stopped.
    #[error("heartbeat sender stopped")]
    HeartbeatStopped,
}

impl WtfError {
    #[must_use]
    pub fn nats_publish(message: impl Into<String>) -> Self {
        Self::NatsPublish {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn nats_timeout(operation: impl Into<String>, timeout_ms: u64) -> Self {
        Self::NatsTimeout {
            operation: operation.into(),
            timeout_ms,
        }
    }

    #[must_use]
    pub fn sled_error(message: impl Into<String>) -> Self {
        Self::SledError {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn replay_divergence(seq: u64, detail: impl Into<String>) -> Self {
        Self::ReplayDivergence {
            seq,
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn instance_not_found(instance_id: impl Into<String>) -> Self {
        Self::InstanceNotFound {
            instance_id: instance_id.into(),
        }
    }

    #[must_use]
    pub fn invalid_workflow_name(name: impl Into<String>) -> Self {
        Self::InvalidWorkflowName { name: name.into() }
    }
}
