#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use std::time::Duration;

use crate::ui::monitor_mode::types::{ScrubberBounds, ScrubberError};

#[must_use]
pub fn validate_replay_seq(seq: u64, max_seq: u64) -> Result<(), ScrubberError> {
    if seq > max_seq {
        Err(ScrubberError::InvalidSequence(seq, max_seq))
    } else {
        Ok(())
    }
}

#[must_use]
pub fn compute_playback_interval() -> Duration {
    Duration::from_millis(500)
}

#[must_use]
pub fn format_seq_label(seq: u64, max_seq: u64) -> String {
    if max_seq == 0 {
        return "0".to_string();
    }
    let percentage = (seq as f64 / max_seq as f64) * 100.0;
    format!("{} ({}%)", seq, percentage as usize)
}

#[must_use]
pub fn should_disable_sse(mode: &crate::ui::monitor_mode::types::MonitorMode) -> bool {
    mode.is_historical()
}

pub fn format_timestamp_relative(timestamp: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| {
            let elapsed = chrono::Utc::now().signed_duration_since(dt.with_timezone(&chrono::Utc));
            if elapsed.num_minutes() < 1 {
                "just now".to_string()
            } else if elapsed.num_hours() < 1 {
                format!("{}m ago", elapsed.num_minutes())
            } else if elapsed.num_days() < 1 {
                format!("{}h ago", elapsed.num_hours())
            } else {
                format!("{}d ago", elapsed.num_days())
            }
        })
        .unwrap_or_else(|_| timestamp.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_valid_seq_when_validating_replay_seq_then_ok() {
        let result = validate_replay_seq(50, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn given_seq_exceeds_max_when_validating_replay_seq_then_error() {
        let result = validate_replay_seq(150, 100);
        assert!(matches!(
            result,
            Err(ScrubberError::InvalidSequence(150, 100))
        ));
    }

    #[test]
    fn given_seq_equals_max_when_validating_replay_seq_then_ok() {
        let result = validate_replay_seq(100, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn given_scrubber_bounds_when_contains_then_true_for_valid() {
        let bounds = ScrubberBounds::new(0, 100);
        assert!(bounds.contains(50));
        assert!(bounds.contains(0));
        assert!(bounds.contains(100));
    }

    #[test]
    fn given_scrubber_bounds_when_contains_then_false_for_invalid() {
        let bounds = ScrubberBounds::new(0, 100);
        assert!(!bounds.contains(101));
        assert!(!bounds.contains(200));
    }

    #[test]
    fn given_scrubber_bounds_when_clamp_then_clamps_correctly() {
        let bounds = ScrubberBounds::new(0, 100);
        assert_eq!(bounds.clamp(50), 50);
        assert_eq!(bounds.clamp(150), 100);
        assert_eq!(bounds.clamp(0), 0);
    }
}
