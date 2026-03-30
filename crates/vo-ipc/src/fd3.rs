/// Writer for FD3 (Engine -> Task direction).
/// Wraps a pipe write-end with envelope framing.
#[derive(Debug)]
pub struct Fd3Writer {
    // Stub: fields TBD in implementation bead
}

/// Reader for FD3 (Task -> Engine direction -- Task-side read).
/// Reads length-prefixed envelope from FD3.
#[derive(Debug)]
pub struct Fd3Reader {
    // Stub: fields TBD in implementation bead
}

/// Errors specific to FD3 operations.
#[derive(Debug)]
pub enum Fd3Error {
    // Stub: variants TBD in implementation bead
}
