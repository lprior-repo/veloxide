#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

// Re-exports from rules module
pub use crate::rules::check_random_in_workflow;
