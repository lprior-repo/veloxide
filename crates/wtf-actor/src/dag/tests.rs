//! Tests for DAG paradigm.

use super::*;
use bytes::Bytes;
use std::collections::HashMap;
use wtf_common::WorkflowEvent;

fn linear_dag() -> HashMap<NodeId, DagNode> {
    let mut nodes = HashMap::new();
    nodes.insert(
        NodeId::new("A"),
        DagNode {
            activity_type: "task_a".into(),
            predecessors: vec![],
        },
    );
    nodes.insert(
        NodeId::new("B"),
        DagNode {
            activity_type: "task_b".into(),
            predecessors: vec![NodeId::new("A")],
        },
    );
    nodes.insert(
        NodeId::new("C"),
        DagNode {
            activity_type: "task_c".into(),
            predecessors: vec![NodeId::new("B")],
        },
    );
    nodes
}

fn parallel_dag() -> HashMap<NodeId, DagNode> {
    let mut nodes = HashMap::new();
    nodes.insert(
        NodeId::new("A"),
        DagNode {
            activity_type: "task_a".into(),
            predecessors: vec![],
        },
    );
    nodes.insert(
        NodeId::new("B"),
        DagNode {
            activity_type: "task_b".into(),
            predecessors: vec![],
        },
    );
    nodes.insert(
        NodeId::new("C"),
        DagNode {
            activity_type: "task_c".into(),
            predecessors: vec![NodeId::new("A"), NodeId::new("B")],
        },
    );
    nodes
}

fn completed_event(id: &str) -> WorkflowEvent {
    WorkflowEvent::ActivityCompleted {
        activity_id: id.into(),
        result: Bytes::from_static(b"ok"),
        duration_ms: 10,
    }
}

fn dispatched_event(id: &str) -> WorkflowEvent {
    WorkflowEvent::ActivityDispatched {
        activity_id: id.into(),
        activity_type: "task".into(),
        payload: Bytes::new(),
        retry_policy: wtf_common::RetryPolicy::default(),
        attempt: 1,
    }
}

fn failed_event(id: &str, exhausted: bool) -> WorkflowEvent {
    WorkflowEvent::ActivityFailed {
        activity_id: id.into(),
        error: "boom".into(),
        retries_exhausted: exhausted,
    }
}

#[test]
fn root_nodes_ready_on_empty_state() {
    let state = DagActorState::new(linear_dag());
    let ready = ready_nodes(&state);
    assert_eq!(ready, vec![NodeId::new("A")]);
}

#[test]
fn parallel_roots_both_ready() {
    let state = DagActorState::new(parallel_dag());
    let ready = ready_nodes(&state);
    assert_eq!(ready, vec![NodeId::new("A"), NodeId::new("B")]);
}

#[test]
fn in_flight_node_not_ready() {
    let state = DagActorState::new(linear_dag());
    let event = dispatched_event("A");
    let (s1, _) = apply_event(&state, &event, 1).expect("apply");
    let ready = ready_nodes(&s1);
    assert!(ready.is_empty());
}

#[test]
fn completed_unblocks_successor() {
    let state = DagActorState::new(linear_dag());
    let (s1, _) = apply_event(&state, &dispatched_event("A"), 1).expect("dispatch A");
    let (s2, _) = apply_event(&s1, &completed_event("A"), 2).expect("complete A");
    let ready = ready_nodes(&s2);
    assert_eq!(ready, vec![NodeId::new("B")]);
}

#[test]
fn duplicate_seq_returns_already_applied() {
    let state = DagActorState::new(linear_dag());
    let (s1, _) = apply_event(&state, &completed_event("A"), 1).expect("first");
    let (_, result) = apply_event(&s1, &completed_event("A"), 1).expect("duplicate");
    assert!(matches!(result, DagApplyResult::AlreadyApplied));
}

#[test]
fn activity_failed_exhausted_adds_to_failed() {
    let state = DagActorState::new(linear_dag());
    let (s1, _) = apply_event(&state, &dispatched_event("A"), 1).expect("dispatch");
    let (s2, result) = apply_event(&s1, &failed_event("A", true), 2).expect("fail exhausted");
    assert!(matches!(result, DagApplyResult::ActivityFailed { .. }));
    assert!(s2.failed.contains(&NodeId::new("A")));
}

#[test]
fn is_succeeded_when_all_complete() {
    let state = DagActorState::new(linear_dag());
    let (s1, _) = apply_event(&state, &completed_event("A"), 1).expect("A");
    let (s2, _) = apply_event(&s1, &completed_event("B"), 2).expect("B");
    let (s3, _) = apply_event(&s2, &completed_event("C"), 3).expect("C");
    assert!(is_succeeded(&s3));
}
