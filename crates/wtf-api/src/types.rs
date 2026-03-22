#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! types.rs - HTTP API request/response types
//!
//! Per ADR-012, these types define the HTTP API contract.
//! Implements Data→Calc→Actions pattern with compile-time validation.

use std::num::NonZeroU64;

use chrono::{DateTime, Utc};
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

// ============================================================================
// NEW TYPES (Data Layer)
// ============================================================================

/// Compile-time enforcement for workflow_name pattern `[a-z][a-z0-9_]*`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorkflowName(String);

impl WorkflowName {
    pub fn new(s: impl AsRef<str>) -> Result<Self, ParseError> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(ParseError::EmptyWorkflowName);
        }
        if !workflow_name_regex().is_match(s) {
            return Err(ParseError::InvalidWorkflowNameFormat);
        }
        Ok(Self(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for WorkflowName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for WorkflowName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).map_err(|e| D::Error::custom(e.to_string()))
    }
}

// Each regex helper uses #[allow] because the literals are compile-time constants
// that are always valid — the unwrap can never fail.
#[allow(clippy::unwrap_used)]
fn workflow_name_regex() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"^[a-z][a-z0-9_]*$").unwrap())
}

/// Compile-time enforcement for signal_name pattern `[a-z][a-z0-9_]+`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SignalName(String);

impl SignalName {
    pub fn new(s: impl AsRef<str>) -> Result<Self, ParseError> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(ParseError::EmptySignalName);
        }
        if !signal_name_regex().is_match(s) {
            return Err(ParseError::InvalidSignalNameFormat);
        }
        Ok(Self(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for SignalName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SignalName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).map_err(|e| D::Error::custom(e.to_string()))
    }
}

#[allow(clippy::unwrap_used)]
fn signal_name_regex() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"^[a-z][a-z0-9_]+$").unwrap())
}

/// Compile-time enforcement for ULID format (26 chars, Crockford base32)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvocationId(String);

impl InvocationId {
    pub fn from_str(s: impl AsRef<str>) -> Result<Self, ParseError> {
        let s = s.as_ref();
        if s.len() != 26 {
            return Err(ParseError::InvalidUlidFormat);
        }
        if !ulid_regex().is_match(s) {
            return Err(ParseError::InvalidUlidFormat);
        }
        Ok(Self(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for InvocationId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for InvocationId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|e| D::Error::custom(e.to_string()))
    }
}

#[allow(clippy::unwrap_used)]
fn ulid_regex() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"^[0-9A-HJKMNP-TV-Z]{26}$").unwrap())
}

/// Compile-time enforcement for retry_after_seconds > 0
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetryAfterSeconds(NonZeroU64);

impl RetryAfterSeconds {
    pub fn new(seconds: u64) -> Result<Self, ValidationError> {
        NonZeroU64::new(seconds)
            .map(Self)
            .ok_or(ValidationError::InvalidRetryAfterSeconds)
    }

    pub fn get(&self) -> u64 {
        self.0.get()
    }
}

impl Serialize for RetryAfterSeconds {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u64(self.0.get())
    }
}

impl<'de> Deserialize<'de> for RetryAfterSeconds {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = u64::deserialize(deserializer)?;
        Self::new(s).map_err(|e| D::Error::custom(e.to_string()))
    }
}

/// RFC3339 timestamp wrapper
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timestamp(String);

impl Timestamp {
    pub fn new(s: impl AsRef<str>) -> Result<Self, ParseError> {
        let s = s.as_ref();
        DateTime::parse_from_rfc3339(s).map_err(|_| ParseError::InvalidTimestampFormat)?;
        Ok(Self(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the parsed `DateTime<Utc>`, or `None` if the stored string is not valid RFC3339.
    #[must_use]
    pub fn as_datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.0)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }
}

impl Serialize for Timestamp {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).map_err(|e| D::Error::custom(e.to_string()))
    }
}

// ============================================================================
// ERROR TYPES (Calculations Layer)
// ============================================================================

/// Parse errors for invalid input format
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("workflow_name is empty string")]
    EmptyWorkflowName,
    #[error("workflow_name does not match pattern [a-z][a-z0-9_]*")]
    InvalidWorkflowNameFormat,
    #[error("signal_name is empty string")]
    EmptySignalName,
    #[error("signal_name does not match pattern [a-z][a-z0-9_]+")]
    InvalidSignalNameFormat,
    #[error("invocation_id is not valid 26-char Crockford base32")]
    InvalidUlidFormat,
    #[error("timestamp is not valid RFC3339")]
    InvalidTimestampFormat,
    #[error("unknown status variant")]
    UnknownStatusVariant,
}

/// Validation errors for business rule violations
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    #[error("retry_after_seconds must be > 0")]
    InvalidRetryAfterSeconds,
    #[error("invalid status transition")]
    InvalidStatusTransition,
    #[error("current_step is inconsistent with status")]
    InvalidCurrentStep,
}

/// Invariant violations for postcondition failures
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvariantViolation {
    #[error("updated_at timestamp precedes started_at")]
    UpdatedBeforeStarted,
    #[error("journal entries not in ascending seq order")]
    EntriesNotSorted,
    #[error("retry_after_seconds set for non-retryable error")]
    InvalidRetryForErrorType,
    #[error("invocation_id is immutable")]
    InvocationIdModified,
    #[error("status must be 'running' for StartWorkflowResponse")]
    InvalidStatusForResponse,
}

// ============================================================================
// REQUEST TYPES (Data Layer)
// ============================================================================

/// Request to start a new workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkflowRequest {
    pub workflow_name: WorkflowName,
    pub input: serde_json::Value,
}

/// Request to send a signal to a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRequest {
    pub signal_name: SignalName,
    pub payload: serde_json::Value,
}

// ============================================================================
// RESPONSE TYPES (Data Layer)
// ============================================================================

/// Workflow status value enum
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStatusValue {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Response after starting a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkflowResponse {
    pub invocation_id: InvocationId,
    pub workflow_name: String,
    pub status: WorkflowStatusValue,
    pub started_at: Timestamp,
}

impl StartWorkflowResponse {
    pub fn validate(&self) -> Result<(), InvariantViolation> {
        if self.status != WorkflowStatusValue::Running {
            return Err(InvariantViolation::InvalidStatusForResponse);
        }
        Ok(())
    }
}

/// Detailed workflow status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub instance_id: String,
    pub namespace: String,
    pub workflow_type: String,
    pub paradigm: String,
    pub phase: String,
    pub events_applied: u64,
}

/// Event record for NDJSON streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub seq: u64,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// Response to a signal request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalResponse {
    pub acknowledged: bool,
}

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

/// Response containing workflow journal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalResponse {
    pub invocation_id: InvocationId,
    pub entries: Vec<JournalEntry>,
}

impl JournalResponse {
    pub fn validate(&self) -> Result<(), InvariantViolation> {
        let seqs = self.entries.iter().map(|e| e.seq);
        if !is_sorted(seqs) {
            return Err(InvariantViolation::EntriesNotSorted);
        }
        Ok(())
    }
}

/// Response containing list of running workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkflowsResponse {
    pub workflows: Vec<WorkflowStatus>,
}

// ============================================================================
// DEFINITION INGESTION TYPES (bead wtf-qyxl)
// ============================================================================

/// POST /api/v1/definitions/<type> request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionRequest {
    pub source: String,
}

/// Diagnostic from the linter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticDto {
    pub code: String,
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<(usize, usize)>,
}

/// Response to POST /api/v1/definitions/<type>.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionResponse {
    pub valid: bool,
    pub diagnostics: Vec<DiagnosticDto>,
}

// ============================================================================
// V3 API REQUEST/RESPONSE TYPES (bead wtf-bjn0)
// ============================================================================

/// POST /api/v1/workflows request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V3StartRequest {
    pub namespace: String,
    pub workflow_type: String,
    pub paradigm: String,
    pub input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
}

/// Response to POST /api/v1/workflows on success (HTTP 201).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V3StartResponse {
    pub instance_id: String,
    pub namespace: String,
    pub workflow_type: String,
}

/// Response to GET /api/v1/workflows/:id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V3StatusResponse {
    pub instance_id: String,
    pub namespace: String,
    pub workflow_type: String,
    pub paradigm: String,
    pub phase: String,
    pub events_applied: u64,
}

/// POST /api/v1/workflows/:id/signals request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V3SignalRequest {
    pub signal_name: String,
    pub payload: serde_json::Value,
}

/// Generic API error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

impl ApiError {
    #[must_use]
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
        }
    }
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<RetryAfterSeconds>,
}

impl ErrorResponse {
    pub fn new(
        error: impl Into<String>,
        message: impl Into<String>,
        retry_after: Option<RetryAfterSeconds>,
    ) -> Result<Self, InvariantViolation> {
        let error_str = error.into();
        let is_retryable = is_retryable_error(&error_str);
        let has_retry = retry_after.is_some();
        if is_retryable && !has_retry {
            return Err(InvariantViolation::InvalidRetryForErrorType);
        }
        if !is_retryable && has_retry {
            return Err(InvariantViolation::InvalidRetryForErrorType);
        }
        Ok(Self {
            error: error_str,
            message: message.into(),
            retry_after_seconds: retry_after,
        })
    }
}

// ============================================================================
// HELPER FUNCTIONS (Calculations Layer)
// ============================================================================

fn is_retryable_error(error: &str) -> bool {
    matches!(error, "at_capacity")
}

fn is_sorted<T: PartialOrd + Clone>(mut iter: impl Iterator<Item = T>) -> bool {
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_name_valid() {
        let cases = ["a", "checkout", "order_2_process", "abc123"];
        for name in cases {
            assert!(WorkflowName::new(name).is_ok());
        }
    }

    #[test]
    fn test_workflow_name_invalid() {
        let cases = ["", "Invalid", "1order", "order-name"];
        for name in cases {
            assert!(WorkflowName::new(name).is_err());
        }
    }

    #[test]
    fn test_signal_name_valid() {
        let cases = ["payment_approved", "cancel", "signal_2"];
        for name in cases {
            assert!(SignalName::new(name).is_ok());
        }
    }

    #[test]
    fn test_invocation_id_valid() {
        assert!(InvocationId::from_str("01ARZ3NDEKTSV4RRFFQ69G5FAV").is_ok());
    }

    #[test]
    fn test_retry_after_seconds_valid() {
        assert!(RetryAfterSeconds::new(5).is_ok());
    }

    #[test]
    fn test_timestamp_valid() {
        assert!(Timestamp::new("2024-01-15T10:30:00Z").is_ok());
    }
}
