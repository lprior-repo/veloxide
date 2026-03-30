use crate::error::ConfigError;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprocessConfig {
    executable_path: PathBuf,
    timeout_ms: u64,
    fd3_payload: Vec<u8>,
}

impl SubprocessConfig {
    /// Creates a validated subprocess configuration.
    ///
    /// # Errors
    /// Returns [`ConfigError`] when the timeout is zero, the program path is missing,
    /// or the target path is not executable.
    pub fn new<P, B>(path: P, timeout_ms: u64, fd3_payload: B) -> Result<Self, ConfigError>
    where
        P: AsRef<Path>,
        B: Into<Vec<u8>>,
    {
        let payload = fd3_payload.into();
        let provided_path = path.as_ref().to_path_buf();

        validate_timeout(timeout_ms)?;
        validate_program_path(&provided_path)?;

        let executable_path =
            fs::canonicalize(&provided_path).map_err(|_| ConfigError::ProgramMissing {
                path: provided_path.clone(),
            })?;

        Ok(Self {
            executable_path,
            timeout_ms,
            fd3_payload: payload,
        })
    }

    #[must_use]
    pub fn executable_path(&self) -> &Path {
        &self.executable_path
    }

    #[must_use]
    pub const fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    #[must_use]
    pub fn fd3_payload(&self) -> &[u8] {
        &self.fd3_payload
    }

    #[must_use]
    pub(crate) fn argv(&self) -> Vec<OsString> {
        parse_fd3_payload_as_argv(&self.fd3_payload)
    }
}

pub(crate) fn validate_timeout(timeout_ms: u64) -> Result<(), ConfigError> {
    (timeout_ms > 0)
        .then_some(())
        .ok_or(ConfigError::TimeoutMustBePositive { timeout_ms })
}

pub(crate) fn validate_program_path(path: &Path) -> Result<(), ConfigError> {
    let metadata = fs::metadata(path).map_err(|_| ConfigError::ProgramMissing {
        path: path.to_path_buf(),
    })?;

    let is_executable = metadata.permissions().mode() & 0o111 != 0;

    is_executable
        .then_some(())
        .ok_or_else(|| ConfigError::ProgramNotExecutable {
            path: path.to_path_buf(),
        })
}

#[must_use]
pub(crate) fn parse_fd3_payload_as_argv(payload: &[u8]) -> Vec<OsString> {
    std::str::from_utf8(payload).map_or_else(
        |_| Vec::new(),
        |text| {
            text.split_whitespace()
                .map(OsString::from)
                .collect::<Vec<_>>()
        },
    )
}
