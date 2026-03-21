# ADR-010: Error Handling Taxonomy

## Status

Accepted

## Context

wtf-engine handles errors at multiple levels:

1. **Workflow errors** - Step failures, caught errors, retry exhaustion
2. **Activity errors** - Activity crashes, timeouts
3. **System errors** - Storage failures, actor crashes
4. **User errors** - Invalid input, unknown workflow

Each level needs **typed errors** with **actionable information**.

### Error Design Principles

1. **Exhaustive** - All error variants documented
2. **Actionable** - Errors suggest remediation
3. **Wrapped** - Low-level errors wrapped with context
4. **Serializable** - Errors can be sent over API

## Decision

We will use a **hierarchical error taxonomy** with `thiserror` for type safety.

### Top-Level Error

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enumWfError {
    // Workflow execution errors
    #[error("workflow {workflow} not found")]
    WorkflowNotFound { workflow: String },

    #[error("invocation {invocation_id} not found")]
    InvocationNotFound { invocation_id: String },

    #[error("step {step} not found in workflow {workflow}")]
    StepNotFound { step: u32, workflow: String },

    #[error("invalid state transition: {0}")]
    InvalidTransition { message: String },

    #[error("workflow failed: {0}")]
    WorkflowFailed { reason: String },

    // Activity errors
    #[error("activity {activity} not found")]
    ActivityNotFound { activity: String },

    #[error("activity {activity} timed out after {timeout_secs}s")]
    ActivityTimeout { activity: String, timeout_secs: u64 },

    #[error("activity {activity} failed: {error}")]
    ActivityFailed { activity: String, error: String },

    // Retry/Catch errors
    #[error("max retries ({attempts}) exceeded for activity {activity}")]
    MaxRetriesExceeded { attempts: u32, activity: String },

    #[error("caught error {error_type}: {message}")]
    CaughtError { error_type: String, message: String, next_state: String },

    // Storage errors
    #[error("storage error: {0}")]
    Storage { #[from] source: sled::Error },

    #[error("serialization error: {0}")]
    Serialization { #[from] source: serde_json::Error },

    // Capacity errors
    #[error("at capacity: {running} workflows running (max {max})")]
    AtCapacity { running: usize, max: usize },
}
```

### Start Errors

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum StartError {
    #[error("workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("at capacity: {running} workflows running (max {max})")]
    AtCapacity { running: usize, max: usize },
}
```

### Signal Errors

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum SignalError {
    #[error("invocation not found: {0}")]
    InvocationNotFound(String),

    #[error("signal {signal} not allowed for workflow in state {state}")]
    SignalNotAllowed { signal: String, state: String },
}
```

### Step Errors

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum StepError {
    #[error("retryable error: {0}")]
    Retryable(String),

    #[error("fatal error: {0}")]
    Fatal(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("cancelled")]
    Cancelled,
}
```

### Error Propagation

```rust
impl From<sled::Error> forWfError {
    fn from(e: sled::Error) -> Self {
       WfError::Storage { source: e }
    }
}

impl From<serde_json::Error> forWfError {
    fn from(e: serde_json::Error) -> Self {
       WfError::Serialization { source: e }
    }
}
```

### API Error Response

```rust
#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<u64>,
}

impl From<&WfError> for ApiError {
    fn from(e: &WfError) -> Self {
        match e {
            WfError::AtCapacity { running, max } => ApiError {
                error: "at_capacity".to_string(),
                message: format!("{running} workflows running (max {max})"),
                details: None,
                retry_after_seconds: Some(5),
            },
            WfError::InvocationNotFound { invocation_id } => ApiError {
                error: "not_found".to_string(),
                message: format!("invocation {invocation_id} not found"),
                details: None,
                retry_after_seconds: None,
            },
            _ => ApiError {
                error: "internal_error".to_string(),
                message: e.to_string(),
                details: None,
                retry_after_seconds: None,
            },
        }
    }
}
```

## Consequences

### Positive

- **Exhaustive matching** - Compiler catches missing variants
- **Rich context** - Errors carry actionable information
- **API-ready** - Easy to serialize to JSON

### Negative

- **Verbose** - Many error types to implement
- **Complexity** - Error wrapping can be tedious

### Guidelines

- Always wrap lower-level errors with context
- Use `thiserror` for all error types
- Include retry_after_seconds for retryable errors
