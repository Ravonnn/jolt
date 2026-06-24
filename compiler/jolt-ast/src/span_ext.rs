use jolt_source::Span;

/// Merge two spans (same file assumed).
pub fn span_between(a: Span, b: Span) -> Span {
    Span {
        file: a.file,
        start: a.start.min(b.start),
        end: a.end.max(b.end),
    }
}

pub fn span_from(start: Span, end: Span) -> Span {
    span_between(start, end)
}
