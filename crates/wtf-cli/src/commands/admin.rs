//! `wtf admin rebuild-views` — rebuild materialized views from JetStream event log.

use anyhow::Context;
use async_nats::jetstream::consumer::push::Config as PushConfig;
use async_nats::jetstream::Context as JetStreamContext;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;
use std::time::Instant;
use wtf_common::{NamespaceId, WorkflowEvent, WtfError};
use wtf_storage::kv::{instance_key, provision_kv_buckets, KvStores};
use wtf_storage::nats::{connect, NatsConfig};

#[derive(Debug, Clone, Deserialize)]
pub struct RebuildViewsConfig {
    pub view: Option<String>,
    pub namespace: Option<String>,
    pub show_progress: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RebuildStats {
    pub instances_rebuilt: u64,
    pub timers_rebuilt: u64,
    pub definitions_rebuilt: u64,
    pub events_processed: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewName {
    Instances,
    Timers,
    Definitions,
    Heartbeats,
    All,
}

impl ViewName {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "instances" => Some(Self::Instances),
            "timers" => Some(Self::Timers),
            "definitions" => Some(Self::Definitions),
            "heartbeats" => Some(Self::Heartbeats),
            _ => None,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Instances, Self::Timers, Self::Definitions]
    }
}

impl FromStr for ViewName {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("invalid view name: {}", s))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceView {
    pub instance_id: String,
    pub workflow_type: String,
    pub status: String,
    pub current_state: Option<String>,
    pub last_event_seq: u64,
    pub updated_at: String,
}

#[derive(Debug)]
struct DiscoveredInstance {
    namespace: NamespaceId,
    instance_id: String,
}

pub async fn run_rebuild_views(
    config: RebuildViewsConfig,
) -> anyhow::Result<std::process::ExitCode> {
    if config.dry_run {
        return run_dry_run(&config);
    }

    let nats_config = NatsConfig::default();
    let nats_client = connect(&nats_config)
        .await
        .context("failed to connect to NATS")?;

    if config.show_progress {
        println!("Connecting to NATS...");
    }

    let stores = provision_kv_buckets(nats_client.jetstream())
        .await
        .context("failed to provision KV buckets")?;

    if config.show_progress {
        println!("KV buckets provisioned");
    }

    let view_filter = config.view.as_ref().map(|v| parse_view_name(v));

    let start = Instant::now();
    let stats = rebuild_views(
        nats_client.jetstream(),
        &stores,
        &config.namespace,
        view_filter.as_ref(),
        config.show_progress,
    )
    .await
    .context("rebuild failed")?;

    print_rebuild_summary(stats, start.elapsed());
    Ok(std::process::ExitCode::SUCCESS)
}

fn run_dry_run(config: &RebuildViewsConfig) -> anyhow::Result<std::process::ExitCode> {
    println!("[dry-run] Would rebuild views");
    if let Some(ref v) = config.view {
        println!("[dry-run]   view: {}", v);
    }
    if let Some(ref ns) = config.namespace {
        println!("[dry-run]   namespace: {}", ns);
    }
    Ok(std::process::ExitCode::SUCCESS)
}

fn parse_view_name(v: &str) -> ViewName {
    ViewName::parse(v).unwrap_or_else(|| {
        eprintln!(
            "error: invalid view '{}'. Valid views: instances, timers, definitions, heartbeats",
            v
        );
        std::process::exit(1);
    })
}

fn print_rebuild_summary(stats: RebuildStats, duration: std::time::Duration) {
    println!();
    println!("Rebuild complete:");
    println!("  instances: {}", stats.instances_rebuilt);
    println!("  timers: {}", stats.timers_rebuilt);
    println!("  definitions: {}", stats.definitions_rebuilt);
    println!("  events processed: {}", stats.events_processed);
    println!("  duration: {}ms", duration.as_millis());
}

async fn rebuild_views(
    js: &JetStreamContext,
    stores: &KvStores,
    namespace_filter: &Option<String>,
    view_filter: Option<&ViewName>,
    show_progress: bool,
) -> Result<RebuildStats, WtfError> {
    let instances = discover_instances(js, namespace_filter, show_progress).await?;

    let should_rebuild_instances =
        view_filter.map_or(true, |v| matches!(v, ViewName::Instances | ViewName::All));
    let should_rebuild_timers =
        view_filter.map_or(true, |v| matches!(v, ViewName::Timers | ViewName::All));

    let total = instances.len() as u64;
    let pb = if show_progress {
        Some(indicatif::ProgressBar::new(total))
    } else {
        None
    };

    let mut stats = RebuildStats {
        instances_rebuilt: 0,
        timers_rebuilt: 0,
        definitions_rebuilt: 0,
        events_processed: 0,
        duration_ms: 0,
    };

    for instance in instances {
        if should_rebuild_instances {
            match rebuild_instance(js, stores, &instance, view_filter).await {
                Ok((events_count, timers_count, defs_count)) => {
                    stats.events_processed += events_count;
                    stats.timers_rebuilt += timers_count;
                    stats.definitions_rebuilt += defs_count;
                    stats.instances_rebuilt += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        namespace = %instance.namespace,
                        instance_id = %instance.instance_id,
                        "failed to rebuild instance: {}", e
                    );
                }
            }
        }

        if let Some(ref pb) = pb {
            pb.inc(1);
        }
    }

    if let Some(pb) = pb {
        pb.finish();
    }

    Ok(stats)
}

async fn discover_instances(
    js: &JetStreamContext,
    namespace_filter: &Option<String>,
    show_progress: bool,
) -> Result<Vec<DiscoveredInstance>, WtfError> {
    let stream = js
        .get_stream("wtf-events")
        .await
        .map_err(|e| WtfError::nats_publish(format!("get wtf-events stream: {e}")))?;

    let filter_subject = match namespace_filter {
        Some(ns) => format!("wtf.log.{}.>", ns),
        None => "wtf.log.>".to_string(),
    };

    let consumer = stream
        .create_consumer(PushConfig {
            name: Some("rebuild-views-discover".to_string()),
            filter_subject: filter_subject.clone(),
            deliver_subject: format!(
                "_INBOX.wtf.rebuild.discover.{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ),
            ..Default::default()
        })
        .await
        .map_err(|e| WtfError::nats_publish(format!("create discovery consumer: {e}")))?;

    let mut messages = consumer
        .messages()
        .await
        .map_err(|e| WtfError::nats_publish(format!("get consumer messages: {e}")))?;

    let mut seen: HashSet<String> = HashSet::new();
    let mut instances = Vec::new();

    if show_progress {
        println!("Discovering instances...");
    }

    while let Some(msg_result) = messages.next().await {
        match msg_result {
            Ok(msg) => {
                let subject = msg.subject.to_string();
                if seen.insert(subject.clone()) {
                    if let Some(instance) = parse_instance_from_subject(&subject) {
                        instances.push(instance);
                    }
                }
                let _ = msg.ack().await;
            }
            Err(e) => {
                tracing::warn!("error receiving discovery message: {}", e);
            }
        }
    }

    Ok(instances)
}

fn parse_instance_from_subject(subject: &str) -> Option<DiscoveredInstance> {
    let parts: Vec<&str> = subject.split('.').collect();
    if parts.len() >= 4 && parts[0] == "wtf" && parts[1] == "log" {
        let namespace = NamespaceId::new(parts[2].to_string());
        let instance_id = parts[3..].join(".");
        Some(DiscoveredInstance {
            namespace,
            instance_id,
        })
    } else {
        None
    }
}

async fn rebuild_instance(
    js: &JetStreamContext,
    stores: &KvStores,
    instance: &DiscoveredInstance,
    view_filter: Option<&ViewName>,
) -> Result<(u64, u64, u64), WtfError> {
    let mut events_processed = 0u64;
    let mut timers_rebuilt = 0u64;
    let mut definitions_rebuilt = 0u64;

    let mut current_status = "running".to_string();
    let mut current_state: Option<String> = None;
    let mut workflow_type = String::new();
    let mut last_seq = 0u64;

    let stream = js
        .get_stream("wtf-events")
        .await
        .map_err(|e| WtfError::nats_publish(format!("get stream: {e}")))?;

    let subject = format!(
        "wtf.log.{}.{}",
        instance.namespace.as_str(),
        instance.instance_id
    );

    let deliver_subject = format!(
        "_INBOX.wtf.rebuild.{}.{}",
        instance.instance_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );

    let consumer = stream
        .create_consumer(PushConfig {
            name: Some(format!("rebuild-{}", instance.instance_id)),
            filter_subject: subject,
            deliver_subject,
            ..Default::default()
        })
        .await
        .map_err(|e| WtfError::nats_publish(format!("create instance consumer: {e}")))?;

    let mut messages = consumer
        .messages()
        .await
        .map_err(|e| WtfError::nats_publish(format!("get messages: {e}")))?;

    while let Some(msg_result) = messages.next().await {
        match msg_result {
            Ok(msg) => {
                let info = msg
                    .info()
                    .map_err(|e| WtfError::nats_publish(format!("msg info: {e}")))?;
                last_seq = info.stream_sequence;

                let event: WorkflowEvent = rmp_serde::from_slice(&msg.payload)
                    .map_err(|e| WtfError::nats_publish(format!("decode event: {e}")))?;

                apply_event_to_state(
                    &event,
                    &mut current_status,
                    &mut current_state,
                    &mut workflow_type,
                    &mut timers_rebuilt,
                    &mut definitions_rebuilt,
                );

                events_processed += 1;
                let _ = msg.ack().await;
            }
            Err(e) => {
                tracing::warn!(
                    "error receiving message for {}: {}",
                    instance.instance_id,
                    e
                );
            }
        }
    }

    let should_write_instances =
        view_filter.map_or(true, |v| matches!(v, ViewName::Instances | ViewName::All));

    if should_write_instances && last_seq > 0 {
        let view = InstanceView {
            instance_id: instance.instance_id.clone(),
            workflow_type,
            status: current_status,
            current_state,
            last_event_seq: last_seq,
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let key = instance_key(
            instance.namespace.as_str(),
            &wtf_common::InstanceId::new(&instance.instance_id),
        );
        let value = serde_json::to_vec(&view)
            .map_err(|e| WtfError::nats_publish(format!("serialize view: {e}")))?;

        stores
            .instances
            .put(&key, value.into())
            .await
            .map_err(|e| WtfError::nats_publish(format!("write instance view: {e}")))?;
    }

    Ok((events_processed, timers_rebuilt, definitions_rebuilt))
}

fn apply_event_to_state(
    event: &WorkflowEvent,
    status: &mut String,
    current_state: &mut Option<String>,
    workflow_type: &mut String,
    timers_rebuilt: &mut u64,
    definitions_rebuilt: &mut u64,
) {
    match event {
        WorkflowEvent::InstanceStarted {
            instance_id: _,
            workflow_type: wf_type,
            input: _,
        } => {
            *workflow_type = wf_type.clone();
            *status = "started".to_string();
            *definitions_rebuilt += 1;
        }
        WorkflowEvent::InstanceCompleted { output: _ } => {
            *status = "completed".to_string();
        }
        WorkflowEvent::InstanceFailed { error: _ } => {
            *status = "failed".to_string();
        }
        WorkflowEvent::InstanceCancelled { reason: _ } => {
            *status = "cancelled".to_string();
        }
        WorkflowEvent::TransitionApplied {
            from_state: _,
            event_name: _,
            to_state,
            effects: _,
        } => {
            *current_state = Some(to_state.clone());
        }
        WorkflowEvent::TimerScheduled {
            timer_id: _,
            fire_at: _,
        } => {
            *timers_rebuilt += 1;
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_name_parse_instances() {
        assert_eq!(ViewName::parse("instances"), Some(ViewName::Instances));
        assert_eq!(ViewName::parse("INSTANCES"), Some(ViewName::Instances));
    }

    #[test]
    fn view_name_parse_invalid() {
        assert_eq!(ViewName::parse("invalid"), None);
    }

    #[test]
    fn view_name_all_returns_three() {
        let all = ViewName::all();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn rebuild_stats_default_is_zero() {
        let stats = RebuildStats {
            instances_rebuilt: 0,
            timers_rebuilt: 0,
            definitions_rebuilt: 0,
            events_processed: 0,
            duration_ms: 0,
        };
        assert_eq!(stats.instances_rebuilt, 0);
    }

    #[test]
    fn parse_instance_from_subject_valid() {
        let result = parse_instance_from_subject("wtf.log.payments.01ARZ");
        assert!(result.is_some());
        let inst = result.unwrap();
        assert_eq!(inst.namespace.as_str(), "payments");
        assert_eq!(inst.instance_id, "01ARZ");
    }

    #[test]
    fn parse_instance_from_subject_with_dots_in_id() {
        let result = parse_instance_from_subject("wtf.log.payments.01ARZ.instance");
        assert!(result.is_some());
        let inst = result.unwrap();
        assert_eq!(inst.namespace.as_str(), "payments");
        assert_eq!(inst.instance_id, "01ARZ.instance");
    }

    #[test]
    fn parse_instance_from_subject_invalid() {
        assert!(parse_instance_from_subject("wtf.events.payments.01ARZ").is_none());
        assert!(parse_instance_from_subject("wtf.log.payments").is_none());
        assert!(parse_instance_from_subject("invalid").is_none());
    }

    #[test]
    fn apply_event_updates_status() {
        let event = WorkflowEvent::InstanceStarted {
            instance_id: "01ARZ".to_string(),
            workflow_type: "checkout".to_string(),
            input: bytes::Bytes::new(),
        };

        let mut status = "running".to_string();
        let mut current_state = None;
        let mut workflow_type = String::new();
        let mut timers = 0u64;
        let mut defs = 0u64;

        apply_event_to_state(
            &event,
            &mut status,
            &mut current_state,
            &mut workflow_type,
            &mut timers,
            &mut defs,
        );

        assert_eq!(status, "started");
        assert_eq!(workflow_type, "checkout");
        assert_eq!(defs, 1);
    }

    #[test]
    fn apply_event_transition_applied_updates_state() {
        let event = WorkflowEvent::TransitionApplied {
            from_state: "Pending".to_string(),
            event_name: "Authorize".to_string(),
            to_state: "Authorized".to_string(),
            effects: vec![],
        };

        let mut status = "running".to_string();
        let mut current_state = None;
        let mut workflow_type = String::new();
        let mut timers = 0u64;
        let mut defs = 0u64;

        apply_event_to_state(
            &event,
            &mut status,
            &mut current_state,
            &mut workflow_type,
            &mut timers,
            &mut defs,
        );

        assert_eq!(current_state, Some("Authorized".to_string()));
    }
}
