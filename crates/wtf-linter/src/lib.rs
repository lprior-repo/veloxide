#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

// ADR-020: Procedural workflow static linter.
// Implemented as syn AST visitors over workflow function bodies.
// Rules WTF-L001 through WTF-L006 — see individual rule modules.

pub mod diagnostic;
pub mod l001_time;
pub mod l004;
pub mod l005;
pub mod l006;
pub mod rules;
pub mod visitor;

pub use diagnostic::{Diagnostic, LintCode, LintError, Severity};
pub use l001_time::lint_workflow_code as lint_workflow_code_l001;
pub use l004::lint_workflow_code as lint_workflow_code_l004;
pub use l005::lint_workflow_code as lint_workflow_code_l005;
pub use l006::lint_workflow_code as lint_workflow_code_l006;

pub fn lint_workflow_code(source: &str) -> Result<Vec<Diagnostic>, LintError> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(l001_time::lint_workflow_code(source)?);
    diagnostics.extend(l004::lint_workflow_code(source)?);
    diagnostics.extend(l005::lint_workflow_code(source)?);
    diagnostics.extend(l006::lint_workflow_code(source)?);
    Ok(diagnostics)
}
