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

/// Lints workflow code for non-deterministic time calls.
///
/// # Errors
/// Returns `LintError::ParseError` if the source cannot be parsed.
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
        let len = segments.len();
        let last_seg = &segments[len - 1];
        if last_seg.ident != "now" {
            return false;
        }
        // 2-segment bare paths: Utc::now, Local::now, SystemTime::now, Instant::now
        // (without chrono:: or std::time:: prefix)
        if len == 2
            && (segments[0].ident == "Utc"
                || segments[0].ident == "Local"
                || segments[0].ident == "SystemTime"
                || segments[0].ident == "Instant")
        {
            return true;
        }
        // Suffix match for chrono::*::now (last 3 segments must be chrono::Utc::now or chrono::Local::now)
        if len >= 3
            && segments[len - 3].ident == "chrono"
            && (segments[len - 2].ident == "Utc" || segments[len - 2].ident == "Local")
        {
            return true;
        }
        // Suffix match for std::time::*::now (last 4 segments must be std::time::SystemTime::now or std::time::Instant::now)
        if len >= 4
            && segments[len - 4].ident == "std"
            && segments[len - 3].ident == "time"
            && (segments[len - 2].ident == "SystemTime" || segments[len - 2].ident == "Instant")
        {
            return true;
        }
        // Suffix match for tokio::time::Instant::now (last 4 segments)
        if len >= 4
            && segments[len - 4].ident == "tokio"
            && segments[len - 3].ident == "time"
            && segments[len - 2].ident == "Instant"
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
                    && (path.segments[0].ident == "std"
                        && (path.segments[1].ident == "SystemTime"
                            || path.segments[1].ident == "Instant"))
                    || (path.segments[0].ident == "chrono"
                        && (path.segments[1].ident == "Utc" || path.segments[1].ident == "Local"))
                    || (path.segments[0].ident == "tokio" && path.segments[1].ident == "time");
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
