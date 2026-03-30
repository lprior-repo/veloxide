//! Cargo.toml contract tests for vo-ipc crate.
//!
//! Tests B-22, B-23, B-24, B-25: Workspace convention fields, dependency
//! inheritance syntax, crate name verification, and dev-dependency presence.
//!
//! Uses the `toml` crate for structured TOML parsing (not raw string matching).
//!
//! Dependency allowlist and forbidden-dep tests live in `cargo_toml_deps_contract.rs`.

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

// ---------------------------------------------------------------------------
// B-24: Crate name is vo-ipc
// ---------------------------------------------------------------------------

#[test]
fn cargo_toml_package_name_is_vo_ipc() {
    // Given: crates/vo-ipc/Cargo.toml file contents
    let toml = parse_cargo_toml();

    // When: The [package] section is parsed
    // Then: package.name == "vo-ipc"
    let name = toml["package"]["name"].as_str();
    assert_eq!(
        name,
        Some("vo-ipc"),
        "Expected package.name == \"vo-ipc\", found: {name:?}"
    );
}

// ---------------------------------------------------------------------------
// B-22: Workspace convention fields — version, edition, license
// ---------------------------------------------------------------------------

#[test]
fn cargo_toml_version_uses_workspace_inheritance() {
    // Given: crates/vo-ipc/Cargo.toml file contents
    let toml = parse_cargo_toml();

    // When: The [package] section is parsed
    // Then: package.version.workspace == true
    let version_workspace = toml
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.get("workspace"))
        .and_then(toml::Value::as_bool);

    assert_eq!(
        version_workspace,
        Some(true),
        "Expected package.version.workspace = true. \
         The version field must use workspace inheritance, not a hardcoded version string."
    );
}

#[test]
fn cargo_toml_edition_uses_workspace_inheritance() {
    // Given: crates/vo-ipc/Cargo.toml file contents
    let toml = parse_cargo_toml();

    // When: The [package] section is parsed
    // Then: package.edition.workspace == true
    let edition_workspace = toml
        .get("package")
        .and_then(|p| p.get("edition"))
        .and_then(|v| v.get("workspace"))
        .and_then(toml::Value::as_bool);

    assert_eq!(
        edition_workspace,
        Some(true),
        "Expected package.edition.workspace = true. \
         The edition field must use workspace inheritance, not a hardcoded edition string."
    );
}

#[test]
fn cargo_toml_license_uses_workspace_inheritance() {
    // Given: crates/vo-ipc/Cargo.toml file contents
    let toml = parse_cargo_toml();

    // When: The [package] section is parsed
    // Then: package.license.workspace == true
    let license_workspace = toml
        .get("package")
        .and_then(|p| p.get("license"))
        .and_then(|v| v.get("workspace"))
        .and_then(toml::Value::as_bool);

    assert_eq!(
        license_workspace,
        Some(true),
        "Expected package.license.workspace = true. \
         The license field must use workspace inheritance, not a hardcoded license string."
    );
}

// ---------------------------------------------------------------------------
// B-23: Dependency inheritance syntax — workspace = true vs path
// ---------------------------------------------------------------------------

#[test]
fn cargo_toml_tokio_uses_workspace_inheritance() {
    // Given: crates/vo-ipc/Cargo.toml file contents
    let toml = parse_cargo_toml();

    // When: The [dependencies] section is parsed
    // Then: The `tokio` dependency entry contains `workspace = true`
    let tokio_workspace = toml
        .get("dependencies")
        .and_then(|d| d.get("tokio"))
        .and_then(|t| t.get("workspace"))
        .and_then(toml::Value::as_bool);

    assert_eq!(
        tokio_workspace,
        Some(true),
        "Expected dependencies.tokio.workspace = true (INV-012). \
         tokio must use workspace inheritance, not a hardcoded version string."
    );
}

#[test]
fn cargo_toml_serde_json_uses_workspace_inheritance() {
    let toml = parse_cargo_toml();

    let serde_json_workspace = toml
        .get("dependencies")
        .and_then(|d| d.get("serde_json"))
        .and_then(|t| t.get("workspace"))
        .and_then(toml::Value::as_bool);

    assert_eq!(
        serde_json_workspace,
        Some(true),
        "Expected dependencies.serde_json.workspace = true (INV-012). \
         serde_json must use workspace inheritance, not a hardcoded version string."
    );
}

#[test]
fn cargo_toml_vo_types_uses_path_dependency() {
    let toml = parse_cargo_toml();

    // Then: The `vo-types` dependency entry contains `path = "../vo-types"`
    let vo_types_path = toml
        .get("dependencies")
        .and_then(|d| d.get("vo-types"))
        .and_then(|t| t.get("path"))
        .and_then(toml::Value::as_str);

    assert_eq!(
        vo_types_path,
        Some("../vo-types"),
        "Expected dependencies.vo-types.path = \"../vo-types\" (INV-012). \
         vo-types must use path dependency syntax for local workspace crate."
    );

    // And: vo-types must NOT use workspace = true (it's a path dep)
    let vo_types_workspace = toml
        .get("dependencies")
        .and_then(|d| d.get("vo-types"))
        .and_then(|t| t.get("workspace"))
        .and_then(toml::Value::as_bool);

    assert_ne!(
        vo_types_workspace,
        Some(true),
        "vo-types must use path dependency, not workspace = true"
    );
}

// ---------------------------------------------------------------------------
// B-25: Dev-dependencies include rstest and proptest with workspace inheritance
// ---------------------------------------------------------------------------

#[test]
fn cargo_toml_dev_deps_include_rstest_with_workspace() {
    // Given: crates/vo-ipc/Cargo.toml file contents
    let toml = parse_cargo_toml();

    // When: The [dev-dependencies] section is parsed
    // Then: The dev-dependency names include "rstest"
    let rstest_workspace = toml
        .get("dev-dependencies")
        .and_then(|d| d.get("rstest"))
        .and_then(|t| t.get("workspace"))
        .and_then(toml::Value::as_bool);

    assert_eq!(
        rstest_workspace,
        Some(true),
        "Expected dev-dependencies.rstest.workspace = true. \
         rstest must be present with workspace inheritance in [dev-dependencies]."
    );
}

#[test]
fn cargo_toml_dev_deps_include_proptest_with_workspace() {
    let toml = parse_cargo_toml();

    let proptest_workspace = toml
        .get("dev-dependencies")
        .and_then(|d| d.get("proptest"))
        .and_then(|t| t.get("workspace"))
        .and_then(toml::Value::as_bool);

    assert_eq!(
        proptest_workspace,
        Some(true),
        "Expected dev-dependencies.proptest.workspace = true. \
         proptest must be present with workspace inheritance in [dev-dependencies]."
    );
}
