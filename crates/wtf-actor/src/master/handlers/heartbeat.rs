use crate::instance::WorkflowInstance;
use crate::master::state::OrchestratorState;
use crate::messages::{InstanceSeed, OrchestratorMsg};
use ractor::{Actor, ActorRef};
use std::collections::HashSet;
use tokio::sync::Mutex;
use std::sync::OnceLock;
use wtf_common::{InstanceId, InstanceMetadata};

/// Acquire the in-flight guard for recovery deduplication.
/// Uses tokio::sync::Mutex to avoid blocking the async runtime.
async fn acquire_in_flight_guard() -> tokio::sync::MutexGuard<'static, HashSet<String>> {
    static IN_FLIGHT: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    // Tokio mutexes do not poison - the lock() returns MutexGuard directly.
    IN_FLIGHT
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .await
}

/// Check whether heartbeat-expired recovery should proceed for this instance.
/// Returns `Some(in_flight_key)` if recovery should proceed, `None` if skipped.
async fn check_recovery_preconditions(
    state: &OrchestratorState,
    instance_id: &InstanceId,
) -> Option<String> {
    if state.active.contains_key(instance_id) {
        tracing::debug!(instance_id = %instance_id, "heartbeat expired but instance still active; skipping recovery");
        return None;
    }

    let in_flight_key = instance_id.to_string();
    let mut guard = acquire_in_flight_guard().await;
    if !guard.insert(in_flight_key.clone()) {
        tracing::debug!(instance_id = %instance_id, "recovery already in-flight; skipping duplicate trigger");
        return None;
    }
    Some(in_flight_key)
}

/// Attempt to spawn a recovered instance from persisted metadata.
async fn attempt_recovery(
    myself: &ActorRef<OrchestratorMsg>,
    state: &mut OrchestratorState,
    instance_id: &InstanceId,
    in_flight_key: &str,
) {
    let Some(metadata) = fetch_metadata(state, instance_id).await else {
        tracing::warn!(instance_id = %instance_id, "instance metadata missing; recovery skipped");
        acquire_in_flight_guard().await.remove(in_flight_key);
        return;
    };

    let args = build_recovery_args(state, &metadata);
    let name = format!("wf-recovered-{}", instance_id.as_str());
    let myself = myself.clone();

    if let Ok((actor_ref, _)) =
        WorkflowInstance::spawn_linked(Some(name), WorkflowInstance, args, myself.into()).await
    {
        state.register(instance_id.clone(), actor_ref);
    }

    // Always clean up the in-flight key, even if spawn failed.
    acquire_in_flight_guard().await.remove(in_flight_key);
}

pub async fn handle_heartbeat_expired(
    myself: ActorRef<OrchestratorMsg>,
    state: &mut OrchestratorState,
    instance_id: InstanceId,
) {
    let Some(in_flight_key) = check_recovery_preconditions(state, &instance_id).await else {
        return;
    };
    attempt_recovery(&myself, state, &instance_id, &in_flight_key).await;
}

async fn fetch_metadata(state: &OrchestratorState, id: &InstanceId) -> Option<InstanceMetadata> {
    if let Some(store) = &state.config.state_store {
        store.get_instance_metadata(id).await.ok().flatten()
    } else {
        None
    }
}

fn build_recovery_args(
    state: &OrchestratorState,
    m: &InstanceMetadata,
) -> crate::messages::InstanceArguments {
    let seed = InstanceSeed {
        namespace: m.namespace.clone(),
        instance_id: m.instance_id.clone(),
        workflow_type: m.workflow_type.clone(),
        paradigm: m.paradigm,
        input: bytes::Bytes::new(),
    };
    state.build_instance_args(seed)
}
