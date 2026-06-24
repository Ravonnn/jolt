//! Lexer for the Jolt **Tiny** subset.
//!
//! Hand-rolled scanner with explicit **max-munch** rules per
//! [`jolt-grammar.md`](../../docs/spec/jolt-grammar.md) §1 (see also §13–§14).
//!
//! - `@fn` lexes as [`TokenKind::At`] + [`TokenKind::Fn`], not a fused token.
//! - `//` line comments (including `///`) are skipped, not emitted.
//! - String literals are supported; `{interpolation}` is not (Phase 1).

mod error;
mod lexer;
mod query;
mod token;

pub use error::LexErrorKind;
pub use lexer::{lex, Lexer};
pub use query::{lex_file, LEX_FILE};
pub use token::{ident_or_keyword, Token, TokenKind};

pub const STAGE: &str = "lexer";

#[cfg(test)]
mod tests;
