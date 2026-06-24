use jolt_query::{hash_bytes, QueryEngine, QueryName};
use jolt_source::FileId;

use crate::parser::parse_tokens;
use crate::ParseResult;
use jolt_lexer::lex_file;

/// Query name for per-file parsing (keyed by file content hash; depends on [`LEX_FILE`]).
pub const PARSE_FILE: QueryName = QueryName("parse_file");

/// Parse `source` through the query engine (memoized; lexes via nested `lex_file`).
pub fn parse_file(engine: &mut QueryEngine, file: FileId, source: &str) -> ParseResult {
    let input = hash_bytes(source.as_bytes());
    engine.query(PARSE_FILE, input, &[input], |eng| {
        let tokens = lex_file(eng, file, source);
        parse_tokens(&tokens)
    })
}

/// Parse `source` without the query cache (lexes directly).
pub fn parse_source(engine: &mut QueryEngine, file: FileId, source: &str) -> ParseResult {
    parse_file(engine, file, source)
}
