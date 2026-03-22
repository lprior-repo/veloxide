use ractor::{Actor, ActorRef};
use wtf_common::{InstanceId, InstanceMetadata};
use crate::messages::{InstanceArguments, OrchestratorMsg};
use crate::master::state::OrchestratorState;
use crate::instance::WorkflowInstance;

pub async fn handle_heartbeat_expired(
    myself: ActorRef<OrchestratorMsg>,
    state: &mut OrchestratorState,
    instance_id: InstanceId,
) {
    if state.active.contains_key(&instance_id) { return; }

    let Some(metadata) = fetch_metadata(state, &instance_id).await else { return };

    let args = build_recovery_args(state, &metadata);
    let name = format!("wf-recovered-{}", instance_id.as_str());

    if let Ok((actor_ref, _)) = WorkflowInstance::spawn_linked(
        Some(name), WorkflowInstance, args, myself.into()
    ).await {
        state.register(instance_id, actor_ref);
    }
}

async fn fetch_metadata(state: &OrchestratorState, id: &InstanceId) -> Option<InstanceMetadata> {
    if let Some(store) = &state.config.state_store {
        store.get_instance_metadata(id).await.ok().flatten()
    } else {
        None
    }
}

fn build_recovery_args(state: &OrchestratorState, m: &InstanceMetadata) -> InstanceArguments {
    InstanceArguments {
        namespace: m.namespace.clone(),
        instance_id: m.instance_id.clone(),
        workflow_type: m.workflow_type.clone(),
        paradigm: m.paradigm,
        input: bytes::Bytes::new(),
        engine_node_id: state.config.engine_node_id.clone(),
        event_store: state.config.event_store.clone(),
        state_store: state.config.state_store.clone(),
        task_queue: state.config.task_queue.clone(),
        snapshot_db: state.config.snapshot_db.clone(),
        procedural_workflow: state.registry.get_procedural(&m.workflow_type),
        workflow_definition: state.registry.get_definition(&m.workflow_type),
    }
}
