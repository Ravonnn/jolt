//! Structured compiler diagnostics with stable rendering for snapshots and CLI output.

use jolt_source::Span;

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        }
    }
}

/// A single diagnostic message anchored to source.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub code: Option<String>,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn error(span: Span, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            span,
            code: None,
            hint: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// Collected diagnostics from a compiler pass.
#[derive(Debug, Clone, PartialEq, Hash, Default)]
pub struct Diagnostics {
    pub items: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn push(&mut self, diag: Diagnostic) {
        self.items.push(diag);
    }

    pub fn extend(&mut self, other: Diagnostics) {
        self.items.extend(other.items);
    }
}

/// Compute 1-based `(line, column)` for a byte offset in `source`.
pub fn line_col(source: &str, byte_offset: u32) -> (usize, usize) {
    let offset = byte_offset as usize;
    let safe = offset.min(source.len());
    let before = &source[..safe];
    let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
    let col = before.rfind('\n').map(|i| safe - i).unwrap_or(safe + 1);
    (line, col)
}

/// Stable one-line format for snapshot tests and CLI stderr.
///
/// Example: `3:5: error: expected Int, found Bool`
pub fn render_diagnostic(diag: &Diagnostic, source: &str) -> String {
    render_at_with_hint(
        diag.severity,
        diag.span.start,
        source,
        &diag.message,
        diag.code.as_deref(),
        diag.hint.as_deref(),
    )
}

/// Render a message at `span.start` (used for parse/resolve errors in the CLI).
pub fn render_at(
    severity: Severity,
    start: u32,
    source: &str,
    message: &str,
    code: Option<&str>,
) -> String {
    let (line, col) = line_col(source, start);
    let mut out = format!("{line}:{col}: {}: {message}", severity.as_str());
    if let Some(code) = code {
        out.push_str(&format!(" [{code}]"));
    }
    out
}

/// Like [`render_at`] but includes an optional hint clause.
pub fn render_at_with_hint(
    severity: Severity,
    start: u32,
    source: &str,
    message: &str,
    code: Option<&str>,
    hint: Option<&str>,
) -> String {
    let mut out = render_at(severity, start, source, message, code);
    if let Some(hint) = hint {
        out.push_str(&format!("; hint: {hint}"));
    }
    out
}

pub const STAGE: &str = "diagnostics";

#[cfg(test)]
mod tests {
    use super::*;
    use jolt_source::FileId;

    #[test]
    fn render_includes_line_col() {
        let source = "line1\nbad here";
        let span = Span {
            file: FileId(1),
            start: 6,
            end: 9,
        };
        let diag = Diagnostic::error(span, "expected Int, found Bool");
        assert_eq!(
            render_diagnostic(&diag, source),
            "2:1: error: expected Int, found Bool"
        );
    }
}
