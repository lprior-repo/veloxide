/// Determine whether retries are exhausted given the attempt number and policy.
#[must_use]
pub fn retries_exhausted(attempt: u32, max_attempts: u32) -> bool {
    attempt >= max_attempts
}

/// Calculate exponential backoff delay in milliseconds.
#[must_use]
pub fn calculate_backoff_delay(
    attempt: u32,
    retry_policy: &wtf_common::RetryPolicy,
) -> Option<u64> {
    if attempt == 0 {
        return None;
    }

    let exponent = (attempt - 1) as f64;
    let multiplier = retry_policy.backoff_coefficient.powf(exponent);
    let delay_f = (retry_policy.initial_interval_ms as f64) * multiplier;

    if delay_f > u64::MAX as f64 {
        return Some(retry_policy.max_interval_ms);
    }

    let delay = delay_f as u64;
    Some(delay.min(retry_policy.max_interval_ms))
}
