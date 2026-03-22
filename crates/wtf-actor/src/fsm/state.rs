use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use wtf_common::ActivityId;

/// In-memory state for an FSM workflow actor.
///
/// This is a pure cache of the JetStream event log. Every field is derivable
/// by replaying `WorkflowEvent` records from the stream (ADR-016).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsmActorState {
    /// Current FSM state name (e.g. `"Pending"`, `"Authorized"`).
    pub current_state: String,

    /// Set of JetStream sequence numbers already applied.
    /// Used to skip duplicate events during replay (idempotency guard — ADR-016).
    pub applied_seq: HashSet<u64>,

    /// Activities currently dispatched but not yet completed.
    /// Key: `ActivityId`. Value: activity type name.
    pub in_flight: HashMap<ActivityId, String>,

    /// Number of events processed since the last snapshot.
    pub events_since_snapshot: u32,
}

impl FsmActorState {
    /// Create a new FSM state starting in `initial_state`.
    #[must_use]
    pub fn new(initial_state: impl Into<String>) -> Self {
        Self {
            current_state: initial_state.into(),
            applied_seq: HashSet::new(),
            in_flight: HashMap::new(),
            events_since_snapshot: 0,
        }
    }
}
