use jolt_query::{hash_bytes, QueryEngine, QueryName};
use jolt_source::FileId;

use crate::lexer::lex;
use crate::token::Token;

/// Query name for per-file lexing (keyed by file content hash).
pub const LEX_FILE: QueryName = QueryName("lex_file");

/// Lex `source` through the query engine (memoized by content hash).
pub fn lex_file(engine: &mut QueryEngine, file: FileId, source: &str) -> Vec<Token> {
    let input = hash_bytes(source.as_bytes());
    engine.query(LEX_FILE, input, &[input], |_| lex(source, file))
}
