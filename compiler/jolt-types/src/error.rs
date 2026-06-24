use jolt_diagnostics::Diagnostic;
use jolt_source::Span;

use crate::ty::Ty;

/// Classification of type errors (stable for unit tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeErrorKind {
    Mismatch,
    UnknownType,
    WrongArgCount,
    UnknownFunction,
    BinOpMismatch,
    BranchMismatch,
    CannotBorrowCopy,
    ClaimRequiresMutable,
    DerefRequiresBorrow,
}

impl TypeErrorKind {
    pub fn code(self) -> &'static str {
        match self {
            TypeErrorKind::Mismatch => "E0301",
            TypeErrorKind::UnknownType => "E0302",
            TypeErrorKind::WrongArgCount => "E0303",
            TypeErrorKind::UnknownFunction => "E0304",
            TypeErrorKind::BinOpMismatch => "E0305",
            TypeErrorKind::BranchMismatch => "E0306",
            TypeErrorKind::CannotBorrowCopy => "E0307",
            TypeErrorKind::ClaimRequiresMutable => "E0308",
            TypeErrorKind::DerefRequiresBorrow => "E0309",
        }
    }
}

pub fn mismatch(span: Span, expected: Ty, found: Ty) -> Diagnostic {
    Diagnostic::error(span, format!("expected {}, found {}", expected, found))
        .with_code(TypeErrorKind::Mismatch.code())
}

pub fn binop_mismatch(span: Span, op: &str, left: Ty, right: Ty) -> Diagnostic {
    Diagnostic::error(span, format!("cannot {op} {left} and {right}"))
        .with_code(TypeErrorKind::BinOpMismatch.code())
}

pub fn branch_mismatch(span: Span, then_ty: Ty, else_ty: Ty) -> Diagnostic {
    Diagnostic::error(
        span,
        format!(
            "if branches must have the same type (then {}, else {})",
            then_ty, else_ty
        ),
    )
    .with_code(TypeErrorKind::BranchMismatch.code())
}

pub fn unknown_type(span: Span, name: &str) -> Diagnostic {
    Diagnostic::error(span, format!("unknown type '{name}'"))
        .with_code(TypeErrorKind::UnknownType.code())
}

pub fn wrong_arg_count(span: Span, name: &str, expected: usize, found: usize) -> Diagnostic {
    Diagnostic::error(
        span,
        format!("function '{name}' expects {expected} arguments, found {found}"),
    )
    .with_code(TypeErrorKind::WrongArgCount.code())
}

pub fn unknown_function(span: Span, name: &str) -> Diagnostic {
    Diagnostic::error(span, format!("unknown function '{name}'"))
        .with_code(TypeErrorKind::UnknownFunction.code())
}

pub fn return_type_mismatch(span: Span, expected: Ty, found: Ty) -> Diagnostic {
    Diagnostic::error(
        span,
        format!("function returns {expected} but body has type {found}"),
    )
    .with_code(TypeErrorKind::Mismatch.code())
}

pub fn println_arg_mismatch(span: Span, found: Ty) -> Diagnostic {
    Diagnostic::error(span, format!("println expects String, found {found}"))
        .with_code(TypeErrorKind::Mismatch.code())
}

pub fn condition_not_bool(span: Span, found: Ty) -> Diagnostic {
    Diagnostic::error(span, format!("expected Bool condition, found {found}"))
        .with_code(TypeErrorKind::Mismatch.code())
}

pub fn for_iter_not_int(span: Span, found: Ty) -> Diagnostic {
    Diagnostic::error(
        span,
        format!("for-in over Int is a Tiny stub; found {found} (expected Int)"),
    )
    .with_code(TypeErrorKind::Mismatch.code())
}

pub fn binding_annotation_mismatch(span: Span, expected: Ty, found: Ty) -> Diagnostic {
    Diagnostic::error(
        span,
        format!("binding type annotation {expected} does not match inferred {found}"),
    )
    .with_code(TypeErrorKind::Mismatch.code())
}

pub fn assign_mismatch(span: Span, expected: Ty, found: Ty) -> Diagnostic {
    Diagnostic::error(
        span,
        format!("cannot assign {found} to binding of type {expected}"),
    )
    .with_code(TypeErrorKind::Mismatch.code())
}

pub fn cannot_borrow_copy(span: Span, found: Ty) -> Diagnostic {
    Diagnostic::error(span, format!("cannot borrow copy type {found}"))
        .with_code(TypeErrorKind::CannotBorrowCopy.code())
}

pub fn claim_requires_mutable(span: Span, name: &str) -> Diagnostic {
    Diagnostic::error(span, format!("claim requires mutable binding `$$`{name}"))
        .with_code(TypeErrorKind::ClaimRequiresMutable.code())
}

pub fn deref_requires_borrow(span: Span, found: Ty) -> Diagnostic {
    Diagnostic::error(
        span,
        format!("deref requires Borrow or Claim, found {found}"),
    )
    .with_code(TypeErrorKind::DerefRequiresBorrow.code())
}
