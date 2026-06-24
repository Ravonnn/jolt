//! Name resolver for the Jolt **Tiny** subset.
//!
//! Implements lexical scopes, `$` / `$$` binding rules, and bare reassignment
//! (`docs/spec/jolt-spec-v0.4.md` §4). Part of year-1 Phase 1 workstream 3
//! (`docs/design/year-1.md`).

mod error;
mod query;
mod resolve;
mod scope;

#[cfg(test)]
mod tests;

pub use error::{ResolveError, ResolveErrorKind};
pub use jolt_ast;
pub use jolt_parser;
pub use query::{resolve_file, resolve_parsed, RESOLVE_FILE};
pub use resolve::resolve_program;
pub use scope::{BindingInfo, BindingOrigin, SymbolId};

use jolt_ast::Program;

/// Result of resolving names in a program.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ResolveResult {
    pub program: Program,
    pub errors: Vec<ResolveError>,
    pub symbols: Vec<BindingInfo>,
}

impl ResolveResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

pub const STAGE: &str = "resolve";
