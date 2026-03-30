//! Workspace integration tests for vo-ipc crate.
//!
//! Tests B-01 through B-06: Workspace membership, cargo metadata resolution,
//! pre-existing crate compilation, independent compilation, clippy cleanliness,
//! and full workspace compilation.
//!
//! These tests invoke real cargo commands and assert on exit codes and stderr.
//! They run unconditionally in CI (no `#[ignore]`).

/// Path to the workspace root (parent of crates/).
const WORKSPACE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

// ---------------------------------------------------------------------------
// B-01: Workspace includes vo-ipc in members array
// ---------------------------------------------------------------------------

#[test]
fn workspace_members_contains_vo_ipc_when_cargo_toml_parsed() {
    // Given: The workspace root Cargo.toml at the repository root
    let workspace_cargo = std::path::PathBuf::from(WORKSPACE_ROOT).join("Cargo.toml");
    let content = std::fs::read_to_string(&workspace_cargo)
        .unwrap_or_else(|e| panic!("Failed to read workspace Cargo.toml: {e}"));

    // When: The [workspace] members array is parsed
    // Then: The members list contains the string "crates/vo-ipc"
    assert!(
        content.contains("\"crates/vo-ipc\""),
        "Workspace Cargo.toml must contain \"crates/vo-ipc\" in members array.\n\
         Actual content of members section:\n{}",
        content
            .lines()
            .skip_while(|l| !l.contains("members"))
            .take(20)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ---------------------------------------------------------------------------
// B-02: cargo-metadata resolves vo-ipc with correct deps
// ---------------------------------------------------------------------------

#[test]
fn cargo_metadata_resolves_vo_ipc_with_correct_deps() {
    // Given: vo-ipc is listed in workspace members with valid Cargo.toml
    // When: `cargo metadata --format-version 1` is executed
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(WORKSPACE_ROOT)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute cargo metadata: {e}"));

    // Then: Exit code is Some(0)
    let exit_code = output.status.code();
    assert_eq!(
        exit_code,
        Some(0),
        "cargo metadata exited with {:?}, expected Some(0).\nstderr: {}",
        exit_code,
        String::from_utf8_lossy(&output.stderr)
    );

    // And: JSON output contains a package with name == "vo-ipc"
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Failed to parse cargo metadata JSON: {e}"));

    let packages = json["packages"]
        .as_array()
        .unwrap_or_else(|| panic!("Expected 'packages' array in cargo metadata output"));

    let vo_ipc_pkg = packages
        .iter()
        .find(|p| p["name"].as_str() == Some("vo-ipc"));

    assert!(
        vo_ipc_pkg.is_some(),
        "Expected to find package 'vo-ipc' in cargo metadata output"
    );

    // And: That package's normal (non-dev) dependencies are exactly: "tokio", "serde_json", "vo-types"
    // cargo metadata lists ALL deps; filter to kind == null (normal deps only)
    let deps: std::collections::BTreeSet<String> = vo_ipc_pkg
        .unwrap_or_else(|| panic!("vo-ipc package not found"))["dependencies"]
        .as_array()
        .unwrap_or_else(|| panic!("Expected 'dependencies' array"))
        .iter()
        .filter(|d| d["kind"].is_null())
        .filter_map(|d| d["name"].as_str().map(String::from))
        .collect();

    let expected: std::collections::BTreeSet<String> = ["tokio", "serde_json", "vo-types"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    assert_eq!(
        deps, expected,
        "vo-ipc dependencies must be exactly {{tokio, serde_json, vo-types}}, found: {deps:?}"
    );
}

// ---------------------------------------------------------------------------
// B-03: Pre-existing crates still compile (E2E)
// ---------------------------------------------------------------------------

#[test]
fn workspace_check_succeeds_with_zero_errors_when_vo_ipc_added() {
    // Given: The workspace contains all crates AND the new vo-ipc
    // When: `cargo check --workspace` is executed
    let output = std::process::Command::new("cargo")
        .args(["check", "--workspace"])
        .current_dir(WORKSPACE_ROOT)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute cargo check --workspace: {e}"));

    let exit_code = output.status.code();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Then: Exit code is Some(0)
    assert_eq!(
        exit_code,
        Some(0),
        "cargo check --workspace exited with {:?}, expected Some(0).\nstderr: {stderr}",
        exit_code
    );

    // And: stderr does not contain "error[E"
    assert!(
        !stderr.contains("error[E"),
        "cargo check --workspace produced compilation errors:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// B-04: vo-ipc compiles independently
// ---------------------------------------------------------------------------

#[test]
fn vo_ipc_check_succeeds_independently() {
    // Given: crates/vo-ipc/ exists with valid Cargo.toml and src/lib.rs
    // When: `cargo check -p vo-ipc` is executed
    let output = std::process::Command::new("cargo")
        .args(["check", "-p", "vo-ipc"])
        .current_dir(WORKSPACE_ROOT)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute cargo check -p vo-ipc: {e}"));

    let exit_code = output.status.code();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Then: Exit code is Some(0)
    assert_eq!(
        exit_code,
        Some(0),
        "cargo check -p vo-ipc exited with {:?}, expected Some(0).\nstderr: {stderr}",
        exit_code
    );

    // And: stderr does not contain "error[E"
    assert!(
        !stderr.contains("error[E"),
        "cargo check -p vo-ipc produced compilation errors:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// B-05: Zero clippy warnings
// ---------------------------------------------------------------------------

#[test]
fn vo_ipc_clippy_emits_zero_warnings() {
    // Given: vo-ipc crate exists with stub code
    // When: `cargo clippy -p vo-ipc --tests -- -D warnings` is executed
    let output = std::process::Command::new("cargo")
        .args(["clippy", "-p", "vo-ipc", "--tests", "--", "-D", "warnings"])
        .current_dir(WORKSPACE_ROOT)
        .output()
        .expect("cargo clippy command must execute");

    let exit_code = output.status.code();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Then: Exit code is Some(0)
    assert_eq!(
        exit_code,
        Some(0),
        "cargo clippy -p vo-ipc exited with {:?}, expected Some(0).\nstderr: {stderr}",
        exit_code
    );

    // And: stderr does not contain "warning:" (excluding "generated N warnings" summary)
    // Note: #[allow(dead_code)] on stubs is acceptable and does not count
    let has_warnings = stderr
        .lines()
        .any(|line| line.contains("warning:") && !line.contains("generated"));
    assert!(
        !has_warnings,
        "cargo clippy -p vo-ipc produced warnings:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// B-06: Full workspace compiles (E2E)
// ---------------------------------------------------------------------------

#[test]
fn full_workspace_check_passes_with_vo_ipc() {
    // Given: vo-ipc is added to the workspace
    // When: `cargo check --workspace` is executed
    let output = std::process::Command::new("cargo")
        .args(["check", "--workspace"])
        .current_dir(WORKSPACE_ROOT)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute cargo check --workspace: {e}"));

    let exit_code = output.status.code();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Then: Exit code is Some(0)
    assert_eq!(
        exit_code,
        Some(0),
        "cargo check --workspace exited with {:?}, expected Some(0).\nstderr: {stderr}",
        exit_code
    );

    // And: stderr does not contain "error[E"
    assert!(
        !stderr.contains("error[E"),
        "cargo check --workspace produced compilation errors:\n{stderr}"
    );
}
