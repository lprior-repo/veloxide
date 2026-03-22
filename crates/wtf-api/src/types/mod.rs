//! HTTP API request/response types (ADR-012).

pub mod errors;
pub mod newtypes;
pub mod requests;
pub mod responses;

pub use errors::*;
pub use newtypes::*;
pub use requests::*;
pub use responses::*;

use serde::{Deserialize, Serialize};

/// Journal entry type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JournalEntryType {
    Run,
    Wait,
}

/// Journal entry for workflow history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub seq: u32,
    #[serde(flatten)]
    pub entry_type: JournalEntryType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fire_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

pub(crate) fn is_retryable_error(error: &str) -> bool {
    matches!(error, "at_capacity")
}

pub(crate) fn is_sorted<T: PartialOrd + Clone>(mut iter: impl Iterator<Item = T>) -> bool {
    let mut prev = match iter.next() {
        Some(v) => v,
        None => return true,
    };
    iter.all(|curr| {
        let result = prev <= curr;
        prev = curr;
        result
    })
}

#[cfg(test)]
mod tests;
