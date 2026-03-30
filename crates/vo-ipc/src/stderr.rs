use std::cmp::min;
use tokio::io::{AsyncRead, AsyncReadExt};

pub const MAX_STDERR_BYTES: usize = 1_048_576;
pub const TRUNCATION_MARKER: &str = "\n[... TRUNCATED AT 1MB ...]";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StderrCapture {
    pub(crate) bytes: Vec<u8>,
    pub(crate) truncated: bool,
    pub(crate) observed_bytes: usize,
}

impl StderrCapture {
    #[must_use]
    pub(crate) const fn empty() -> Self {
        Self {
            bytes: Vec::new(),
            truncated: false,
            observed_bytes: 0,
        }
    }
}

#[must_use]
pub(crate) fn update_capture(capture: StderrCapture, chunk: &[u8]) -> StderrCapture {
    let StderrCapture {
        bytes: existing_bytes,
        truncated: was_truncated,
        observed_bytes: previous_observed_bytes,
    } = capture;

    let available = MAX_STDERR_BYTES.saturating_sub(existing_bytes.len());
    let retained = min(available, chunk.len());
    let mut bytes = existing_bytes;
    bytes.extend_from_slice(&chunk[..retained]);

    let truncated = was_truncated || chunk.len() > retained;
    let observed_bytes = previous_observed_bytes.saturating_add(chunk.len());

    StderrCapture {
        bytes,
        truncated,
        observed_bytes,
    }
}

#[must_use]
pub(crate) fn finalize_capture(capture: StderrCapture) -> StderrCapture {
    let StderrCapture {
        bytes: original_bytes,
        truncated,
        observed_bytes,
    } = capture;

    if !truncated || original_bytes.ends_with(TRUNCATION_MARKER.as_bytes()) {
        return StderrCapture {
            bytes: original_bytes,
            truncated,
            observed_bytes,
        };
    }

    let mut bytes = original_bytes;
    bytes.extend_from_slice(TRUNCATION_MARKER.as_bytes());
    StderrCapture {
        bytes,
        truncated: true,
        observed_bytes,
    }
}

pub(crate) async fn read_bounded_stderr<R>(mut reader: R) -> std::io::Result<StderrCapture>
where
    R: AsyncRead + Unpin,
{
    let mut buffer = [0_u8; 8192];
    let mut capture = StderrCapture::empty();

    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            return Ok(finalize_capture(capture));
        }
        capture = update_capture(capture, &buffer[..read]);
    }
}
