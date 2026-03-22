//! `wtf serve` — run the wtf-engine server.
//! Provision streams and buckets, then start the NATS JetStream context.

use anyhow::Context;
use std::path::PathBuf;
use wtf_storage::{connect, provision_kv_buckets, provision_streams, NatsClient, NatsConfig};

/// Configuration for the `serve` command.
#[derive(Debug, Clone)]
pub struct ServeConfig {
    pub port: u16,
    pub nats_url: String,
    pub embedded_nats: bool,
    pub data_dir: PathBuf,
    pub max_concurrent: usize,
}

impl From<ServeConfig> for NatsConfig {
    fn from(cfg: ServeConfig) -> Self {
        Self {
            urls: vec![cfg.nats_url],
            embedded: cfg.embedded_nats,
            ..Default::default()
        }
    }
}

/// Run the `serve` command.
///
/// Establishes NATS connection, provisions storage, and returns the NATS client.
pub async fn run_serve(config: ServeConfig) -> anyhow::Result<NatsClient> {
    let nats_config = NatsConfig::from(config);

    let nats = connect(&nats_config)
        .await
        .context("failed to connect to NATS")?;

    provision_storage(&nats).await?;

    Ok(nats)
}

async fn provision_storage(nats: &NatsClient) -> anyhow::Result<()> {
    provision_streams(nats.jetstream())
        .await
        .context("failed to provision JetStream streams")?;

    provision_kv_buckets(nats.jetstream())
        .await
        .context("failed to provision KV buckets")?;

    Ok(())
}
