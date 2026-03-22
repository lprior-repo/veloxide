use ractor::rpc::CallResult;
use ractor::RpcReplyPort;
use wtf_common::InstanceId;
use crate::messages::{InstanceMsg, TerminateError};
use crate::master::state::OrchestratorState;
use std::time::Duration;

const INSTANCE_CALL_TIMEOUT: Duration = Duration::from_millis(500);

pub async fn handle_terminate(
    state: &mut OrchestratorState,
    instance_id: InstanceId,
    reason: String,
    reply: RpcReplyPort<Result<(), TerminateError>>,
) {
    match state.get(&instance_id) {
        None => { let _ = reply.send(Err(TerminateError::NotFound(instance_id))); }
        Some(actor_ref) => {
            let res = call_cancel(actor_ref, reason).await;
            let _ = reply.send(res);
        }
    }
}

async fn call_cancel(
    actor_ref: &ractor::ActorRef<InstanceMsg>,
    reason: String,
) -> Result<(), TerminateError> {
    let call_result = actor_ref
        .call(
            |tx| InstanceMsg::Cancel { reason, reply: tx },
            Some(INSTANCE_CALL_TIMEOUT),
        )
        .await;

    match call_result {
        Ok(CallResult::Success(inner)) => inner.map_err(|e: wtf_common::WtfError| TerminateError::Failed(e.to_string())),
        Ok(CallResult::Timeout) => Err(TerminateError::Failed("cancel timed out".into())),
        Ok(CallResult::SenderError) => Err(TerminateError::Failed("actor dropped reply".into())),
        Err(e) => Err(TerminateError::Failed(format!("send failed: {e}"))),
    }
}
