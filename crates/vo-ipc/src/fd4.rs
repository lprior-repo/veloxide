/// Writer for FD4 (Task -> Engine direction -- Task-side write).
/// Wraps a pipe write-end with envelope framing.
#[derive(Debug)]
pub struct Fd4Writer {
    // Stub: fields TBD in implementation bead
}

/// Reader for FD4 (Engine reads Task output).
/// Reads length-prefixed envelope from FD4 with bounded buffer.
#[derive(Debug)]
pub struct Fd4Reader {
    // Stub: fields TBD in implementation bead
}

/// Errors specific to FD4 operations.
#[derive(Debug)]
pub enum Fd4Error {
    // Stub: variants TBD in implementation bead
}
