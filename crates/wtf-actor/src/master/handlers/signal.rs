use crate::master::state::OrchestratorState;
use crate::messages::InstanceMsg;
use bytes::Bytes;
use ractor::RpcReplyPort;
use wtf_common::{InstanceId, WtfError};

/// Handle a Signal message.
pub fn handle_signal(
    state: &OrchestratorState,
    instance_id: InstanceId,
    signal_name: String,
    payload: Bytes,
    reply: RpcReplyPort<Result<(), WtfError>>,
) {
    match state.get(&instance_id) {
        Some(actor_ref) => {
            let _ = actor_ref.cast(InstanceMsg::InjectSignal {
                signal_name,
                payload,
                reply,
            });
        }
        None => {
            let _ = reply.send(Err(WtfError::instance_not_found(instance_id.as_str())));
        }
    }
}
