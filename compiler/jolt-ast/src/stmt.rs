use jolt_source::Span;

use crate::expr::Expr;
use crate::item::{Block, TypeExpr};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Binding {
    pub mutable: bool,
    pub name: String,
    pub ty: Option<TypeExpr>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Assign {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ForPattern {
    Ident(String),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForStmt {
    pub pattern: ForPattern,
    pub iter: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    Binding(Binding),
    Assign(Assign),
    Expr(Expr),
    If(IfStmt),
    Loop(Block),
    For(ForStmt),
    Return {
        value: Option<Expr>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Next {
        span: Span,
    },
}
