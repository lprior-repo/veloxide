/// Configuration for spawning a subprocess.
/// Carries the binary path, environment overrides, and timeout.
#[derive(Debug)]
pub struct SpawnConfig {
    // Stub: fields TBD in implementation bead
}

/// Handle to a running child process with FD3/FD4 pipes attached.
#[derive(Debug)]
pub struct ChildHandle {
    // Stub: fields TBD in implementation bead
}

/// Result of a completed subprocess execution.
#[derive(Debug)]
pub struct SpawnResult {
    // Stub: fields TBD in implementation bead
}

/// Errors that can occur during subprocess spawn and lifecycle.
#[derive(Debug)]
pub enum SpawnError {
    // Stub: variants TBD in implementation bead
}
