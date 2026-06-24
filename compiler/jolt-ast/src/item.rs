use jolt_source::Span;

use crate::expr::Expr;
use crate::stmt::Stmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    Fn(FnDecl),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: FnBody,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeExpr {
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FnBody {
    Expr(Expr),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail: Option<Expr>,
    pub span: Span,
}
