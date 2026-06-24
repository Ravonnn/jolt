use jolt_query::{hash_bytes, QueryEngine, QueryName};
use jolt_source::FileId;
use jolt_types::{check_file, CheckResult};

use crate::check::check_program;
use crate::CustodyResult;

/// Query name for per-file custody checking (depends on [`CHECK_FILE`]).
pub const CUSTODY_FILE: QueryName = QueryName("custody_file");

pub fn custody_file(engine: &mut QueryEngine, file: FileId, source: &str) -> CustodyResult {
    let input = hash_bytes(source.as_bytes());
    engine.query(CUSTODY_FILE, input, &[input], |eng| {
        let checked = check_file(eng, file, source);
        custody_checked(&checked)
    })
}

pub fn custody_checked(checked: &CheckResult) -> CustodyResult {
    if !checked.is_ok() {
        return CustodyResult {
            program: checked.program.clone(),
            diagnostics: Default::default(),
        };
    }
    let diagnostics = check_program(&checked.program);
    CustodyResult {
        program: checked.program.clone(),
        diagnostics,
    }
}
