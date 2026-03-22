use ractor::{Actor, ActorRef, RpcReplyPort};
use wtf_common::{InstanceId, NamespaceId};
use crate::messages::{InstanceArguments, InstanceMetadata, OrchestratorMsg, StartError, WorkflowParadigm};
use crate::master::state::OrchestratorState;
use crate::instance::WorkflowInstance;

/// Handle a StartWorkflow message.
pub async fn handle_start_workflow(
    myself: ActorRef<OrchestratorMsg>,
    state: &mut OrchestratorState,
    namespace: NamespaceId,
    instance_id: InstanceId,
    workflow_type: String,
    paradigm: WorkflowParadigm,
    input: bytes::Bytes,
    reply: RpcReplyPort<Result<InstanceId, StartError>>,
) {
    if let Err(e) = validate_request(state, &instance_id) {
        let _ = reply.send(Err(e));
        return;
    }

    let args = build_args(state, namespace, instance_id, workflow_type, paradigm, input);
    let result = spawn_and_register(myself, state, args).await;
    let _ = reply.send(result);
}

fn validate_request(state: &OrchestratorState, id: &InstanceId) -> Result<(), StartError> {
    if !state.has_capacity() {
        return Err(StartError::AtCapacity {
            running: state.active_count(),
            max: state.config.max_instances,
        });
    }
    if state.active.contains_key(id) {
        return Err(StartError::AlreadyExists(id.clone()));
    }
    Ok(())
}

fn build_args(
    state: &OrchestratorState,
    ns: NamespaceId,
    id: InstanceId,
    wtype: String,
    paradigm: WorkflowParadigm,
    input: bytes::Bytes,
) -> InstanceArguments {
    InstanceArguments {
        namespace: ns,
        instance_id: id,
        workflow_type: wtype.clone(),
        paradigm,
        input,
        engine_node_id: state.config.engine_node_id.clone(),
        nats: state.config.nats.clone(),
        snapshot_db: state.config.snapshot_db.clone(),
        procedural_workflow: state.registry.get_procedural(&wtype),
        workflow_definition: state.registry.get_definition(&wtype),
    }
}

async fn spawn_and_register(
    myself: ActorRef<OrchestratorMsg>,
    state: &mut OrchestratorState,
    args: InstanceArguments,
) -> Result<InstanceId, StartError> {
    let id = args.instance_id.clone();
    let name = format!("wf-{}", id.as_str());
    let (actor_ref, _) = WorkflowInstance::spawn_linked(
        Some(name), WorkflowInstance, args.clone(), myself.into()
    ).await.map_err(|e| StartError::SpawnFailed(e.to_string()))?;

    persist_metadata(state, &args).await;
    state.register(id.clone(), actor_ref);
    Ok(id)
}

async fn persist_metadata(state: &OrchestratorState, args: &InstanceArguments) {
    let Some(nats) = &state.config.nats else { return };
    let js = nats.jetstream();
    let Ok(kv) = js.get_key_value(wtf_storage::bucket_names::INSTANCES).await else { return };

    let metadata = InstanceMetadata {
        namespace: args.namespace.clone(),
        instance_id: args.instance_id.clone(),
        workflow_type: args.workflow_type.clone(),
        paradigm: args.paradigm,
        engine_node_id: state.config.engine_node_id.clone(),
    };

    if let Ok(json) = serde_json::to_vec(&metadata) {
        let key = wtf_storage::instance_key(metadata.namespace.as_str(), &metadata.instance_id);
        let _ = kv.put(&key, json.into()).await;
    }
}
