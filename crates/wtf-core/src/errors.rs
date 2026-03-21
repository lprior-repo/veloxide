//! errors.rs - Error types placeholder

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("invalid workflow graph")]
    InvalidGraph,
    #[error("journal error: {0}")]
    JournalError(String),
}
