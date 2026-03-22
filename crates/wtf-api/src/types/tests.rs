//! Tests for API types.

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
