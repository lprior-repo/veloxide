//! DAG event application logic.

use super::state::{DagActorState, NodeId};
use bytes::Bytes;
use wtf_common::WorkflowEvent;

/// Result of applying a single event to DAG state.
#[derive(Debug, Clone)]
pub enum DagApplyResult {
    AlreadyApplied,
    None,
    ActivityCompleted { node_id: NodeId, result: Bytes },
    ActivityFailed { node_id: NodeId },
}

/// Error applying an event.
#[derive(Debug, thiserror::Error)]
pub enum DagApplyError {
    #[error("activity_completed for unknown node: {0}")]
    UnknownNode(String),
}

/// Apply a single `WorkflowEvent` to the DAG actor state.
pub fn apply_event(
    state: &DagActorState,
    event: &WorkflowEvent,
    seq: u64,
) -> Result<(DagActorState, DagApplyResult), DagApplyError> {
    if state.applied_seq.contains(&seq) {
        return Ok((state.clone(), DagApplyResult::AlreadyApplied));
    }

    match event {
        WorkflowEvent::ActivityDispatched { activity_id, .. } => {
            let mut next = state.clone();
            next.in_flight.insert(NodeId::new(activity_id));
            next.applied_seq.insert(seq);
            next.events_since_snapshot += 1;
            Ok((next, DagApplyResult::None))
        }
        WorkflowEvent::ActivityCompleted {
            activity_id,
            result,
            ..
        } => handle_activity_completed(state, activity_id, result, seq),
        WorkflowEvent::ActivityFailed {
            activity_id,
            retries_exhausted,
            ..
        } => handle_activity_failed(state, activity_id, *retries_exhausted, seq),
        WorkflowEvent::SnapshotTaken { .. } => {
            let mut next = state.clone();
            next.applied_seq.insert(seq);
            next.events_since_snapshot = 0;
            Ok((next, DagApplyResult::None))
        }
        _ => {
            let mut next = state.clone();
            next.applied_seq.insert(seq);
            next.events_since_snapshot += 1;
            Ok((next, DagApplyResult::None))
        }
    }
}

fn handle_activity_completed(
    state: &DagActorState,
    activity_id: &str,
    result: &Bytes,
    seq: u64,
) -> Result<(DagActorState, DagApplyResult), DagApplyError> {
    let node_id = NodeId::new(activity_id);
    if !state.nodes.contains_key(&node_id) {
        return Err(DagApplyError::UnknownNode(activity_id.to_string()));
    }
    let mut next = state.clone();
    next.in_flight.remove(&node_id);
    next.completed.insert(node_id.clone(), result.clone());
    next.applied_seq.insert(seq);
    next.events_since_snapshot += 1;
    Ok((
        next,
        DagApplyResult::ActivityCompleted {
            node_id,
            result: result.clone(),
        },
    ))
}

fn handle_activity_failed(
    state: &DagActorState,
    activity_id: &str,
    retries_exhausted: bool,
    seq: u64,
) -> Result<(DagActorState, DagApplyResult), DagApplyError> {
    let node_id = NodeId::new(activity_id);
    let mut next = state.clone();
    if retries_exhausted {
        next.in_flight.remove(&node_id);
        next.failed.insert(node_id.clone());
        next.applied_seq.insert(seq);
        next.events_since_snapshot += 1;
        Ok((next, DagApplyResult::ActivityFailed { node_id }))
    } else {
        next.applied_seq.insert(seq);
        next.events_since_snapshot += 1;
        Ok((next, DagApplyResult::None))
    }
}
