//! Activity result reporting (bead wtf-nas1).
//!
//! Workers call [`complete_activity`] or [`fail_activity`] after executing an
//! activity. Both functions append the result as a `WorkflowEvent` to JetStream
//! (ADR-015 write-ahead) and then ack the work-queue message.
//!
//! # Write-ahead guarantee
//! The sequence is:
//! 1. Execute activity (side effect).
//! 2. Call `complete_activity` / `fail_activity` — appends event to JetStream.
//! 3. Await `PublishAck` before acking the NATS work-queue message.
//! 4. Ack the work-queue message — removes it from the queue.
//!
//! If the process crashes between steps 2 and 4 the work-queue message is
//! re-delivered, the worker re-executes, and the duplicate `ActivityCompleted`
//! event is handled idempotently by the instance actor (applied_seq check).

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use async_nats::jetstream::Context;
use bytes::Bytes;
use std::sync::Arc;
use wtf_common::{ActivityId, InstanceId, NamespaceId, WorkflowEvent, WtfError};
use wtf_storage::append_event;

/// Report a successful activity result.
///
/// Appends `ActivityCompleted` to JetStream and returns the log sequence number.
/// The caller MUST ack the work-queue message after this returns `Ok`.
///
/// # Parameters
/// - `js` — JetStream context.
/// - `namespace` — Namespace of the owning workflow instance.
/// - `instance_id` — Instance that owns this activity.
/// - `activity_id` — The `ActivityId` from the dispatched task.
/// - `result` — Msgpack-encoded return value of the activity.
/// - `duration_ms` — Wall-clock execution time in milliseconds.
///
/// # Errors
/// Returns `WtfError::NatsPublish` if the append or ack fails.
pub async fn complete_activity(
    js: &Context,
    namespace: &NamespaceId,
    instance_id: &InstanceId,
    activity_id: &ActivityId,
    result: Bytes,
    duration_ms: u64,
) -> Result<u64, WtfError> {
    let event = WorkflowEvent::ActivityCompleted {
        activity_id: activity_id.as_str().to_owned(),
        result,
        duration_ms,
    };

    let seq = append_event(js, namespace, instance_id, &event).await?;

    tracing::debug!(
        %namespace,
        %instance_id,
        %activity_id,
        seq,
        duration_ms,
        "activity completed"
    );

    Ok(seq)
}

/// Report a failed activity result.
///
/// Appends `ActivityFailed` to JetStream and returns the log sequence number.
/// The caller MUST ack the work-queue message after this returns `Ok`.
///
/// # Parameters
/// - `retries_exhausted` — Set `true` when `attempt >= retry_policy.max_attempts`.
///   The instance actor uses this to transition the workflow to a failed state
///   rather than re-dispatching the activity.
///
/// # Errors
/// Returns `WtfError::NatsPublish` if the append or ack fails.
pub async fn fail_activity(
    js: &Context,
    namespace: &NamespaceId,
    instance_id: &InstanceId,
    activity_id: &ActivityId,
    error: String,
    retries_exhausted: bool,
) -> Result<u64, WtfError> {
    let event = WorkflowEvent::ActivityFailed {
        activity_id: activity_id.as_str().to_owned(),
        error: error.clone(),
        retries_exhausted,
    };

    let seq = append_event(js, namespace, instance_id, &event).await?;

    tracing::warn!(
        %namespace,
        %instance_id,
        %activity_id,
        %error,
        retries_exhausted,
        seq,
        "activity failed"
    );

    Ok(seq)
}

/// Determine whether retries are exhausted given the attempt number and policy.
///
/// `attempt` is 1-based (first attempt = 1). Returns `true` when no further
/// retries should be attempted.
#[must_use]
pub fn retries_exhausted(attempt: u32, max_attempts: u32) -> bool {
    attempt >= max_attempts
}

/// Calculate exponential backoff delay in milliseconds.
///
/// delay = min(initial_interval_ms * (backoff_coefficient ^ (attempt - 1)), max_interval_ms)
///
/// Returns `None` if the calculated delay would exceed u64::MAX or if attempt is 0.
/// Attempt is 1-based (first attempt = 1).
#[must_use]
pub fn calculate_backoff_delay(attempt: u32, retry_policy: &wtf_common::RetryPolicy) -> Option<u64> {
    if attempt == 0 {
        return None;
    }

    let exponent = (attempt - 1) as f64;
    let multiplier = retry_policy.backoff_coefficient.powf(exponent);
    let delay_f = (retry_policy.initial_interval_ms as f64) * multiplier;

    if delay_f > u64::MAX as f64 {
        return Some(retry_policy.max_interval_ms);
    }

    let delay = delay_f as u64;
    Some(delay.min(retry_policy.max_interval_ms))
}

/// Maximum size of a heartbeat progress string in bytes (1KB).
const MAX_HEARTBEAT_PROGRESS_BYTES: usize = 1024;

/// Send a heartbeat for a running activity.
///
/// Appends `ActivityHeartbeat` to JetStream and returns the sequence number.
/// Heartbeats are fire-and-forget; failures are logged but do not affect activity outcome.
///
/// # Parameters
/// - `js` — JetStream context
/// - `namespace` — Namespace of the owning workflow instance
/// - `instance_id` — Instance that owns this activity
/// - `activity_id` — The activity's unique ID
/// - `progress` — Human-readable progress string (max 1KB)
///
/// # Errors
/// Returns `WtfError::NatsPublish` if the append fails.
/// Returns `WtfError::InvalidInput` if `progress` exceeds 1KB.
pub async fn send_heartbeat(
    js: &Context,
    namespace: &NamespaceId,
    instance_id: &InstanceId,
    activity_id: &ActivityId,
    progress: &str,
) -> Result<u64, WtfError> {
    let progress_bytes = progress.as_bytes();
    if progress_bytes.len() > MAX_HEARTBEAT_PROGRESS_BYTES {
        return Err(WtfError::InvalidInput {
            detail: format!(
                "heartbeat progress exceeds {} bytes (got {})",
                MAX_HEARTBEAT_PROGRESS_BYTES,
                progress_bytes.len()
            ),
        });
    }

    let event = WorkflowEvent::ActivityHeartbeat {
        activity_id: activity_id.as_str().to_owned(),
        progress: progress.to_owned(),
    };

    let seq = append_event(js, namespace, instance_id, &event).await?;

    tracing::debug!(
        %namespace,
        %instance_id,
        %activity_id,
        seq,
        progress_len = progress_bytes.len(),
        "heartbeat sent"
    );

    Ok(seq)
}

/// A handle for sending heartbeats during activity execution.
///
/// Created by the worker before invoking the activity handler.
/// The handler can call `send()` to emit heartbeats and `stop()` to release resources.
#[derive(Debug, Clone)]
pub struct HeartbeatSender {
    js: Context,
    namespace: NamespaceId,
    instance_id: InstanceId,
    activity_id: ActivityId,
    stopped: Arc<std::sync::atomic::AtomicBool>,
}

impl HeartbeatSender {
    /// Create a new heartbeat sender for the given activity.
    #[must_use]
    pub fn new(
        js: Context,
        namespace: NamespaceId,
        instance_id: InstanceId,
        activity_id: ActivityId,
    ) -> Self {
        Self {
            js,
            namespace,
            instance_id,
            activity_id,
            stopped: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Send a heartbeat with the given progress message.
    ///
    /// # Errors
    /// Returns `WtfError::HeartbeatStopped` if `stop()` was already called.
    /// Returns `WtfError::InvalidInput` if `progress` exceeds 1KB.
    /// Returns `WtfError::NatsPublish` if the publish fails.
    pub async fn send(&self, progress: &str) -> Result<u64, WtfError> {
        if self.stopped.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(WtfError::HeartbeatStopped);
        }

        send_heartbeat(
            &self.js,
            &self.namespace,
            &self.instance_id,
            &self.activity_id,
            progress,
        )
        .await
    }

    /// Stop sending heartbeats and release resources.
    ///
    /// Idempotent: calling multiple times is safe.
    pub fn stop(&self) {
        self.stopped.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wtf_common::RetryPolicy;

    // complete_activity / fail_activity require a live NATS server.
    // The write-ahead sequence is covered by integration tests (wtf-2bbn).
    // Unit tests here cover the pure helper.

    #[test]
    fn retries_exhausted_first_attempt_of_one() {
        // max_attempts = 1 means no retries: exhausted on attempt 1
        assert!(retries_exhausted(1, 1));
    }

    #[test]
    fn retries_exhausted_not_yet_on_first_of_three() {
        assert!(!retries_exhausted(1, 3));
    }

    #[test]
    fn retries_exhausted_second_of_three_not_yet() {
        assert!(!retries_exhausted(2, 3));
    }

    #[test]
    fn retries_exhausted_third_of_three_is_exhausted() {
        assert!(retries_exhausted(3, 3));
    }

    #[test]
    fn retries_exhausted_beyond_max_is_exhausted() {
        // Defensive: attempt exceeds max (e.g. due to a race) — treat as exhausted
        assert!(retries_exhausted(5, 3));
    }

    #[test]
    fn retries_exhausted_zero_max_always_exhausted() {
        // max_attempts = 0: even attempt 0 is exhausted
        assert!(retries_exhausted(0, 0));
    }

    #[test]
    fn retries_not_exhausted_at_zero_attempts_when_max_is_three() {
        assert!(!retries_exhausted(0, 3));
    }

    #[test]
    fn calculate_backoff_delay_first_attempt() {
        let policy = RetryPolicy::default();
        let delay = calculate_backoff_delay(1, &policy);
        assert_eq!(delay, Some(1000));
    }

    #[test]
    fn calculate_backoff_delay_second_attempt() {
        let policy = RetryPolicy::default();
        let delay = calculate_backoff_delay(2, &policy);
        assert_eq!(delay, Some(2000));
    }

    #[test]
    fn calculate_backoff_delay_third_attempt() {
        let policy = RetryPolicy::default();
        let delay = calculate_backoff_delay(3, &policy);
        assert_eq!(delay, Some(4000));
    }

    #[test]
    fn calculate_backoff_delay_caps_at_max() {
        let policy = RetryPolicy {
            initial_interval_ms: 1000,
            backoff_coefficient: 2.0,
            max_interval_ms: 10000,
            ..Default::default()
        };
        let delay = calculate_backoff_delay(10, &policy);
        assert_eq!(delay, Some(10000));
    }

    #[test]
    fn calculate_backoff_delay_zero_attempt_returns_none() {
        let policy = RetryPolicy::default();
        let delay = calculate_backoff_delay(0, &policy);
        assert_eq!(delay, None);
    }

    #[test]
    fn calculate_backoff_delay_linear_coefficient_one() {
        let policy = RetryPolicy {
            initial_interval_ms: 500,
            backoff_coefficient: 1.0,
            max_interval_ms: 60000,
            ..Default::default()
        };
        assert_eq!(calculate_backoff_delay(1, &policy), Some(500));
        assert_eq!(calculate_backoff_delay(2, &policy), Some(500));
        assert_eq!(calculate_backoff_delay(3, &policy), Some(500));
    }

    #[test]
    fn calculate_backoff_delay_fractional_coefficient() {
        let policy = RetryPolicy {
            initial_interval_ms: 1000,
            backoff_coefficient: 1.5,
            max_interval_ms: 60000,
            ..Default::default()
        };
        assert_eq!(calculate_backoff_delay(1, &policy), Some(1000));
        assert_eq!(calculate_backoff_delay(2, &policy), Some(1500));
        assert_eq!(calculate_backoff_delay(3, &policy), Some(2250));
    }

    // Heartbeat tests (pure helpers — no NATS required)
    // HeartbeatSender::send() requires a live NATS context (covered by integration tests).
    // Unit tests here cover pure helper constants.

    #[test]
    fn heartbeat_max_progress_bytes_constant_is_1kb() {
        assert_eq!(MAX_HEARTBEAT_PROGRESS_BYTES, 1024);
    }
}
