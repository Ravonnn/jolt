/// Classification of parse diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseErrorKind {
    UnexpectedToken,
    UnexpectedEof,
    LexError,
    MissingSemiSemi,
    InvalidSyntax,
}

/// A non-fatal parse error (recovery may continue).
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ParseError {
    pub message: String,
    pub span: jolt_source::Span,
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, span: jolt_source::Span, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span,
            kind,
        }
    }
}
