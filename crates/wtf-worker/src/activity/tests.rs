use super::reporting::MAX_HEARTBEAT_PROGRESS_BYTES;
use super::retry::*;
use wtf_common::RetryPolicy;

#[test]
fn retries_exhausted_first_attempt_of_one() {
    assert!(retries_exhausted(1, 1));
}

#[test]
fn retries_exhausted_not_yet_on_first_of_three() {
    assert!(!retries_exhausted(1, 3));
}

#[test]
fn retries_exhausted_second_of_three_not_yet() {
    assert!(!retries_exhausted(2, 3));
}

#[test]
fn retries_exhausted_third_of_three_is_exhausted() {
    assert!(retries_exhausted(3, 3));
}

#[test]
fn retries_exhausted_beyond_max_is_exhausted() {
    assert!(retries_exhausted(5, 3));
}

#[test]
fn retries_exhausted_zero_max_always_exhausted() {
    assert!(retries_exhausted(0, 0));
}

#[test]
fn retries_not_exhausted_at_zero_attempts_when_max_is_three() {
    assert!(!retries_exhausted(0, 3));
}

#[test]
fn calculate_backoff_delay_first_attempt() {
    let policy = RetryPolicy::default();
    let delay = calculate_backoff_delay(1, &policy);
    assert_eq!(delay, Some(1000));
}

#[test]
fn calculate_backoff_delay_second_attempt() {
    let policy = RetryPolicy::default();
    let delay = calculate_backoff_delay(2, &policy);
    assert_eq!(delay, Some(2000));
}

#[test]
fn calculate_backoff_delay_third_attempt() {
    let policy = RetryPolicy::default();
    let delay = calculate_backoff_delay(3, &policy);
    assert_eq!(delay, Some(4000));
}

#[test]
fn calculate_backoff_delay_caps_at_max() {
    let policy = RetryPolicy {
        initial_interval_ms: 1000,
        backoff_coefficient: 2.0,
        max_interval_ms: 10000,
        ..Default::default()
    };
    let delay = calculate_backoff_delay(10, &policy);
    assert_eq!(delay, Some(10000));
}

#[test]
fn calculate_backoff_delay_zero_attempt_returns_none() {
    let policy = RetryPolicy::default();
    let delay = calculate_backoff_delay(0, &policy);
    assert_eq!(delay, None);
}

#[test]
fn calculate_backoff_delay_linear_coefficient_one() {
    let policy = RetryPolicy {
        initial_interval_ms: 500,
        backoff_coefficient: 1.0,
        max_interval_ms: 60000,
        ..Default::default()
    };
    assert_eq!(calculate_backoff_delay(1, &policy), Some(500));
    assert_eq!(calculate_backoff_delay(2, &policy), Some(500));
    assert_eq!(calculate_backoff_delay(3, &policy), Some(500));
}

#[test]
fn calculate_backoff_delay_fractional_coefficient() {
    let policy = RetryPolicy {
        initial_interval_ms: 1000,
        backoff_coefficient: 1.5,
        max_interval_ms: 60000,
        ..Default::default()
    };
    assert_eq!(calculate_backoff_delay(1, &policy), Some(1000));
    assert_eq!(calculate_backoff_delay(2, &policy), Some(1500));
    assert_eq!(calculate_backoff_delay(3, &policy), Some(2250));
}

#[test]
fn heartbeat_max_progress_bytes_constant_is_1kb() {
    assert_eq!(MAX_HEARTBEAT_PROGRESS_BYTES, 1024);
}
