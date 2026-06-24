//! AST for the Jolt **Tiny** subset (`docs/spec/jolt-grammar.md` §2–§7, §9).

mod debug;

use jolt_source::Span;

pub use debug::debug_print;

/// A parsed source file.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Program {
    pub span: Span,
    pub items: Vec<Item>,
}

/// Top-level item.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Item {
    Fn(FnDecl),
}

/// `@name(params) ReturnType -> body`
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FnDecl {
    pub span: Span,
    pub attrs: Vec<Ident>,
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: FnBody,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Param {
    pub span: Span,
    pub name: Ident,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum FnBody {
    Expr(Expr),
    Block(Block),
}

/// `-> { stmts [tail] } ;;` — trailing expression has no semicolon.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Block {
    pub span: Span,
    pub stmts: Vec<Stmt>,
    pub tail: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Stmt {
    Binding(Binding),
    Assign(Assign),
    Expr(Expr),
    If(IfStmt),
    Loop(LoopStmt),
    For(ForStmt),
    Return(ReturnStmt),
    Break(BreakStmt),
    Next(NextStmt),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Binding {
    pub span: Span,
    pub mutable: bool,
    pub name: Ident,
    pub ty: Option<TypeExpr>,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Assign {
    pub span: Span,
    pub name: Ident,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct IfStmt {
    pub span: Span,
    pub cond: Expr,
    pub then_block: Block,
    pub else_block: Option<Block>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct LoopStmt {
    pub span: Span,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ForStmt {
    pub span: Span,
    pub pattern: ForPattern,
    pub iter: Expr,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ForPattern {
    Ident(Ident),
    Wildcard(Span),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ReturnStmt {
    pub span: Span,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct BreakStmt {
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct NextStmt {
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Expr {
    IntLit(Spanned<i64>),
    BoolLit(Spanned<bool>),
    StringLit(Spanned<String>),
    Ident(Spanned<String>),
    Unary(Spanned<UnaryOp>, Box<Expr>),
    Binary(Spanned<BinaryOp>, Box<Expr>, Box<Expr>),
    Call(Spanned<String>, Vec<Expr>),
    Block(Box<Block>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Or,
    And,
    Eq,
    NotEq,
    Lt,
    Gt,
    Le,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Mod,
    Pow,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum TypeExpr {
    Named(Spanned<String>),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Ident {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

impl Ident {
    pub fn new(span: Span, name: impl Into<String>) -> Self {
        Self {
            span,
            name: name.into(),
        }
    }
}

impl<T> Spanned<T> {
    pub fn new(span: Span, value: T) -> Self {
        Self { span, value }
    }
}

pub const STAGE: &str = "ast";
