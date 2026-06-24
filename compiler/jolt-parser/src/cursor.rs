use jolt_ast::span_between;
use jolt_lexer::{Token, TokenKind};
use jolt_source::Span;

use crate::error::{ParseError, ParseErrorKind};

pub struct ParserCursor<'a> {
    tokens: &'a [Token],
    pos: usize,
    errors: Vec<ParseError>,
}

impl<'a> ParserCursor<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    pub fn into_errors(self) -> Vec<ParseError> {
        self.errors
    }

    pub fn push_error(&mut self, err: ParseError) {
        self.errors.push(err);
    }

    pub fn is_at_end(&self) -> bool {
        self.peek_kind() == Some(TokenKind::Eof)
    }

    pub fn peek(&self) -> Option<&Token> {
        let t = self.tokens.get(self.pos)?;
        if matches!(t.kind, TokenKind::Eof) {
            None
        } else {
            Some(t)
        }
    }

    pub fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|t| t.kind.clone())
    }

    pub fn peek_kind_at(&self, offset: usize) -> Option<TokenKind> {
        let idx = self.pos + offset;
        let t = self.tokens.get(idx)?;
        if matches!(t.kind, TokenKind::Eof) {
            None
        } else {
            Some(t.kind.clone())
        }
    }

    pub fn bump(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos)?.clone();
        if !matches!(t.kind, TokenKind::Eof) {
            self.pos += 1;
        }
        if let TokenKind::Error(e) = t.kind {
            self.errors.push(ParseError::new(
                ParseErrorKind::LexError(e),
                format!("lexical error: {e:?}"),
                t.span,
            ));
        }
        Some(t)
    }

    pub fn current_span(&self) -> Span {
        self.peek()
            .map(|t| t.span)
            .or_else(|| self.tokens.last().map(|t| t.span))
            .unwrap_or(Span {
                file: jolt_source::FileId(0),
                start: 0,
                end: 0,
            })
    }

    pub fn expect_ident(&mut self, msg: &str) -> Option<(String, Span)> {
        let sp = self.current_span();
        match self.peek_kind() {
            Some(TokenKind::Ident(name)) => {
                let t = self.bump()?;
                Some((name, t.span))
            }
            _ => {
                self.errors.push(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    msg,
                    sp,
                ));
                None
            }
        }
    }

    pub fn expect(&mut self, kind: TokenKind, msg: &str) -> Option<Token> {
        let sp = self.current_span();
        match self.peek_kind() {
            Some(k) if tokens_eq(&k, &kind) => self.bump(),
            Some(_) => {
                self.errors.push(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    msg,
                    sp,
                ));
                None
            }
            None => {
                self.errors.push(ParseError::new(
                    ParseErrorKind::UnexpectedEof,
                    msg,
                    sp,
                ));
                None
            }
        }
    }

    /// Skip tokens until `;`, `;;`, `->`, or `Eof` (LSP-oriented recovery).
    pub fn synchronize(&mut self) {
        while let Some(kind) = self.peek_kind() {
            match kind {
                TokenKind::Semi | TokenKind::SemiSemi | TokenKind::Arrow | TokenKind::Eof => {
                    return;
                }
                TokenKind::Error(_) => {
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    pub fn span_from(&self, start: Span) -> Span {
        let end = self
            .tokens
            .get(self.pos.wrapping_sub(1))
            .map(|t| t.span)
            .unwrap_or(start);
        span_between(start, end)
    }
}

fn tokens_eq(a: &TokenKind, b: &TokenKind) -> bool {
    match (a, b) {
        (TokenKind::Ident(x), TokenKind::Ident(y)) => x == y,
        (TokenKind::IntLit(x), TokenKind::IntLit(y)) => x == y,
        (TokenKind::StringLit(x), TokenKind::StringLit(y)) => x == y,
        (TokenKind::Error(x), TokenKind::Error(y)) => x == y,
        _ => {
            std::mem::discriminant(a) == std::mem::discriminant(b)
                && !matches!(
                    a,
                    TokenKind::Ident(_)
                        | TokenKind::IntLit(_)
                        | TokenKind::StringLit(_)
                        | TokenKind::Error(_)
                )
        }
    }
}
