use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use wtf_common::{InstanceId, NamespaceId, TimerId, WtfError};

/// A pending timer stored in the `wtf-timers` KV bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerRecord {
    pub timer_id: TimerId,
    pub namespace: NamespaceId,
    pub instance_id: InstanceId,
    pub fire_at: DateTime<Utc>,
}

impl TimerRecord {
    pub fn to_msgpack(&self) -> Result<Bytes, WtfError> {
        rmp_serde::to_vec_named(self)
            .map(Bytes::from)
            .map_err(|e| WtfError::nats_publish(format!("serialize TimerRecord: {e}")))
    }

    pub fn from_msgpack(bytes: &[u8]) -> Result<Self, WtfError> {
        rmp_serde::from_slice(bytes)
            .map_err(|e| WtfError::nats_publish(format!("deserialize TimerRecord: {e}")))
    }

    #[must_use]
    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        self.fire_at <= now
    }
}
