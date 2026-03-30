//! Scaffold integration tests for vo-ipc crate.
//!
//! Tests B-07 through B-17: Module accessibility, type exports, and Debug trait
//! implementations. These tests verify the crate's public API surface matches
//! the contract specification (POST-002, POST-003, INV-008, INV-009, INV-010).

// ---------------------------------------------------------------------------
// Helper: compile-time Debug trait bound assertion (MINOR-4 mandated pattern)
// ---------------------------------------------------------------------------

/// Asserts at compile time that type T implements `std::fmt::Debug`.
/// No instance construction needed — works for unit structs, empty structs,
/// and zero-variant enums.
fn assert_debug<T: std::fmt::Debug>() {}

// ---------------------------------------------------------------------------
// B-07: spawn module is publicly accessible
// ---------------------------------------------------------------------------

#[test]
fn spawn_module_is_publicly_accessible() {
    // Given: vo-ipc crate is compiled
    // When: An integration test imports vo_ipc::spawn
    // Then: The import resolves — the test compiles and passes
    let _: fn() = || {
        let _ = std::any::type_name::<vo_ipc::spawn::SpawnConfig>();
    };
}

// ---------------------------------------------------------------------------
// B-08: envelope module is publicly accessible
// ---------------------------------------------------------------------------

#[test]
fn envelope_module_is_publicly_accessible() {
    // Given: vo-ipc crate is compiled
    // When: An integration test imports vo_ipc::envelope
    // Then: The import resolves — the test compiles and passes
    let _: fn() = || {
        let _ = std::any::type_name::<vo_ipc::envelope::Envelope>();
    };
}

// ---------------------------------------------------------------------------
// B-09: fd3 module is publicly accessible
// ---------------------------------------------------------------------------

#[test]
fn fd3_module_is_publicly_accessible() {
    // Given: vo-ipc crate is compiled
    // When: An integration test imports vo_ipc::fd3
    // Then: The import resolves — the test compiles and passes
    let _: fn() = || {
        let _ = std::any::type_name::<vo_ipc::fd3::Fd3Writer>();
    };
}

// ---------------------------------------------------------------------------
// B-10: fd4 module is publicly accessible
// ---------------------------------------------------------------------------

#[test]
fn fd4_module_is_publicly_accessible() {
    // Given: vo-ipc crate is compiled
    // When: An integration test imports vo_ipc::fd4
    // Then: The import resolves — the test compiles and passes
    let _: fn() = || {
        let _ = std::any::type_name::<vo_ipc::fd4::Fd4Writer>();
    };
}

// ---------------------------------------------------------------------------
// B-11: timeout module is publicly accessible
// ---------------------------------------------------------------------------

#[test]
fn timeout_module_is_publicly_accessible() {
    // Given: vo-ipc crate is compiled
    // When: An integration test imports vo_ipc::timeout
    // Then: The import resolves — the test compiles and passes
    let _: fn() = || {
        let _ = std::any::type_name::<vo_ipc::timeout::TimeoutConfig>();
    };
}

// ---------------------------------------------------------------------------
// B-12: lib.rs declares exactly five pub mod statements
// ---------------------------------------------------------------------------

#[test]
fn lib_rs_declares_exactly_five_pub_mod_statements() {
    // Given: crates/vo-ipc/src/lib.rs file contents
    let lib_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
    let content =
        std::fs::read_to_string(&lib_path).unwrap_or_else(|e| panic!("Failed to read lib.rs: {e}"));

    // When: All lines matching the pattern `pub mod <ident>;` are counted
    let pub_mods: Vec<&str> = content
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("pub mod ") && line.ends_with(';'))
        .collect();

    let expected_modules: std::collections::BTreeSet<&str> =
        ["spawn", "envelope", "fd3", "fd4", "timeout"]
            .iter()
            .copied()
            .collect();

    let actual_modules: std::collections::BTreeSet<&str> = pub_mods
        .iter()
        .filter_map(|line| {
            line.strip_prefix("pub mod ")
                .and_then(|rest| rest.strip_suffix(';'))
                .map(str::trim)
        })
        .collect();

    // Then: Count == 5
    assert_eq!(
        pub_mods.len(),
        5,
        "Expected exactly 5 `pub mod` statements in lib.rs, found {}: {:?}",
        pub_mods.len(),
        pub_mods
    );

    // And: The identifiers are exactly: {spawn, envelope, fd3, fd4, timeout}
    assert_eq!(
        actual_modules, expected_modules,
        "Expected pub mod set {:?}, found {:?}",
        expected_modules, actual_modules
    );
}

// ---------------------------------------------------------------------------
// B-13: spawn module exports — SpawnConfig, ChildHandle, SpawnResult, SpawnError
// ---------------------------------------------------------------------------

#[test]
fn spawn_config_is_public_and_implements_debug() {
    // Given: vo-ipc crate is compiled
    // When: SpawnConfig is imported and checked for Debug
    // Then: The trait bound is satisfied at compile time
    assert_debug::<vo_ipc::spawn::SpawnConfig>();
}

#[test]
fn child_handle_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::spawn::ChildHandle>();
}

#[test]
fn spawn_result_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::spawn::SpawnResult>();
}

#[test]
fn spawn_error_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::spawn::SpawnError>();
}

// ---------------------------------------------------------------------------
// B-14: envelope module exports — Envelope, EnvelopeError
// ---------------------------------------------------------------------------

#[test]
fn envelope_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::envelope::Envelope>();
}

#[test]
fn envelope_error_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::envelope::EnvelopeError>();
}

// ---------------------------------------------------------------------------
// B-15: fd3 module exports — Fd3Writer, Fd3Reader, Fd3Error
// ---------------------------------------------------------------------------

#[test]
fn fd3_writer_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::fd3::Fd3Writer>();
}

#[test]
fn fd3_reader_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::fd3::Fd3Reader>();
}

#[test]
fn fd3_error_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::fd3::Fd3Error>();
}

// ---------------------------------------------------------------------------
// B-16: fd4 module exports — Fd4Writer, Fd4Reader, Fd4Error
// ---------------------------------------------------------------------------

#[test]
fn fd4_writer_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::fd4::Fd4Writer>();
}

#[test]
fn fd4_reader_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::fd4::Fd4Reader>();
}

#[test]
fn fd4_error_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::fd4::Fd4Error>();
}

// ---------------------------------------------------------------------------
// B-17: timeout module exports — TimeoutConfig, TimeoutError
// ---------------------------------------------------------------------------

#[test]
fn timeout_config_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::timeout::TimeoutConfig>();
}

#[test]
fn timeout_error_is_public_and_implements_debug() {
    assert_debug::<vo_ipc::timeout::TimeoutError>();
}
