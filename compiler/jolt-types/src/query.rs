use jolt_query::{hash_bytes, QueryEngine, QueryName};
use jolt_resolve::{resolve_file, ResolveResult};
use jolt_source::FileId;

use crate::check::check_program;
use crate::CheckResult;

/// Query name for per-file type checking (keyed by file content hash; depends on [`RESOLVE_FILE`]).
pub const CHECK_FILE: QueryName = QueryName("check_file");

/// Type-check `source` through the query engine (memoized; resolves via nested `resolve_file`).
///
/// Per-function [`CHECK_FN`] incrementality is deferred; file-level memoization is supported.
pub fn check_file(engine: &mut QueryEngine, file: FileId, source: &str) -> CheckResult {
    let input = hash_bytes(source.as_bytes());
    engine.query(CHECK_FILE, input, &[input], |eng| {
        let resolved = resolve_file(eng, file, source);
        check_resolved(&resolved)
    })
}

/// Type-check a resolved program.
///
/// If `resolved.errors` is non-empty, the type pass is **skipped** and an empty
/// [`CheckResult::diagnostics`] is returned. Callers (e.g. `jolt check`) should report resolve
/// errors separately. Parse errors are not visible here; resolve already ran on the parse output.
pub fn check_resolved(resolved: &ResolveResult) -> CheckResult {
    if !resolved.is_ok() {
        return CheckResult {
            program: resolved.program.clone(),
            diagnostics: Default::default(),
        };
    }
    let diagnostics = check_program(&resolved.program);
    CheckResult {
        program: resolved.program.clone(),
        diagnostics,
    }
}
