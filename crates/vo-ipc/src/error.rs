use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    #[error("timeout must be greater than 0ms, got {timeout_ms}")]
    TimeoutMustBePositive { timeout_ms: u64 },
    #[error("program path does not exist: {path:?}")]
    ProgramMissing { path: PathBuf },
    #[error("program path is not executable: {path:?}")]
    ProgramNotExecutable { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IpcError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error("vo-ipc requires unix process support")]
    UnsupportedPlatform,
    #[error("failed to create subprocess pipes: {detail}")]
    PipeSetupFailed { detail: String },
    #[error("failed to spawn subprocess: {detail}")]
    SpawnFailed { detail: String },
    #[error("failed to wait for subprocess: {detail}")]
    WaitFailed { detail: String },
    #[error("failed to read fd4 payload: {detail}")]
    Fd4ReadFailed { detail: String },
    #[error("failed to capture stderr: {detail}")]
    StderrReadFailed { detail: String },
    #[error("failed to signal subprocess: {detail}")]
    SignalFailed { detail: String },
    #[error("subprocess timed out after {elapsed_ms}ms")]
    Timeout {
        elapsed_ms: u64,
        stderr_bytes: Vec<u8>,
        stderr_truncated: bool,
    },
    #[error("subprocess exited with code {exit_code}")]
    ProcessFailed {
        exit_code: i32,
        stderr_bytes: Vec<u8>,
        stderr_truncated: bool,
    },
}
