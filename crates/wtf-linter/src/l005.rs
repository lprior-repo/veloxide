#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::diagnostic::{Diagnostic, LintCode, LintError, Severity};
use syn::spanned::Spanned;
use syn::{Expr, ItemImpl};

pub fn lint_workflow_code(source: &str) -> Result<Vec<Diagnostic>, LintError> {
    let syntax_tree = syn::parse_file(source).map_err(LintError::ParseError)?;
    let mut collector = L005Visitor::new();
    collector.visit_file(&syntax_tree);
    Ok(collector.diagnostics)
}

struct L005Visitor {
    diagnostics: Vec<Diagnostic>,
    inside_workflow_fn: bool,
}

impl L005Visitor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            inside_workflow_fn: false,
        }
    }

    fn is_workflow_impl(&self, impl_item: &ItemImpl) -> bool {
        for item in &impl_item.items {
            if let syn::ImplItem::Fn(impl_fn) = item {
                if impl_fn.sig.ident == "execute" && impl_fn.sig.asyncness.is_some() {
                    return true;
                }
            }
        }
        false
    }

    fn check_expr_for_tokio_spawn(&mut self, expr: &Expr) {
        match expr {
            Expr::Call(call) => {
                if let Expr::Path(path_expr) = &*call.func {
                    if is_tokio_spawn_path(&path_expr.path) {
                        let span = call.span();
                        let (start, end) = (span.start().column, span.end().column);
                        self.diagnostics.push(
                            Diagnostic::new(LintCode::L005, "tokio::spawn() is not allowed inside a procedural workflow function. Spawned tasks detach from the workflow context and violate determinism.")
                                .with_suggestion("Use workflow context's spawn method or convert to a child activity instead.")
                                .with_span((start, end)),
                        );
                    }
                }
                for arg in &call.args {
                    self.check_expr_for_tokio_spawn(arg);
                }
            }
            Expr::Async(async_expr) => {
                self.check_expr_for_tokio_spawn(&async_expr.block);
            }
            Expr::Block(block) => {
                for stmt in &block.stmts {
                    self.visit_stmt(stmt);
                }
            }
            Expr::If(if_expr) => {
                self.check_expr_for_tokio_spawn(&if_expr.cond);
                self.check_expr_for_tokio_spawn(&if_expr.then_branch);
                if let Some((_, else_branch)) = &if_expr.else_branch {
                    self.check_expr_for_tokio_spawn(else_branch);
                }
            }
            Expr::Match(match_expr) => {
                for arm in &match_expr.arms {
                    self.check_expr_for_tokio_spawn(&arm.body);
                }
            }
            Expr::Loop(loop_expr) => {
                self.check_expr_for_tokio_spawn(&loop_expr.body);
            }
            Expr::ForLoop(for_expr) => {
                self.check_expr_for_tokio_spawn(&for_expr.body);
            }
            Expr::While(while_expr) => {
                self.check_expr_for_tokio_spawn(&while_expr.body);
            }
            Expr::Closure(closure_expr) => {
                self.check_expr_for_tokio_spawn(&closure_expr.body);
            }
            Expr::Return(ret_expr) => {
                if let Some(expr) = &ret_expr.expr {
                    self.check_expr_for_tokio_spawn(expr);
                }
            }
            Expr::Let(let_expr) => {
                self.check_expr_for_tokio_spawn(&let_expr.expr);
            }
            Expr::Assign(assign_expr) => {
                self.check_expr_for_tokio_spawn(&assign_expr.left);
                self.check_expr_for_tokio_spawn(&assign_expr.right);
            }
            Expr::AssignOp(assign_op_expr) => {
                self.check_expr_for_tokio_spawn(&assign_op_expr.left);
                self.check_expr_for_tokio_spawn(&assign_op_expr.right);
            }
            Expr::MethodCall(method_call) => {
                for arg in &method_call.args {
                    self.check_expr_for_tokio_spawn(arg);
                }
            }
            Expr::Tuple(tuple_expr) => {
                for elem in &tuple_expr.elems {
                    self.check_expr_for_tokio_spawn(elem);
                }
            }
            Expr::Array(array_expr) => {
                for elem in &array_expr.elems {
                    self.check_expr_for_tokio_spawn(elem);
                }
            }
            Expr::Cast(cast_expr) => {
                self.check_expr_for_tokio_spawn(&cast_expr.expr);
            }
            Expr::Unary(unary_expr) => {
                self.check_expr_for_tokio_spawn(&unary_expr.expr);
            }
            Expr::Binary(binary_expr) => {
                self.check_expr_for_tokio_spawn(&binary_expr.left);
                self.check_expr_for_tokio_spawn(&binary_expr.right);
            }
            Expr::Break(break_expr) => {
                if let Some(expr) = &break_expr.expr {
                    self.check_expr_for_tokio_spawn(expr);
                }
            }
            Expr::Continue(_) => {}
            Expr::Reference(reference_expr) => {
                self.check_expr_for_tokio_spawn(&reference_expr.expr);
            }
            Expr::Field(field_expr) => {
                self.check_expr_for_tokio_spawn(&field_expr.expr);
            }
            Expr::Index(index_expr) => {
                self.check_expr_for_tokio_spawn(&index_expr.expr);
                self.check_expr_for_tokio_spawn(&index_expr.index);
            }
            Expr::Range(range_expr) => {
                if let Some(start) = &range_expr.start {
                    self.check_expr_for_tokio_spawn(start);
                }
                if let Some(end) = &range_expr.end {
                    self.check_expr_for_tokio_spawn(end);
                }
            }
            Expr::Paren(paren_expr) => {
                self.check_expr_for_tokio_spawn(&paren_expr.expr);
            }
            Expr::Try(try_expr) => {
                self.check_expr_for_tokio_spawn(&try_expr.expr);
            }
            Expr::TryBlock(try_block) => {
                for stmt in &try_block.block.stmts {
                    self.visit_stmt(stmt);
                }
            }
            Expr::Yield(yield_expr) => {
                if let Some(expr) = &yield_expr.expr {
                    self.check_expr_for_tokio_spawn(expr);
                }
            }
            Expr::Struct(_) | Expr::Repeat(_) | Expr::Verbatim(_) | Expr::Mac(_) => {}
        }
    }

    fn visit_stmt(&mut self, stmt: &syn::Stmt) {
        match stmt {
            syn::Stmt::Local(local) => {
                if let Some(expr) = &local.init {
                    self.check_expr_for_tokio_spawn(&expr.1);
                }
            }
            syn::Stmt::Expr(expr) => {
                self.check_expr_for_tokio_spawn(expr);
            }
            syn::Stmt::Item(_) => {}
        }
    }
}

fn is_tokio_spawn_path(path: &syn::Path) -> bool {
    path.segments.len() == 2
        && path.segments[0].ident == "tokio"
        && path.segments[1].ident == "spawn"
}

impl Diagnostic {
    pub fn with_span(mut self, span: (usize, usize)) -> Self {
        self.span = Some(span);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lints_clean_code_with_no_violations() {
        let source = r#"
async fn helper() {
    let x = 42;
}
"#;
        let result = lint_workflow_code(source);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_returns_parse_error_for_invalid_rust() {
        let source = "not valid rust {";
        let result = lint_workflow_code(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_handles_empty_source() {
        let source = "";
        let result = lint_workflow_code(source);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_no_false_positive_outside_workflow() {
        let source = r#"
async fn helper_function() {
    tokio::spawn(async {
        println!("helper task");
    });
}
"#;
        let result = lint_workflow_code(source);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_no_false_positive_different_spawn() {
        let source = r#"
async fn execute() {
    let handle = std::thread::spawn(|| {
        println!("thread");
    });
}
"#;
        let result = lint_workflow_code(source);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_violation_tokio_spawn_in_workflow() {
        let source = r#"
async fn execute() {
    tokio::spawn(async {
        println!("detached task");
    });
}
"#;
        let result = lint_workflow_code(source);
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, LintCode::L005);
    }

    #[test]
    fn test_violation_nested_tokio_spawn() {
        let source = r#"
async fn execute() {
    let _ = if true {
        tokio::spawn(async {
            do_work().await;
        })
    } else {
        42
    };
}
"#;
        let result = lint_workflow_code(source);
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, LintCode::L005);
    }

    #[test]
    fn test_multiple_tokio_spawns() {
        let source = r#"
async fn execute() {
    tokio::spawn(async { println!("first"); });
    tokio::spawn(async { println!("second"); });
}
"#;
        let result = lint_workflow_code(source);
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn test_tokio_spawn_in_closure() {
        let source = r#"
async fn execute() {
    let numbers = vec![1, 2, 3];
    numbers.iter().for_each(|_| {
        tokio::spawn(async { println!("in closure"); });
    });
}
"#;
        let result = lint_workflow_code(source);
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn test_no_false_positive_qualified_tokio_spawn() {
        let source = r#"
async fn helper() {
    some_other::spawn(async { });
}
"#;
        let result = lint_workflow_code(source);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
