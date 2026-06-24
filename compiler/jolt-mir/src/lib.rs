//! MIR lowering for the Jolt Tiny subset (Phase 3a).

mod lower;
mod mir;
mod query;

#[cfg(test)]
mod tests;

pub use lower::lower_program;
pub use mir::{LocalId, MirFn, MirInstr, MirModule};
pub use query::{mir_custodied, mir_file, MIR_FILE};

use jolt_ast::Program;
use jolt_diagnostics::Diagnostics;

/// Result of lowering a program to MIR.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct MirResult {
    pub program: Program,
    pub module: MirModule,
    pub diagnostics: Diagnostics,
}

impl MirResult {
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn has_entry(&self) -> bool {
        self.module.entry_fn().is_some()
    }
}

pub const STAGE: &str = "mir";
