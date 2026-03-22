#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepIndex(pub usize);

impl StepIndex {
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }

    #[must_use]
    pub fn try_from_usize(value: usize) -> Option<Self> {
        Some(Self(value))
    }
}

impl Default for StepIndex {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum TimeTravelError {
    #[error("history is empty")]
    NoHistory,
    #[error("step index {0} out of bounds for {1} steps")]
    InvalidStepIndex(usize, usize),
    #[error("run is still active")]
    RunStillActive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderedStep {
    pub index: StepIndex,
    pub node_id: uuid::Uuid,
    pub node_name: String,
    pub output: serde_json::Value,
}

impl OrderedStep {
    #[must_use]
    pub const fn new(
        index: StepIndex,
        node_id: uuid::Uuid,
        node_name: String,
        output: serde_json::Value,
    ) -> Self {
        Self {
            index,
            node_id,
            node_name,
            output,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffEntry {
    Added {
        key: String,
        value: serde_json::Value,
    },
    Removed {
        key: String,
    },
    Changed {
        key: String,
        old: serde_json::Value,
        new: serde_json::Value,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateDiff {
    pub entries: Vec<DiffEntry>,
}

impl StateDiff {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn compute(before: &serde_json::Value, after: &serde_json::Value) -> Self {
        let mut entries = Vec::new();

        let before_obj = match before {
            serde_json::Value::Object(map) => map,
            _ => return Self { entries },
        };
        let after_obj = match after {
            serde_json::Value::Object(map) => map,
            _ => return Self { entries },
        };

        for (key, new_value) in after_obj {
            match before_obj.get(key) {
                None => {
                    entries.push(DiffEntry::Added {
                        key: key.clone(),
                        value: new_value.clone(),
                    });
                }
                Some(old_value) if old_value != new_value => {
                    entries.push(DiffEntry::Changed {
                        key: key.clone(),
                        old: old_value.clone(),
                        new: new_value.clone(),
                    });
                }
                Some(_) => {}
            }
        }

        for (key, old_value) in before_obj {
            if !after_obj.contains_key(key) {
                entries.push(DiffEntry::Removed { key: key.clone() });
            }
        }

        Self { entries }
    }
}

#[must_use]
pub fn validate_step_index(index: usize, max_steps: usize) -> Result<StepIndex, TimeTravelError> {
    if max_steps == 0 {
        return Err(TimeTravelError::NoHistory);
    }
    if index >= max_steps {
        return Err(TimeTravelError::InvalidStepIndex(index, max_steps));
    }
    Ok(StepIndex::new(index))
}

#[must_use]
pub fn clamp_step_index(index: usize, max_steps: usize) -> StepIndex {
    StepIndex::new(index.min(max_steps.saturating_sub(1)))
}

#[must_use]
pub fn next_step(current: StepIndex, max_steps: usize) -> Option<StepIndex> {
    let next = current.get().checked_add(1)?;
    if next >= max_steps {
        return None;
    }
    Some(StepIndex::new(next))
}

#[must_use]
pub fn prev_step(current: StepIndex) -> Option<StepIndex> {
    current.get().checked_sub(1).map(StepIndex::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_zero_max_steps_when_validating_index_then_error() {
        let result = validate_step_index(0, 0);
        assert_eq!(result, Err(TimeTravelError::NoHistory));
    }

    #[test]
    fn given_index_beyond_max_when_validating_then_error() {
        let result = validate_step_index(5, 3);
        assert_eq!(result, Err(TimeTravelError::InvalidStepIndex(5, 3)));
    }

    #[test]
    fn given_valid_index_when_validating_then_ok() {
        let result = validate_step_index(2, 5);
        assert_eq!(result, Ok(StepIndex::new(2)));
    }

    #[test]
    fn given_current_at_max_when_next_then_none() {
        let current = StepIndex::new(4);
        let result = next_step(current, 5);
        assert_eq!(result, None);
    }

    #[test]
    fn given_current_at_zero_when_prev_then_none() {
        let current = StepIndex::new(0);
        let result = prev_step(current);
        assert_eq!(result, None);
    }

    #[test]
    fn given_current_in_middle_when_next_then_incremented() {
        let current = StepIndex::new(2);
        let result = next_step(current, 5);
        assert_eq!(result, Some(StepIndex::new(3)));
    }

    #[test]
    fn given_current_in_middle_when_prev_then_decremented() {
        let current = StepIndex::new(3);
        let result = prev_step(current);
        assert_eq!(result, Some(StepIndex::new(2)));
    }

    #[test]
    fn state_diff_detects_added_fields() {
        let before = serde_json::json!({});
        let after = serde_json::json!({"key": "value"});

        let diff = StateDiff::compute(&before, &after);

        assert_eq!(diff.entries.len(), 1);
        assert!(matches!(
            diff.entries[0],
            DiffEntry::Added { ref key, .. } if key == "key"
        ));
    }

    #[test]
    fn state_diff_detects_removed_fields() {
        let before = serde_json::json!({"key": "value"});
        let after = serde_json::json!({});

        let diff = StateDiff::compute(&before, &after);

        assert_eq!(diff.entries.len(), 1);
        assert!(matches!(
            diff.entries[0],
            DiffEntry::Removed { ref key } if key == "key"
        ));
    }

    #[test]
    fn state_diff_detects_changed_fields() {
        let before = serde_json::json!({"key": "old"});
        let after = serde_json::json!({"key": "new"});

        let diff = StateDiff::compute(&before, &after);

        assert_eq!(diff.entries.len(), 1);
        assert!(matches!(
            diff.entries[0],
            DiffEntry::Changed { ref key, .. } if key == "key"
        ));
    }

    #[test]
    fn state_diff_ignores_unchanged_fields() {
        let before = serde_json::json!({"key": "value"});
        let after = serde_json::json!({"key": "value"});

        let diff = StateDiff::compute(&before, &after);

        assert!(diff.entries.is_empty());
    }
}
