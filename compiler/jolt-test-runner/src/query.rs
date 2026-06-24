use jolt_mir::{mir_file, MirResult};
use jolt_query::{hash_bytes, QueryEngine, QueryName};
use jolt_source::FileId;

use crate::runner::{discover_tests, run_tests, TestReport};

/// Query name for per-file test execution (depends on [`MIR_FILE`]).
pub const TEST_FILE: QueryName = QueryName("test_file");

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct TestFileResult {
    pub mir: MirResult,
    pub report: TestReport,
}

impl TestFileResult {
    pub fn is_ok(&self) -> bool {
        self.mir.is_ok() && self.report.is_ok()
    }
}

pub fn test_file(engine: &mut QueryEngine, file: FileId, source: &str) -> TestFileResult {
    let input = hash_bytes(source.as_bytes());
    engine.query(TEST_FILE, input, &[input], |eng| {
        let mir = mir_file(eng, file, source);
        test_mir(&mir)
    })
}

pub fn test_mir(mir: &MirResult) -> TestFileResult {
    if !mir.is_ok() {
        return TestFileResult {
            mir: mir.clone(),
            report: TestReport { cases: Vec::new() },
        };
    }
    let tests = discover_tests(&mir.program);
    let report = run_tests(&mir.module, &tests);
    TestFileResult {
        mir: mir.clone(),
        report,
    }
}
