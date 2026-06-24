use jolt_diagnostics::Diagnostic;
use jolt_source::Span;

/// Classification of custody violations (stable for unit tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustodyErrorKind {
    UseAfterMove,
    UseWhileBorrowed,
    ClaimWhileBorrowed,
    SharedWhileClaimed,
}

impl CustodyErrorKind {
    pub fn code(self) -> &'static str {
        match self {
            CustodyErrorKind::UseAfterMove => "E0401",
            CustodyErrorKind::UseWhileBorrowed => "E0402",
            CustodyErrorKind::ClaimWhileBorrowed => "E0403",
            CustodyErrorKind::SharedWhileClaimed => "E0404",
        }
    }
}

pub fn use_after_move(span: Span, name: &str) -> Diagnostic {
    Diagnostic::error(
        span,
        format!("custody violation: use of moved value `{name}`"),
    )
    .with_code(CustodyErrorKind::UseAfterMove.code())
    .with_hint(format!(
        "borrow `{name}` before the last use, or clone if the type supports copy"
    ))
}

pub fn use_while_borrowed(span: Span, name: &str) -> Diagnostic {
    Diagnostic::error(
        span,
        format!("custody violation: cannot use `{name}` while borrowed"),
    )
    .with_code(CustodyErrorKind::UseWhileBorrowed.code())
    .with_hint("wait until borrow handles on this value expire (last use), then use it again")
}

pub fn claim_while_borrowed(span: Span, name: &str) -> Diagnostic {
    Diagnostic::error(
        span,
        format!("custody violation: cannot claim `{name}` while borrowed"),
    )
    .with_code(CustodyErrorKind::ClaimWhileBorrowed.code())
    .with_hint("release shared borrows first (let borrow handles go out of use)")
}

pub fn shared_while_claimed(span: Span, name: &str) -> Diagnostic {
    Diagnostic::error(
        span,
        format!("custody violation: cannot borrow `{name}` while claimed"),
    )
    .with_code(CustodyErrorKind::SharedWhileClaimed.code())
    .with_hint("drop the active claim handle before taking a shared borrow")
}
