use std::fmt;

/// Tiny subset type representation (`docs/spec/jolt-spec-v0.4.md` §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ty {
    Int,
    Bool,
    None,
    String,
    /// Shared borrow of `String` (Tiny Phase 2b).
    BorrowString,
    /// Exclusive borrow of `String` (Tiny Phase 2b).
    ClaimString,
    /// Poison type for cascading after an error.
    Error,
}

impl Ty {
    pub fn is_error(self) -> bool {
        self == Ty::Error
    }

    /// `Int`, `Bool`, `None`, and borrow handles copy; owned `String` moves.
    pub fn is_copy(self) -> bool {
        matches!(
            self,
            Ty::Int | Ty::Bool | Ty::None | Ty::BorrowString | Ty::ClaimString
        )
    }

    pub fn is_borrow_handle(self) -> bool {
        matches!(self, Ty::BorrowString | Ty::ClaimString)
    }

    pub fn pointee(self) -> Option<Self> {
        match self {
            Ty::BorrowString | Ty::ClaimString => Some(Ty::String),
            _ => None,
        }
    }

    pub fn unify(a: Self, b: Self) -> Self {
        if a.is_error() || b.is_error() {
            return Ty::Error;
        }
        if a == b {
            a
        } else {
            Ty::Error
        }
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int => write!(f, "Int"),
            Ty::Bool => write!(f, "Bool"),
            Ty::None => write!(f, "None"),
            Ty::String => write!(f, "String"),
            Ty::BorrowString => write!(f, "Borrow<String>"),
            Ty::ClaimString => write!(f, "Claim<String>"),
            Ty::Error => write!(f, "<error>"),
        }
    }
}
