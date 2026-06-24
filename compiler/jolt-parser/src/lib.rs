//! Parser for the Jolt **Tiny** subset.
//!
//! Hand-written **recursive descent** parser with layered precedence for expressions
//! (`docs/spec/jolt-grammar.md` §9). Blocks and control flow follow §2–§7.
//!
//! Error recovery synchronizes on `;`, `;;`, `->`, or `@` so partial ASTs are returned
//! for LSP use.

mod error;
mod parser;
mod query;

pub use error::{ParseError, ParseErrorKind};
pub use jolt_ast;
pub use parser::parse_tokens;
pub use query::{parse_file, parse_source, PARSE_FILE};

use jolt_ast::Program;

/// Result of parsing: a (possibly partial) program plus non-fatal errors.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ParseResult {
    pub program: Program,
    pub errors: Vec<ParseError>,
}

impl ParseResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

pub const STAGE: &str = "parser";

#[cfg(test)]
mod tests;
