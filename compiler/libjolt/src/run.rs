//! Run pipeline: parse through interpret.

use jolt_diagnostics::{render_at, render_diagnostic, Severity};
use jolt_source::FileId;

use crate::{CustodyResult, MirResult, ParseResult, ResolveResult, RunResult, Session};
use jolt_types::CheckResult;

/// Combined compile + run result for one source file.
#[derive(Debug, Clone)]
pub struct RunReport {
    pub parse: ParseResult,
    pub resolve: ResolveResult,
    pub check: CheckResult,
    pub custody: CustodyResult,
    pub mir: MirResult,
    pub run: RunResult,
}

impl RunReport {
    pub fn is_ok(&self) -> bool {
        self.run.is_ok()
    }

    pub fn stdout(&self) -> Option<&str> {
        self.run.stdout()
    }

    /// Render compile-phase diagnostics (stops before runtime errors).
    pub fn render_compile_diagnostics(&self, source: &str) -> Vec<String> {
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
        for diag in &self.mir.diagnostics.items {
            lines.push(render_diagnostic(diag, source));
        }
        lines
    }

    pub fn render_all_errors(&self, source: &str) -> Vec<String> {
        let mut lines = self.render_compile_diagnostics(source);
        if let Err(e) = &self.run.interpret {
            lines.push(format!("runtime error: {e}"));
        }
        lines
    }
}

impl Session {
    /// Run parse → … → MIR → interpret on `source` (memoized per stage).
    pub fn run_source(&mut self, file: FileId, source: &str) -> RunReport {
        let parse = self.parse_file(file, source);
        let resolve = self.resolve_file(file, source);
        let check = self.check_file(file, source);
        let custody = self.custody_file(file, source);
        let mir = self.mir_file(file, source);
        let run = self.run_file(file, source);
        RunReport {
            parse,
            resolve,
            check,
            custody,
            mir,
            run,
        }
    }
}
