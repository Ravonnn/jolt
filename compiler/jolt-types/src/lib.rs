//! Tiny type checker for Jolt (`docs/spec/jolt-spec-v0.4.md` §5).
//!
//! Part of year-1 Phase 1 workstream 4 (`docs/design/year-1.md`). Checks the Tiny subset:
//! `Int`, `Bool`, `None`, `String`, operators, functions, and control flow.

mod check;
mod error;
mod query;
mod ty;

#[cfg(test)]
mod tests;

pub use check::check_program;
pub use error::TypeErrorKind;
pub use jolt_ast;
pub use jolt_diagnostics;
pub use jolt_resolve;
pub use query::{check_file, check_resolved, CHECK_FILE};
pub use ty::Ty;

use jolt_ast::Program;
use jolt_diagnostics::Diagnostics;

/// Result of type-checking a program.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct CheckResult {
    pub program: Program,
    pub diagnostics: Diagnostics,
}

impl CheckResult {
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub const STAGE: &str = "types";
