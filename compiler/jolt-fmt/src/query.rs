use jolt_parser::{parse_file, ParseResult};
use jolt_query::{hash_bytes, QueryEngine, QueryName};
use jolt_source::FileId;

use crate::printer::print_program;
use crate::FormatResult;

/// Query name for per-file formatting (keyed by file content hash; depends on [`PARSE_FILE`]).
pub const FMT_FILE: QueryName = QueryName("fmt_file");

/// Format `source` through the query engine (memoized; parses via nested `parse_file`).
pub fn format_file(engine: &mut QueryEngine, file: FileId, source: &str) -> FormatResult {
    let input = hash_bytes(source.as_bytes());
    engine.query(FMT_FILE, input, &[input], |eng| {
        let parse = parse_file(eng, file, source);
        format_parsed(&parse, source)
    })
}

/// Format a parse result.
///
/// On parse errors, returns the original `source` unchanged and copies parse errors into the result.
pub fn format_parsed(parse: &ParseResult, source: &str) -> FormatResult {
    if !parse.is_ok() {
        return FormatResult {
            source: source.to_string(),
            errors: parse.errors.clone(),
        };
    }
    FormatResult {
        source: print_program(&parse.program),
        errors: Vec::new(),
    }
}

/// Format a program AST directly (no parse errors).
pub fn format_program(program: &jolt_ast::Program) -> String {
    print_program(program)
}
