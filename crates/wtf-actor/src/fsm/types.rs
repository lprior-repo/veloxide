use super::state::FsmActorState;
use bytes::Bytes;
use wtf_common::{EffectDeclaration, WorkflowEvent};

/// Which phase the actor is in — determines whether effects are executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionPhase {
    /// Replaying the event log — effects are skipped (they already happened).
    Replay,
    /// Processing new events in real-time — effects must be executed.
    Live,
}

/// Result of applying a single event to FSM state.
#[derive(Debug, Clone)]
pub enum ApplyResult {
    /// Event was already applied (duplicate delivery) — state unchanged.
    AlreadyApplied,
    /// No effect to execute (informational event).
    None,
    /// Effects to execute in Live Phase (from `TransitionApplied`).
    Effects(Vec<EffectDeclaration>),
    /// Activity completed — caller should deliver result to pending waiter.
    ActivityResult(String, Bytes),
}

/// Error applying an event.
#[derive(Debug, thiserror::Error)]
pub enum ApplyError {
    #[error("event type not applicable to FSM actor: {0}")]
    UnexpectedEvent(String),
}

/// Output of [`plan_fsm_signal`].
#[derive(Debug, Clone)]
pub struct FsmTransitionPlan {
    pub transition_event: WorkflowEvent,
    pub next_state: FsmActorState,
}
