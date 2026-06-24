//! Test pipeline: parse through `[test]` execution.

use jolt_diagnostics::{render_at, render_diagnostic, Severity};
use jolt_source::FileId;

use crate::{CustodyResult, MirResult, ParseResult, ResolveResult, Session};
use jolt_test_runner::{TestFileResult, TestReport};
use jolt_types::CheckResult;

/// Combined compile + test result for one source file.
#[derive(Debug, Clone)]
pub struct TestSourceReport {
    pub parse: ParseResult,
    pub resolve: ResolveResult,
    pub check: CheckResult,
    pub custody: CustodyResult,
    pub mir: MirResult,
    pub test: TestFileResult,
}

impl TestSourceReport {
    pub fn is_ok(&self) -> bool {
        self.test.is_ok()
    }

    pub fn report(&self) -> &TestReport {
        &self.test.report
    }

    pub fn render_all_errors(&self, source: &str) -> Vec<String> {
        let mut out = Vec::new();
        for err in &self.parse.errors {
            out.push(render_at(
                Severity::Error,
                err.span.start,
                source,
                &err.message,
                None,
            ));
        }
        for err in &self.resolve.errors {
            out.push(render_at(
                Severity::Error,
                err.span.start,
                source,
                &err.message,
                None,
            ));
        }
        for diag in &self.check.diagnostics.items {
            out.push(render_diagnostic(diag, source));
        }
        for diag in &self.custody.diagnostics.items {
            out.push(render_diagnostic(diag, source));
        }
        for diag in &self.mir.diagnostics.items {
            out.push(render_diagnostic(diag, source));
        }
        out
    }
}

pub fn test_source(session: &mut Session, file: FileId, source: &str) -> TestSourceReport {
    let parse = session.parse_file(file, source);
    let resolve = session.resolve_file(file, source);
    let check = session.check_file(file, source);
    let custody = session.custody_file(file, source);
    let mir = session.mir_file(file, source);
    let test = jolt_test_runner::test_file(session.engine_mut(), file, source);
    TestSourceReport {
        parse,
        resolve,
        check,
        custody,
        mir,
        test,
    }
}
