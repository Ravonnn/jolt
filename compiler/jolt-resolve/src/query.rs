use jolt_parser::{parse_file, ParseResult};
use jolt_query::{hash_bytes, QueryEngine, QueryName};
use jolt_source::FileId;

use crate::resolve::resolve_program;
use crate::ResolveResult;

/// Query name for per-file name resolution (keyed by file content hash; depends on [`PARSE_FILE`]).
pub const RESOLVE_FILE: QueryName = QueryName("resolve_file");

/// Resolve names in `source` through the query engine (memoized; parses via nested `parse_file`).
///
/// Per-function [`RESOLVE_FN`] incrementality is deferred; file-level memoization is supported.
pub fn resolve_file(engine: &mut QueryEngine, file: FileId, source: &str) -> ResolveResult {
    let input = hash_bytes(source.as_bytes());
    engine.query(RESOLVE_FILE, input, &[input], |eng| {
        let parse = parse_file(eng, file, source);
        resolve_parsed(&parse)
    })
}

/// Resolve a parse result.
///
/// Always walks `parse.program` even when `parse.errors` is non-empty so partial ASTs from
/// error recovery still get name checks. Parse diagnostics are not copied into [`ResolveResult`];
/// callers should inspect [`ParseResult::is_ok`] separately.
pub fn resolve_parsed(parse: &ParseResult) -> ResolveResult {
    resolve_program(&parse.program)
}
