//! Dependency contract tests for vo-ipc crate.
//!
//! Tests B-18 and B-19: Dependency allowlist enforcement and forbidden dependency
//! rejection. These tests verify the exact set of allowed dependencies and that
//! no crate from the forbidden list (INV-002 through INV-006) is present.
//!
//! Uses the `toml` crate for structured TOML parsing (not raw string matching).

/// Path to the crate's Cargo.toml.
const CARGO_TOML_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");

/// Read and parse the crate Cargo.toml as a `toml::Value`.
fn parse_cargo_toml() -> toml::Value {
    let content = std::fs::read_to_string(CARGO_TOML_PATH)
        .unwrap_or_else(|e| panic!("Failed to read Cargo.toml at {CARGO_TOML_PATH}: {e}"));
    content
        .parse::<toml::Value>()
        .unwrap_or_else(|e| panic!("Failed to parse Cargo.toml as TOML: {e}"))
}

/// Check whether a dependency name appears in EITHER `[dependencies]` OR
/// `[dev-dependencies]`. Returns `true` when the dep is present in either table.
fn dep_exists_in_deps_or_dev_deps(toml: &toml::Value, dep_name: &str) -> bool {
    let in_deps = toml
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .is_some_and(|d| d.contains_key(dep_name));
    let in_dev_deps = toml
        .get("dev-dependencies")
        .and_then(toml::Value::as_table)
        .is_some_and(|d| d.contains_key(dep_name));
    in_deps || in_dev_deps
}

// ---------------------------------------------------------------------------
// B-18: Only allowed dependencies — exact set {tokio, serde_json, vo-types}
// ---------------------------------------------------------------------------

#[test]
fn cargo_toml_dependencies_are_exactly_tokio_serde_json_vo_types() {
    // Given: crates/vo-ipc/Cargo.toml file contents
    let toml = parse_cargo_toml();

    // When: The [dependencies] section is parsed
    let deps_table = toml.get("dependencies").and_then(toml::Value::as_table);

    assert!(
        deps_table.is_some(),
        "Expected [dependencies] section in Cargo.toml"
    );

    let dep_names: std::collections::BTreeSet<String> = deps_table
        .unwrap_or_else(|| panic!("dependencies section missing"))
        .keys()
        .cloned()
        .collect();

    let expected: std::collections::BTreeSet<String> = ["tokio", "serde_json", "vo-types"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Then: The set of dependency names is exactly: {"tokio", "serde_json", "vo-types"}
    assert_eq!(
        dep_names, expected,
        "Dependencies must be exactly {{tokio, serde_json, vo-types}}. Found: {dep_names:?}"
    );
}

// ---------------------------------------------------------------------------
// B-19: No forbidden dependencies — one test per forbidden dep
// ---------------------------------------------------------------------------

#[test]
fn cargo_toml_does_not_contain_fjall() {
    // Given: crates/vo-ipc/Cargo.toml file contents
    let toml = parse_cargo_toml();

    // Then: "fjall" does not appear in [dependencies] OR [dev-dependencies]
    assert!(
        !dep_exists_in_deps_or_dev_deps(&toml, "fjall"),
        "Forbidden dependency 'fjall' found in [dependencies] or [dev-dependencies] (INV-002)"
    );
}

#[test]
fn cargo_toml_does_not_contain_axum() {
    let toml = parse_cargo_toml();

    assert!(
        !dep_exists_in_deps_or_dev_deps(&toml, "axum"),
        "Forbidden dependency 'axum' found in [dependencies] or [dev-dependencies] (INV-003)"
    );
}

#[test]
fn cargo_toml_does_not_contain_ractor() {
    let toml = parse_cargo_toml();

    assert!(
        !dep_exists_in_deps_or_dev_deps(&toml, "ractor"),
        "Forbidden dependency 'ractor' found in [dependencies] or [dev-dependencies] (INV-004)"
    );
}

#[test]
fn cargo_toml_does_not_contain_async_nats() {
    let toml = parse_cargo_toml();

    assert!(
        !dep_exists_in_deps_or_dev_deps(&toml, "async-nats"),
        "Forbidden dependency 'async-nats' found in [dependencies] or [dev-dependencies] (INV-005)"
    );
}

#[test]
fn cargo_toml_does_not_contain_dioxus() {
    let toml = parse_cargo_toml();

    assert!(
        !dep_exists_in_deps_or_dev_deps(&toml, "dioxus"),
        "Forbidden dependency 'dioxus' found in [dependencies] or [dev-dependencies] (INV-006)"
    );
}
