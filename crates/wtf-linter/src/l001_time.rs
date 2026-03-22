#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use crate::diagnostic::{Diagnostic, LintCode, LintError};
use syn::visit::Visit;
use syn::{spanned::Spanned, Expr, Path};

const SUGGESTION: &str =
    "use ctx.now() instead — returns a logged timestamp that is consistent on replay";

pub fn lint_workflow_code(source: &str) -> Result<Vec<Diagnostic>, LintError> {
    let syntax_tree = syn::parse_file(source).map_err(|e| LintError::ParseError(e.to_string()))?;
    let mut visitor = L001Visitor::new();
    visitor.visit_file(&syntax_tree);
    Ok(visitor.diagnostics)
}

struct L001Visitor {
    diagnostics: Vec<Diagnostic>,
}

impl L001Visitor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    fn emit_diagnostic(&mut self, span: Option<(usize, usize)>) {
        let mut diagnostic = Diagnostic::new(
            LintCode::L001,
            "non-deterministic time call in workflow function",
        )
        .with_suggestion(SUGGESTION);
        if let Some((start, end)) = span {
            diagnostic.span = Some((start, end));
        }
        self.diagnostics.push(diagnostic);
    }

    fn is_time_now_call(path: &Path) -> bool {
        let segments = &path.segments;
        if segments.is_empty() {
            return false;
        }
        let last_seg = &segments[segments.len() - 1];
        if last_seg.ident != "now" {
            return false;
        }
        if segments.len() == 3 && segments[0].ident == "chrono" && segments[1].ident == "Utc" {
            return true;
        }
        if segments.len() == 3 && segments[0].ident == "chrono" && segments[1].ident == "Local" {
            return true;
        }
        if segments.len() == 4
            && segments[0].ident == "std"
            && segments[1].ident == "time"
            && (segments[2].ident == "SystemTime" || segments[2].ident == "Instant")
        {
            return true;
        }
        if segments.len() == 4
            && segments[0].ident == "tokio"
            && segments[1].ident == "time"
            && segments[2].ident == "Instant"
        {
            return true;
        }
        false
    }

    fn is_time_now_method(expr: &syn::ExprMethodCall) -> bool {
        if expr.method == "now" {
            if let Expr::Path(path_expr) = expr.receiver.as_ref() {
                let path = &path_expr.path;
                return path.segments.len() == 2
                    && ((path.segments[0].ident == "std"
                        && path.segments[1].ident == "SystemTime")
                        || (path.segments[0].ident == "std"
                            && path.segments[1].ident == "Instant")
                        || (path.segments[0].ident == "chrono"
                            && path.segments[1].ident == "Utc")
                        || (path.segments[0].ident == "chrono"
                            && path.segments[1].ident == "Local")
                        || (path.segments[0].ident == "tokio"
                            && path.segments[1].ident == "time"));
            }
        }
        false
    }
}

impl Default for L001Visitor {
    fn default() -> Self {
        Self::new()
    }
}

impl<'ast> syn::visit::Visit<'ast> for L001Visitor {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        match expr {
            Expr::Call(call_expr) => {
                if let Expr::Path(path_expr) = call_expr.func.as_ref() {
                    if Self::is_time_now_call(&path_expr.path) {
                        self.emit_diagnostic(Some(loc_of(expr)));
                    }
                }
            }
            Expr::MethodCall(method_expr) => {
                if Self::is_time_now_method(method_expr) {
                    self.emit_diagnostic(Some(loc_of(expr)));
                }
            }
            _ => {}
        }
        syn::visit::visit_expr(self, expr);
    }
}

fn loc_of(expr: &Expr) -> (usize, usize) {
    (expr.span().start().column, expr.span().end().column)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(source: &str) -> Result<Vec<Diagnostic>, LintError> {
        lint_workflow_code(source)
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
}
