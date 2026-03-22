//! Integration tests for WTF-L001 non-deterministic time detection.

use wtf_linter::diagnostic::{Diagnostic, LintCode, LintError};

fn check(source: &str) -> Result<Vec<Diagnostic>, LintError> {
    // Re-exported from lib.rs as the main linting entry point
    // Note: This uses the full linter - individual rule functions are internal
    wtf_linter::lint_workflow_source(source).map(|r| r.diagnostics)
}

#[test]
fn test_emits_no_diagnostic_for_code_without_time_calls() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = ctx.now();
    Ok(())
}
"#;
    let result = check(source);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_emits_diagnostic_when_chrono_utc_now_found() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = chrono::Utc::now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, LintCode::L001);
}

#[test]
fn test_emits_diagnostic_when_chrono_local_now_found() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = chrono::Local::now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, LintCode::L001);
}

#[test]
fn test_emits_diagnostic_when_system_time_now_found() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = std::time::SystemTime::now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, LintCode::L001);
}

#[test]
fn test_emits_diagnostic_when_instant_now_found() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = std::time::Instant::now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, LintCode::L001);
}

#[test]
fn test_emits_diagnostic_when_tokio_instant_now_found() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = tokio::time::Instant::now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, LintCode::L001);
}

#[test]
fn test_emits_no_diagnostic_when_ctx_now_found() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = ctx.now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert!(result.is_empty());
}

#[test]
fn test_emits_multiple_diagnostics_for_multiple_time_calls() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let a = chrono::Utc::now();
    let b = std::time::SystemTime::now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|d| d.code == LintCode::L001));
}

#[test]
fn test_diagnostic_code_is_wtf_l001() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = chrono::Utc::now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert_eq!(result[0].code.as_str(), "WTF-L001");
}

#[test]
fn test_diagnostic_message_contains_non_deterministic() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = chrono::Utc::now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert!(result[0].message.contains("non-deterministic"));
}

#[test]
fn test_diagnostic_suggestion_contains_ctx_now() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = chrono::Utc::now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert!(result[0]
        .suggestion
        .as_ref()
        .is_some_and(|s| s.contains("ctx.now()")));
}

#[test]
fn test_returns_parse_error_for_invalid_rust() {
    let source = "async fn workflow { // missing parentheses";
    let result = check(source);
    assert!(result.is_err());
}

#[test]
fn test_emits_diagnostic_for_bare_utc_now() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = Utc::now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, LintCode::L001);
}

#[test]
fn test_emits_diagnostic_for_bare_local_now() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = Local::now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, LintCode::L001);
}

#[test]
fn test_emits_diagnostic_for_deep_chrono_path() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = some::deep::chrono::Utc::now();
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, LintCode::L001);
}

#[test]
fn test_macro_does_not_expand_vec_chrono_utc_now() {
    let source = r#"
async fn workflow(ctx: &Ctx) -> Result<(), Error> {
    let t = vec![chrono::Utc::now()];
    Ok(())
}
"#;
    let result = check(source).expect("should parse");
    assert_eq!(result.len(), 0);
}
