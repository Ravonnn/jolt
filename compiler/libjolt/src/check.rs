//! Full parse → resolve → type-check pipeline for tools and tests.

use jolt_diagnostics::{render_at, render_diagnostic, Severity};
use jolt_source::FileId;

use crate::{CheckResult, CustodyResult, ParseResult, ResolveResult, Session};

/// Combined result of the front-end pipeline for one source file.
#[derive(Debug, Clone)]
pub struct CheckReport {
    pub parse: ParseResult,
    pub resolve: ResolveResult,
    pub check: CheckResult,
    pub custody: CustodyResult,
}

impl CheckReport {
    pub fn is_ok(&self) -> bool {
        self.parse.is_ok() && self.resolve.is_ok() && self.check.is_ok() && self.custody.is_ok()
    }

    /// Render all diagnostics as stable `line:col: error:` lines (parse → resolve → type → custody).
    pub fn render_diagnostics(&self, source: &str) -> Vec<String> {
        let mut lines = Vec::new();
        for err in &self.parse.errors {
            lines.push(render_at(
                Severity::Error,
                err.span.start,
                source,
                &err.message,
                None,
            ));
        }
        for err in &self.resolve.errors {
            lines.push(render_at(
                Severity::Error,
                err.span.start,
                source,
                &err.message,
                None,
            ));
        }
        for diag in &self.check.diagnostics.items {
            lines.push(render_diagnostic(diag, source));
        }
        for diag in &self.custody.diagnostics.items {
            lines.push(render_diagnostic(diag, source));
        }
        lines
    }
}

impl Session {
    /// Run parse → resolve → type-check → custody on `source` (memoized per stage).
    pub fn check_source(&mut self, file: FileId, source: &str) -> CheckReport {
        let parse = self.parse_file(file, source);
        let resolve = self.resolve_file(file, source);
        let check = self.check_file(file, source);
        let custody = self.custody_file(file, source);
        CheckReport {
            parse,
            resolve,
            check,
            custody,
        }
    }
}
