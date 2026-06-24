use jolt_source::Span;

/// Classification of name-resolution diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolveErrorKind {
    UndefinedName,
    DuplicateBinding,
    ImmutableAssign,
    InvalidReassign,
}

/// A name-resolution error with a stable kind for tests.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ResolveError {
    pub message: String,
    pub span: Span,
    pub kind: ResolveErrorKind,
}

impl ResolveError {
    pub fn new(kind: ResolveErrorKind, span: Span, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span,
            kind,
        }
    }
}
