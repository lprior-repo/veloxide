use wtf_linter::{check_random_in_workflow, LintCode, Severity};

fn lint(source: &str) -> Vec<wtf_linter::Diagnostic> {
    let file = syn::parse_file(source).expect("parse error");
    check_random_in_workflow(&file)
}

#[test]
fn test_uuid_new_v4_detected() {
    let source = r#"
        async fn checkout(ctx: &Ctx) -> Result<OrderId> {
            let order_id = uuid::Uuid::new_v4();
            Ok(order_id)
        }
    "#;
    let diagnostics = lint(source);
    assert!(
        !diagnostics.is_empty(),
        "Expected L002 diagnostic for uuid::Uuid::new_v4()"
    );
    let d = &diagnostics[0];
    assert_eq!(d.code, LintCode::L002);
    assert_eq!(d.severity, Severity::Error);
    assert!(d.message.contains("non-deterministic random"));
}

#[test]
fn test_rand_random_detected() {
    let source = r#"
        async fn session(ctx: &Ctx) -> Result<u64> {
            let token: u64 = rand::random();
            Ok(token)
        }
    "#;
    let diagnostics = lint(source);
    assert!(
        !diagnostics.is_empty(),
        "Expected L002 diagnostic for rand::random()"
    );
    let d = &diagnostics[0];
    assert_eq!(d.code, LintCode::L002);
}

#[test]
fn test_rand_random_with_type_detected() {
    let source = r#"
        async fn session(ctx: &Ctx) -> Result<u128> {
            let token: u128 = rand::random::<u128>();
            Ok(token)
        }
    "#;
    let diagnostics = lint(source);
    assert!(
        !diagnostics.is_empty(),
        "Expected L002 diagnostic for rand::random::<T>()"
    );
}

#[test]
fn test_ctx_random_u64_not_flagged() {
    let source = r#"
        async fn workflow(ctx: &Ctx) -> Result<u64> {
            let nonce = ctx.random_u64();
            Ok(nonce)
        }
    "#;
    let diagnostics = lint(source);
    assert!(
        diagnostics.is_empty(),
        "ctx.random_u64() should not be flagged"
    );
}

#[test]
fn test_uuid_nil_not_flagged() {
    let source = r#"
        async fn workflow(ctx: &Ctx) -> Result<Uuid> {
            let id = uuid::Uuid::nil();
            Ok(id)
        }
    "#;
    let diagnostics = lint(source);
    assert!(
        diagnostics.is_empty(),
        "uuid::Uuid::nil() should not be flagged"
    );
}

#[test]
fn test_multiple_violations() {
    let source = r#"
        async fn workflow(ctx: &Ctx) -> Result<(Uuid, u64)> {
            let id = uuid::Uuid::new_v4();
            let n: u32 = rand::random();
            Ok((id, n as u64))
        }
    "#;
    let diagnostics = lint(source);
    assert_eq!(diagnostics.len(), 2, "Expected 2 diagnostics");
    assert!(diagnostics.iter().all(|d| d.code == LintCode::L002));
}
