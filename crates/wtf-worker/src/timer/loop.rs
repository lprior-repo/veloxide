use std::time::Duration;
use async_nats::jetstream::kv::Store;
use async_nats::jetstream::Context;
use chrono::Utc;
use futures::StreamExt;
use wtf_common::WtfError;
use super::record::TimerRecord;
use super::ops::fire_timer;

pub const TIMER_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Run the timer firing loop.
pub async fn run_timer_loop(
    js: Context,
    timers: Store,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<(), WtfError> {
    tracing::info!("timer loop started");
    let mut interval = tokio::time::interval(TIMER_POLL_INTERVAL);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = poll_and_fire(&js, &timers).await {
                    tracing::error!(error = %e, "timer poll error");
                }
            }
            result = shutdown_rx.changed() => {
                if result.is_ok() || result.is_err() {
                    tracing::info!("timer loop shutting down");
                    break;
                }
            }
        }
    }

    Ok(())
}

async fn poll_and_fire(js: &Context, timers: &Store) -> Result<(), WtfError> {
    let now = Utc::now();
    let mut keys = timers
        .keys()
        .await
        .map_err(|e| WtfError::nats_publish(format!("list timer keys: {e}")))?;

    while let Some(key_result) = keys.next().await {
        match key_result {
            Err(e) => tracing::warn!(error = %e, "error iterating timer keys"),
            Ok(key) => {
                match timers.get(&key).await {
                    Err(e) => tracing::warn!(key = %key, error = %e, "failed to get timer entry"),
                    Ok(None) => {}
                    Ok(Some(value)) => match TimerRecord::from_msgpack(&value) {
                        Err(e) => tracing::warn!(key = %key, error = %e, "failed to deserialize timer record — skipping"),
                        Ok(record) => {
                            if record.is_due(now) {
                                if let Err(e) = fire_timer(js, timers, &record).await {
                                    tracing::error!(timer_id = %record.timer_id, error = %e, "failed to fire timer");
                                }
                            }
                        }
                    },
                }
            }
        }
    }

    Ok(())
}
