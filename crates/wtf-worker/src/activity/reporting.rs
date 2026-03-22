use async_nats::jetstream::Context;
use bytes::Bytes;
use wtf_common::{ActivityId, InstanceId, NamespaceId, WorkflowEvent, WtfError};
use wtf_storage::append_event;

/// Maximum size of a heartbeat progress string in bytes (1KB).
pub const MAX_HEARTBEAT_PROGRESS_BYTES: usize = 1024;

/// Report a successful activity result.
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
    tracing::debug!(%namespace, %instance_id, %activity_id, seq, duration_ms, "activity completed");
    Ok(seq)
}

/// Report a failed activity result.
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
    tracing::warn!(%namespace, %instance_id, %activity_id, %error, retries_exhausted, seq, "activity failed");
    Ok(seq)
}

/// Send a heartbeat for a running activity.
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
    tracing::debug!(%namespace, %instance_id, %activity_id, seq, progress_len = progress_bytes.len(), "heartbeat sent");
    Ok(seq)
}
