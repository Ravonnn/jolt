//! Tiny Custodian pass — move analysis (`docs/spec/jolt-spec-v0.4.md` §9).
//!
//! Phase 2a: use-after-move for non-`Copy` bindings (`String` moves; `Int`/`Bool`/`None` copy).

mod check;
mod error;
mod liveness;
mod query;

#[cfg(test)]
mod tests;

pub use check::check_program;
pub use error::CustodyErrorKind;
pub use jolt_ast;
pub use jolt_diagnostics;
pub use jolt_types;
pub use query::{custody_checked, custody_file, CUSTODY_FILE};

use jolt_ast::Program;
use jolt_diagnostics::Diagnostics;

/// Result of custody analysis on a program.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct CustodyResult {
    pub program: Program,
    pub diagnostics: Diagnostics,
}

impl CustodyResult {
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub const STAGE: &str = "custody";
