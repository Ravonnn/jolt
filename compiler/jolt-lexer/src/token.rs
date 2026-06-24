use std::fmt;

use jolt_source::Span;

use crate::error::LexErrorKind;

/// Token classification for the Tiny lexer (and shared punctuators for later phases).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Keywords
    If,
    Else,
    Loop,
    For,
    In,
    Return,
    Break,
    Next,
    True,
    False,
    /// Function introducer after `@` (e.g. `@fn`, `@main` uses `Ident`).
    Fn,

    // Literals
    IntLit(i64),
    StringLit(String),

    // Identifiers (Int, Bool, println, etc. are Idents until semantic analysis)
    Ident(String),

    // Sigils
    At,
    Dollar,
    DollarDollar,

    // Comparison
    EqEq,
    NotEq,
    Lt,
    Gt,
    Le,
    Ge,

    // Logic
    AndAnd,
    OrOr,
    Not,

    // Arithmetic
    Plus,
    Minus,
    Star,
    Slash,
    SlashSlash,
    Percent,
    Caret,

    // Bitwise
    Amp,
    Pipe,
    Tilde,
    PercentPipe,
    TildeAmp,
    TildePipe,
    TildePercentPipe,

    // Shift
    LtLt,
    GtGt,
    LtLtLt,
    GtGtGt,
    LtLtPipe,
    GtGtPipe,

    // Assignment (lex only)
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    SlashSlashEq,
    PercentEq,
    CaretEq,
    AmpEq,
    PipeEq,
    PercentPipeEq,
    TildeAmpEq,
    TildePipeEq,
    TildePercentPipeEq,
    GtGtEq,
    LtLtEq,
    GtGtPipeEq,
    LtLtPipeEq,
    LtLtLtEq,
    GtGtGtEq,

    // Blocks
    Arrow,
    Semi,
    SemiSemi,

    // Delimiters
    LParen,
    RParen,
    Comma,
    Colon,
    LBracket,
    RBracket,

    // Range / misc
    DotDot,
    DotDotDot,

    Eof,
    Error(LexErrorKind),
}

/// A single lexeme with source span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Next => write!(f, "next"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::IntLit(n) => write!(f, "{n}"),
            TokenKind::StringLit(s) => write!(f, "\"{s}\""),
            TokenKind::Ident(s) => write!(f, "{s}"),
            TokenKind::At => write!(f, "@"),
            TokenKind::Dollar => write!(f, "$"),
            TokenKind::DollarDollar => write!(f, "$$"),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::NotEq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::Le => write!(f, "<="),
            TokenKind::Ge => write!(f, ">="),
            TokenKind::AndAnd => write!(f, "&&"),
            TokenKind::OrOr => write!(f, "||"),
            TokenKind::Not => write!(f, "!"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::SlashSlash => write!(f, "//"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Amp => write!(f, "&"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::PercentPipe => write!(f, "%|"),
            TokenKind::TildeAmp => write!(f, "~&"),
            TokenKind::TildePipe => write!(f, "~|"),
            TokenKind::TildePercentPipe => write!(f, "~%|"),
            TokenKind::LtLt => write!(f, "<<"),
            TokenKind::GtGt => write!(f, ">>"),
            TokenKind::LtLtLt => write!(f, "<<<"),
            TokenKind::GtGtGt => write!(f, ">>>"),
            TokenKind::LtLtPipe => write!(f, "<<|"),
            TokenKind::GtGtPipe => write!(f, ">>|"),
            TokenKind::Eq => write!(f, "="),
            TokenKind::PlusEq => write!(f, "+="),
            TokenKind::MinusEq => write!(f, "-="),
            TokenKind::StarEq => write!(f, "*="),
            TokenKind::SlashEq => write!(f, "/="),
            TokenKind::SlashSlashEq => write!(f, "//="),
            TokenKind::PercentEq => write!(f, "%="),
            TokenKind::CaretEq => write!(f, "^="),
            TokenKind::AmpEq => write!(f, "&="),
            TokenKind::PipeEq => write!(f, "|="),
            TokenKind::PercentPipeEq => write!(f, "%|="),
            TokenKind::TildeAmpEq => write!(f, "~&="),
            TokenKind::TildePipeEq => write!(f, "~|="),
            TokenKind::TildePercentPipeEq => write!(f, "~%|="),
            TokenKind::GtGtEq => write!(f, ">>="),
            TokenKind::LtLtEq => write!(f, "<<="),
            TokenKind::GtGtPipeEq => write!(f, ">>|="),
            TokenKind::LtLtPipeEq => write!(f, "<<|="),
            TokenKind::LtLtLtEq => write!(f, "<<<="),
            TokenKind::GtGtGtEq => write!(f, ">>>="),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Semi => write!(f, ";"),
            TokenKind::SemiSemi => write!(f, ";;"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::DotDotDot => write!(f, "..."),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::Error(e) => write!(f, "ERROR({e:?})"),
        }
    }
}

/// Map ASCII identifier text to a keyword, or `Ident`.
pub fn ident_or_keyword(text: &str) -> TokenKind {
    match text {
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "loop" => TokenKind::Loop,
        "for" => TokenKind::For,
        "in" => TokenKind::In,
        "return" => TokenKind::Return,
        "break" => TokenKind::Break,
        "next" => TokenKind::Next,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "fn" => TokenKind::Fn,
        other => TokenKind::Ident(other.to_string()),
    }
}
