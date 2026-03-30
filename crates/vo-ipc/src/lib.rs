#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::nursery))]

pub mod config;
pub mod error;
pub mod run;
pub mod stderr;

pub use config::SubprocessConfig;
pub use error::{ConfigError, IpcError};
pub use run::{run_subprocess, SubprocessOutput};
pub use stderr::{MAX_STDERR_BYTES, TRUNCATION_MARKER};

#[cfg(test)]
mod unit_tests;
