#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

// ADR-020: Procedural workflow static linter.
// Implemented as syn AST visitors over workflow function bodies.
// Rules WTF-L001 through WTF-L006 — see individual rule modules.

pub mod diagnostic;
pub mod l005;
pub mod l006;
pub mod rules;
pub mod visitor;

pub use diagnostic::{Diagnostic, LintCode, LintError, Severity};
pub use l005::lint_workflow_code;
pub use l006::lint_workflow_code as lint_workflow_code_l006;

use std::collections::HashSet;

pub struct LintResult {
    pub diagnostics: Vec<Diagnostic>,
    pub has_errors: bool,
}

impl LintResult {
    #[must_use]
    pub fn new(diagnostics: Vec<Diagnostic>) -> Self {
        let has_errors = !diagnostics.is_empty();
        Self {
            diagnostics,
            has_errors,
        }
    }
}

/// Lint workflow source code with all registered rules (L001-L006).
///
/// # Errors
/// Returns `LintError::ParseError` if the source cannot be parsed.
pub fn lint_workflow_source(source: &str) -> Result<LintResult, LintError> {
    let mut all_diagnostics: Vec<Diagnostic> = Vec::new();
    let mut seen_codes: HashSet<String> = HashSet::new();

    let l005_result = l005::lint_workflow_code(source)?;
    for diag in l005_result {
        let code_str = diag.code.to_string();
        if seen_codes.insert(code_str.clone()) {
            all_diagnostics.push(diag);
        }
    }

    let l006_result = l006::lint_workflow_code(source)?;
    for diag in l006_result {
        let code_str = diag.code.to_string();
        if seen_codes.insert(code_str.clone()) {
            all_diagnostics.push(diag);
        }
    }

    Ok(LintResult::new(all_diagnostics))
}
