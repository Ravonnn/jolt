//! Canonical formatter for the Jolt **Tiny** subset (parse → print).
//!
//! Part of year-1 Phase 1 workstream 5 (`docs/design/year-1.md`). Formatting is
//! **parse-driven** only — it does not require resolve or type-check.

mod printer;
mod query;

#[cfg(test)]
mod tests;

pub use jolt_ast;
pub use jolt_parser;
pub use printer::print_program;
pub use query::{format_file, format_parsed, format_program, FMT_FILE};

use jolt_parser::ParseError;

/// Result of formatting a source file.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FormatResult {
    pub source: String,
    pub errors: Vec<ParseError>,
}

impl FormatResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

pub const STAGE: &str = "fmt";
