/// Lexical error kinds (carried on [`crate::TokenKind::Error`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LexErrorKind {
    /// `"` opened but not closed before EOF/newline.
    UnterminatedString,
    /// Unknown escape after `\` in a string.
    InvalidEscape,
    /// Invalid digit for the numeric base.
    InvalidDigit,
    /// Integer literal does not fit in `i64`.
    IntOverflow,
    /// Unrecognized byte/character.
    UnexpectedChar,
}
