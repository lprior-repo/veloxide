use std::collections::{HashMap, HashSet};
use wtf_common::EffectDeclaration;

/// FSM workflow definition (bead wtf-tzjw).
#[derive(Debug, Clone, Default)]
pub struct FsmDefinition {
    transitions: HashMap<(String, String), (String, Vec<EffectDeclaration>)>,
    terminal_states: HashSet<String>,
}

impl FsmDefinition {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare `state` as a terminal state (workflow ends when entered).
    pub fn add_terminal_state(&mut self, state: impl Into<String>) {
        self.terminal_states.insert(state.into());
    }

    /// Returns `true` if `state` is a declared terminal state.
    #[must_use]
    pub fn is_terminal(&self, state: &str) -> bool {
        self.terminal_states.contains(state)
    }

    pub fn add_transition(
        &mut self,
        from: impl Into<String>,
        event: impl Into<String>,
        to: impl Into<String>,
        effects: Vec<EffectDeclaration>,
    ) {
        self.transitions
            .insert((from.into(), event.into()), (to.into(), effects));
    }

    #[must_use]
    pub fn transition(
        &self,
        current_state: &str,
        event_name: &str,
    ) -> Option<(&str, &[EffectDeclaration])> {
        self.transitions
            .get(&(current_state.to_owned(), event_name.to_owned()))
            .map(|(to, effects)| (to.as_str(), effects.as_slice()))
    }
}
