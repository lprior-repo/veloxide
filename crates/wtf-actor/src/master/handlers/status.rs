use ractor::rpc::CallResult;
use wtf_common::InstanceId;
use crate::messages::{InstanceMsg, InstanceStatusSnapshot};
use crate::master::state::OrchestratorState;
use std::time::Duration;

const INSTANCE_CALL_TIMEOUT: Duration = Duration::from_millis(500);

pub async fn handle_get_status(
    state: &OrchestratorState,
    instance_id: &InstanceId,
) -> Option<InstanceStatusSnapshot> {
    let actor_ref = state.get(instance_id)?;
    match actor_ref.call(InstanceMsg::GetStatus, Some(INSTANCE_CALL_TIMEOUT)).await {
        Ok(CallResult::Success(snapshot)) => Some(snapshot),
        _ => None,
    }
}
