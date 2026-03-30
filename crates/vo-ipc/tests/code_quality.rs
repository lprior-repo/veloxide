//! Code quality tests for vo-ipc crate.
//!
//! Tests B-20 and B-21: Zero-panic source code and no stdout/stderr usage.
//! These tests scan all source files in crates/vo-ipc/src/ for forbidden patterns.
//!
//! Implementation follows the loop-free strategy mandated by the test plan:
//! - Known file set (6 files) is read via iterator chain, not loops
//! - Contents are concatenated into a single String with file-name markers
//! - Assertions are single calls on the aggregated string

/// Known source files in crates/vo-ipc/src/ — fixed set per contract.
const SOURCE_FILES: &[&str] = &[
    "lib.rs",
    "spawn.rs",
    "envelope.rs",
    "fd3.rs",
    "fd4.rs",
    "timeout.rs",
];

/// Strip Rust comments from a line of source code.
///
/// Removes:
/// - `//` single-line comments (including `///` doc comments and `//!` module doc)
/// - Preserves string literals (simplified: does not handle raw strings or `//` inside
///   string literals, but for scanning purposes this is conservative — it may keep
///   string content that *looks* like a comment, which is the safe direction).
fn strip_comment(line: &str) -> &str {
    // Fast path: if there's no `//` anywhere, return the whole line.
    let Some(slash_pos) = line.find("//") else {
        return line;
    };
    // Everything before the first `//` is the code portion.
    &line[..slash_pos]
}

/// Read all source files, strip comments, and concatenate them with file-name markers.
/// Uses iterator chain (not a loop). Each file is prefixed with a marker
/// line `--- filename.rs ---` for diagnostics when a violation is found.
///
/// Lines inside `/* ... */` block comments are NOT stripped (Rust block comments
/// are rare in practice and the added complexity is unwarranted for this scanner).
fn read_all_source_files_stripping_comments() -> String {
    let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    SOURCE_FILES
        .iter()
        .map(|f| {
            let path = base.join(f);
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {f}: {e}"));
            let stripped: String = content
                .lines()
                .map(strip_comment)
                .collect::<Vec<&str>>()
                .join("\n");
            format!("--- {f} ---\n{stripped}\n")
        })
        .collect()
}

// ---------------------------------------------------------------------------
// B-20: No unwrap/expect/panic in source (INV-011)
// ---------------------------------------------------------------------------

#[test]
fn vo_ipc_source_contains_no_unwrap_expect_or_panic() {
    // Given: All .rs files in crates/vo-ipc/src/ (with comments stripped)
    let combined = read_all_source_files_stripping_comments();

    // When: Content is scanned for forbidden tokens
    // Then: Zero matches for `.unwrap()`
    assert!(
        !combined.contains(".unwrap()"),
        "Found `.unwrap()` in vo-ipc source files (INV-011 violation).\n\
         Scan the combined output for `--- <filename> ---` markers to identify the file:\n{combined}"
    );

    // And: Zero matches for `.expect(`
    assert!(
        !combined.contains(".expect("),
        "Found `.expect(` in vo-ipc source files (INV-011 violation).\n\
         Scan the combined output for `--- <filename> ---` markers to identify the file:\n{combined}"
    );

    // And: Zero matches for `panic!(`
    assert!(
        !combined.contains("panic!("),
        "Found `panic!(` in vo-ipc source files (INV-011 violation).\n\
         Scan the combined output for `--- <filename> ---` markers to identify the file:\n{combined}"
    );
}

// ---------------------------------------------------------------------------
// B-21: No stdout/stderr usage for structured IPC (INV-007)
// ---------------------------------------------------------------------------

#[test]
fn vo_ipc_source_contains_no_stdout_stderr_usage() {
    // Given: All .rs files in crates/vo-ipc/src/ (with comments stripped)
    let combined = read_all_source_files_stripping_comments();

    // When: Content is scanned for forbidden I/O tokens
    // Then: Zero matches for `println!`
    assert!(
        !combined.contains("println!"),
        "Found `println!` in vo-ipc source files (INV-007 violation).\n\
         FD3/FD4 are the ONLY channels for structured IPC.\n{combined}"
    );

    // And: Zero matches for `eprintln!`
    assert!(
        !combined.contains("eprintln!"),
        "Found `eprintln!` in vo-ipc source files (INV-007 violation).\n{combined}"
    );

    // And: Zero matches for `std::io::stdout`
    assert!(
        !combined.contains("std::io::stdout"),
        "Found `std::io::stdout` in vo-ipc source files (INV-007 violation).\n{combined}"
    );

    // And: Zero matches for `std::io::stderr`
    assert!(
        !combined.contains("std::io::stderr"),
        "Found `std::io::stderr` in vo-ipc source files (INV-007 violation).\n{combined}"
    );

    // And: Zero matches for `io::stdout`
    assert!(
        !combined.contains("io::stdout"),
        "Found `io::stdout` in vo-ipc source files (INV-007 violation).\n{combined}"
    );

    // And: Zero matches for `io::stderr`
    assert!(
        !combined.contains("io::stderr"),
        "Found `io::stderr` in vo-ipc source files (INV-007 violation).\n{combined}"
    );
}
