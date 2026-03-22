use crate::messages::InstanceStatusSnapshot;
use crate::master::state::OrchestratorState;
use crate::master::handlers::status::handle_get_status;

pub async fn handle_list_active(
    state: &OrchestratorState,
) -> Vec<InstanceStatusSnapshot> {
    let mut snapshots = Vec::with_capacity(state.active.len());
    for id in state.active.keys() {
        if let Some(snapshot) = handle_get_status(state, id).await {
            snapshots.push(snapshot);
        }
    }
    snapshots
}
