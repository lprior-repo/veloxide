#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::diagnostic::{Diagnostic, LintCode};
use syn::{spanned::Spanned, visit::Visit, Expr, ExprMethodCall, Path};

const SUGGESTION: &str = "wrap in ctx.activity(...)";

pub struct DirectAsyncIoVisitor {
    pub diagnostics: Vec<Diagnostic>,
}

impl DirectAsyncIoVisitor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    #[must_use]
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    fn emit_diagnostic(&mut self, span: Option<(usize, usize)>) {
        let diagnostic = Diagnostic::new(LintCode::L003, "direct async I/O in workflow function")
            .with_suggestion(SUGGESTION);
        let diagnostic = match span {
            Some((start, end)) => Diagnostic {
                span: Some((start, end)),
                ..diagnostic
            },
            None => diagnostic,
        };
        self.diagnostics.push(diagnostic);
    }

    fn check_path(&mut self, path: &Path) {
        let path_str = path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        if path_str.contains("reqwest")
            || path_str.contains("sqlx")
            || path_str.contains("tokio::fs")
        {
            self.emit_diagnostic(Some((path.span().start().column, path.span().end().column)));
        }
    }

    fn check_method(&mut self, expr: &ExprMethodCall) {
        let method = expr.method.to_string();
        if matches!(
            method.as_str(),
            "fetch_one" | "fetch_optional" | "fetch_all" | "post" | "get"
        ) {
            // We'll catch these via their receiver paths or direct call checks
        }
    }
}

impl<'ast> Visit<'ast> for DirectAsyncIoVisitor {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        match expr {
            Expr::Call(call) => {
                if let Expr::Path(path_expr) = call.func.as_ref() {
                    self.check_path(&path_expr.path);
                }
            }
            Expr::MethodCall(method) => {
                self.check_method(method);
            }
            Expr::Path(path_expr) => {
                self.check_path(&path_expr.path);
            }
            _ => {}
        }
        syn::visit::visit_expr(self, expr);
    }
}

pub fn check_direct_async_io(
    source: &str,
) -> Result<Vec<Diagnostic>, crate::diagnostic::LintError> {
    let file = syn::parse_file(source)
        .map_err(|e| crate::diagnostic::LintError::ParseError(e.to_string()))?;
    let mut visitor = DirectAsyncIoVisitor::new();
    visitor.visit_file(&file);
    // De-duplicate diagnostics on the same line/span
    let mut unique = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for d in visitor.diagnostics {
        if let Some(span) = d.span {
            if seen.insert(span) {
                unique.push(d);
            }
        } else {
            unique.push(d);
        }
    }
    Ok(unique)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_emits_no_diagnostic_for_code_without_async_io() {
        let s = "async fn wf(ctx: &Ctx) { ctx.activity(\"x\", ()).await; }";
        assert!(check_direct_async_io(s).unwrap().is_empty());
    }
    #[test]
    fn test_emits_diagnostic_for_reqwest_get_call() {
        let s = "async fn wf() { reqwest::get(\"url\").await; }";
        assert_eq!(check_direct_async_io(s).unwrap().len(), 1);
    }
    #[test]
    fn test_emits_diagnostic_for_sqlx_query_fetch_one() {
        let s = "async fn wf(pool: &Pool) { sqlx::query(\"...\").fetch_one(pool).await; }";
        assert_eq!(check_direct_async_io(s).unwrap().len(), 1);
    }
}
