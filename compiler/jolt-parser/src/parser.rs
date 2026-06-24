use jolt_ast::*;
use jolt_lexer::{Token, TokenKind};
use jolt_source::Span;

use crate::error::{ParseError, ParseErrorKind};
use crate::ParseResult;

pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens
            .get(self.pos)
            .filter(|t| !matches!(t.kind, TokenKind::Eof))
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }

    fn bump(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos)?.clone();
        if !matches!(t.kind, TokenKind::Eof) {
            self.pos += 1;
        }
        Some(t)
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek_kind().is_some_and(|k| k == kind)
    }

    fn merge_span(&self, a: Span, b: Span) -> Span {
        Span {
            file: a.file,
            start: a.start.min(b.start),
            end: a.end.max(b.end),
        }
    }

    fn error(&mut self, kind: ParseErrorKind, span: Span, msg: impl Into<String>) {
        self.errors.push(ParseError::new(kind, span, msg));
    }

    fn expect_kind(&mut self, kind: TokenKind, ctx: &str) -> Option<Token> {
        match self.peek() {
            Some(t) if t.kind == kind => {
                let t = t.clone();
                self.pos += 1;
                Some(t)
            }
            Some(t) => {
                self.error(
                    ParseErrorKind::UnexpectedToken,
                    t.span,
                    format!("expected {ctx}, found {}", t.kind),
                );
                None
            }
            None => {
                let eof = self.tokens.last().map(|t| t.span).unwrap_or(Span {
                    file: jolt_source::FileId(0),
                    start: 0,
                    end: 0,
                });
                self.error(
                    ParseErrorKind::UnexpectedEof,
                    eof,
                    format!("expected {ctx}"),
                );
                None
            }
        }
    }

    fn skip_lex_errors(&mut self) {
        while let Some(t) = self.peek() {
            if let TokenKind::Error(e) = t.kind {
                let span = t.span;
                self.bump();
                self.error(
                    ParseErrorKind::LexError,
                    span,
                    format!("lexer error: {e:?}"),
                );
            } else {
                break;
            }
        }
    }

    fn sync_item(&mut self) {
        while let Some(t) = self.peek() {
            match t.kind {
                TokenKind::At | TokenKind::Eof => break,
                _ => {
                    self.bump();
                }
            }
        }
    }

    fn parse_program(&mut self) -> Program {
        let start = self.peek().map(|t| t.span).unwrap_or(Span {
            file: jolt_source::FileId(0),
            start: 0,
            end: 0,
        });
        let mut items = Vec::new();
        self.skip_lex_errors();
        while self.peek().is_some() {
            if self.check(&TokenKind::At) || self.check(&TokenKind::LBracket) {
                if let Some(item) = self.parse_fn_decl() {
                    items.push(item);
                } else {
                    self.sync_item();
                }
            } else if let Some(t) = self.peek() {
                self.error(
                    ParseErrorKind::UnexpectedToken,
                    t.span,
                    "expected function declaration",
                );
                self.bump();
                self.sync_item();
            } else {
                break;
            }
            self.skip_lex_errors();
        }
        let end = self.tokens.last().map(|t| t.span).unwrap_or(start);
        Program {
            span: self.merge_span(start, end),
            items,
        }
    }

    fn parse_fn_decl(&mut self) -> Option<Item> {
        let attrs = self.parse_attrs()?;
        let start = self.expect_kind(TokenKind::At, "@")?;
        let name = self.parse_fn_name()?;
        self.expect_kind(TokenKind::LParen, "(")?;
        let params = self.parse_params()?;
        self.expect_kind(TokenKind::RParen, ")")?;
        let return_type = self.parse_optional_return_type();
        let body = self.parse_fn_body()?;
        let span = self.merge_span(
            start.span,
            match &body {
                FnBody::Expr(e) => expr_span(e),
                FnBody::Block(b) => b.span,
            },
        );
        Some(Item::Fn(FnDecl {
            span,
            attrs,
            name,
            params,
            return_type,
            body,
        }))
    }

    fn parse_attrs(&mut self) -> Option<Vec<Ident>> {
        let mut attrs = Vec::new();
        while self.check(&TokenKind::LBracket) {
            self.bump();
            if !self.check(&TokenKind::RBracket) {
                loop {
                    let name = self.parse_ident("attribute name")?;
                    attrs.push(name);
                    if self.check(&TokenKind::Comma) {
                        self.bump();
                        continue;
                    }
                    break;
                }
            }
            self.expect_kind(TokenKind::RBracket, "]")?;
        }
        Some(attrs)
    }

    fn parse_fn_name(&mut self) -> Option<Ident> {
        let t = self.peek()?.clone();
        match &t.kind {
            TokenKind::Ident(s) => {
                self.bump();
                Some(Ident::new(t.span, s.clone()))
            }
            TokenKind::Fn => {
                self.bump();
                Some(Ident::new(t.span, "fn".to_string()))
            }
            _ => {
                self.error(
                    ParseErrorKind::UnexpectedToken,
                    t.span,
                    "expected function name",
                );
                None
            }
        }
    }

    fn parse_params(&mut self) -> Option<Vec<Param>> {
        let mut params = Vec::new();
        if self.check(&TokenKind::RParen) {
            return Some(params);
        }
        loop {
            let name = self.parse_ident("parameter name")?;
            self.expect_kind(TokenKind::Colon, ":")?;
            let ty = self.parse_type()?;
            let span = self.merge_span(name.span, type_span(&ty));
            params.push(Param { span, name, ty });
            if self.check(&TokenKind::Comma) {
                self.bump();
            } else {
                break;
            }
        }
        Some(params)
    }

    fn parse_optional_return_type(&mut self) -> Option<TypeExpr> {
        if self.check(&TokenKind::Arrow) || self.check(&TokenKind::SemiSemi) {
            return None;
        }
        if matches!(
            self.peek_kind(),
            Some(TokenKind::Ident(_)) | Some(TokenKind::Fn)
        ) {
            self.parse_type()
        } else {
            None
        }
    }

    fn parse_type(&mut self) -> Option<TypeExpr> {
        let t = self.peek()?.clone();
        let name = match &t.kind {
            TokenKind::Ident(s) => {
                self.bump();
                s.clone()
            }
            _ => {
                self.error(
                    ParseErrorKind::UnexpectedToken,
                    t.span,
                    "expected type name",
                );
                return None;
            }
        };
        Some(TypeExpr::Named(Spanned::new(t.span, name)))
    }

    fn parse_fn_body(&mut self) -> Option<FnBody> {
        let block = self.parse_block_body()?;
        if block.stmts.is_empty() {
            if let Some(expr) = block.tail {
                return Some(FnBody::Expr(*expr));
            }
        }
        Some(FnBody::Block(block))
    }

    fn parse_block_body(&mut self) -> Option<Block> {
        let start = self.expect_kind(TokenKind::Arrow, "->")?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::SemiSemi) && !self.at_eof() {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                break;
            }
        }
        let tail = if self.check(&TokenKind::SemiSemi) {
            None
        } else {
            self.parse_expr().ok().map(Box::new)
        };
        let end = if let Some(t) = self.expect_kind(TokenKind::SemiSemi, ";;") {
            t.span
        } else {
            self.error(
                ParseErrorKind::MissingSemiSemi,
                start.span,
                "expected ;; to close block",
            );
            start.span
        };
        Some(Block {
            span: self.merge_span(start.span, end),
            stmts,
            tail,
        })
    }

    fn at_eof(&self) -> bool {
        self.peek().is_none()
    }

    fn parse_block(&mut self) -> Option<Block> {
        self.parse_block_body()
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        self.skip_lex_errors();
        let t = self.peek()?.clone();
        match &t.kind {
            TokenKind::Dollar | TokenKind::DollarDollar => self.parse_binding_stmt(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::Loop => self.parse_loop_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::Break => self.parse_break_stmt(),
            TokenKind::Next => self.parse_next_stmt(),
            TokenKind::Ident(_) => {
                if self.is_assign_ahead() {
                    self.parse_assign_stmt()
                } else {
                    let saved = self.pos;
                    let expr = self.parse_expr().ok()?;
                    if self.check(&TokenKind::Semi) {
                        self.bump();
                        Some(Stmt::Expr(expr))
                    } else if self.check(&TokenKind::SemiSemi) {
                        self.pos = saved;
                        None
                    } else {
                        self.error(
                            ParseErrorKind::UnexpectedToken,
                            self.peek().map(|t| t.span).unwrap_or(expr_span(&expr)),
                            "expected ; or ;;",
                        );
                        None
                    }
                }
            }
            TokenKind::IntLit(_)
            | TokenKind::StringLit(_)
            | TokenKind::True
            | TokenKind::False
            | TokenKind::LParen
            | TokenKind::Minus
            | TokenKind::Not
            | TokenKind::Arrow => {
                // Expression statement or block tail — defer to tail if no trailing `;`.
                None
            }
            _ => {
                self.error(ParseErrorKind::InvalidSyntax, t.span, "expected statement");
                None
            }
        }
    }

    fn is_assign_ahead(&self) -> bool {
        if !matches!(self.peek_kind(), Some(TokenKind::Ident(_))) {
            return false;
        }
        self.tokens
            .get(self.pos + 1)
            .is_some_and(|t| t.kind == TokenKind::Eq)
    }

    fn parse_binding_stmt(&mut self) -> Option<Stmt> {
        let start = self.peek()?.clone();
        let mutable = matches!(start.kind, TokenKind::DollarDollar);
        self.bump();
        let name = self.parse_ident("binding name")?;
        let ty = if self.check(&TokenKind::Colon) {
            self.bump();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect_kind(TokenKind::Eq, "=")?;
        let value = self.parse_expr().ok()?;
        self.expect_kind(TokenKind::Semi, ";")?;
        let span = self.merge_span(start.span, expr_span(&value));
        Some(Stmt::Binding(Binding {
            span,
            mutable,
            name,
            ty,
            value,
        }))
    }

    fn parse_assign_stmt(&mut self) -> Option<Stmt> {
        let name = self.parse_ident("assignment target")?;
        self.expect_kind(TokenKind::Eq, "=")?;
        let value = self.parse_expr().ok()?;
        self.expect_kind(TokenKind::Semi, ";")?;
        let span = self.merge_span(name.span, expr_span(&value));
        Some(Stmt::Assign(Assign { span, name, value }))
    }

    fn parse_if_stmt(&mut self) -> Option<Stmt> {
        let start = self.expect_kind(TokenKind::If, "if")?;
        let cond = self.parse_expr().ok()?;
        let then_block = self.parse_block()?;
        let else_block = if self.check(&TokenKind::Else) {
            self.bump();
            Some(self.parse_block()?)
        } else {
            None
        };
        let span = self.merge_span(
            start.span,
            else_block
                .as_ref()
                .map(|b| b.span)
                .unwrap_or(then_block.span),
        );
        Some(Stmt::If(IfStmt {
            span,
            cond,
            then_block,
            else_block,
        }))
    }

    fn parse_loop_stmt(&mut self) -> Option<Stmt> {
        let start = self.expect_kind(TokenKind::Loop, "loop")?;
        let body = self.parse_block()?;
        Some(Stmt::Loop(LoopStmt {
            span: self.merge_span(start.span, body.span),
            body,
        }))
    }

    fn parse_for_stmt(&mut self) -> Option<Stmt> {
        let start = self.expect_kind(TokenKind::For, "for")?;
        let pattern = self.parse_for_pattern()?;
        self.expect_kind(TokenKind::In, "in")?;
        let iter = self.parse_expr().ok()?;
        let body = self.parse_block()?;
        Some(Stmt::For(ForStmt {
            span: self.merge_span(start.span, body.span),
            pattern,
            iter,
            body,
        }))
    }

    fn parse_for_pattern(&mut self) -> Option<ForPattern> {
        let t = self.peek()?.clone();
        match &t.kind {
            TokenKind::Ident(s) if s == "_" => {
                self.bump();
                Some(ForPattern::Wildcard(t.span))
            }
            TokenKind::Ident(s) => {
                self.bump();
                Some(ForPattern::Ident(Ident::new(t.span, s.clone())))
            }
            _ => {
                self.error(ParseErrorKind::UnexpectedToken, t.span, "expected pattern");
                None
            }
        }
    }

    fn parse_return_stmt(&mut self) -> Option<Stmt> {
        let start = self.expect_kind(TokenKind::Return, "return")?;
        let value = if self.check(&TokenKind::Semi) {
            None
        } else {
            Some(self.parse_expr().ok()?)
        };
        self.expect_kind(TokenKind::Semi, ";")?;
        Some(Stmt::Return(ReturnStmt {
            span: start.span,
            value,
        }))
    }

    fn parse_break_stmt(&mut self) -> Option<Stmt> {
        let start = self.expect_kind(TokenKind::Break, "break")?;
        self.expect_kind(TokenKind::Semi, ";")?;
        Some(Stmt::Break(BreakStmt { span: start.span }))
    }

    fn parse_next_stmt(&mut self) -> Option<Stmt> {
        let start = self.expect_kind(TokenKind::Next, "next")?;
        self.expect_kind(TokenKind::Semi, ";")?;
        Some(Stmt::Next(NextStmt { span: start.span }))
    }

    fn parse_ident(&mut self, ctx: &str) -> Option<Ident> {
        let t = self.peek()?.clone();
        match &t.kind {
            TokenKind::Ident(s) => {
                self.bump();
                Some(Ident::new(t.span, s.clone()))
            }
            _ => {
                self.error(ParseErrorKind::UnexpectedToken, t.span, ctx);
                None
            }
        }
    }

    // --- expressions (Pratt / layered precedence) ---

    fn parse_expr(&mut self) -> Result<Expr, ()> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_and_expr()?;
        while self.check(&TokenKind::OrOr) {
            let op = self.bump().unwrap();
            let right = self.parse_and_expr()?;
            left = Expr::Binary(
                Spanned::new(op.span, BinaryOp::Or),
                Box::new(left),
                Box::new(right),
            );
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_eq_expr()?;
        while self.check(&TokenKind::AndAnd) {
            let op = self.bump().unwrap();
            let right = self.parse_eq_expr()?;
            left = Expr::Binary(
                Spanned::new(op.span, BinaryOp::And),
                Box::new(left),
                Box::new(right),
            );
        }
        Ok(left)
    }

    fn parse_eq_expr(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_rel_expr()?;
        while matches!(self.peek_kind(), Some(TokenKind::EqEq | TokenKind::NotEq)) {
            let op = self.bump().unwrap();
            let bin = match op.kind {
                TokenKind::EqEq => BinaryOp::Eq,
                TokenKind::NotEq => BinaryOp::NotEq,
                _ => unreachable!(),
            };
            let right = self.parse_rel_expr()?;
            left = Expr::Binary(Spanned::new(op.span, bin), Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_rel_expr(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_add_expr()?;
        while matches!(
            self.peek_kind(),
            Some(TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge)
        ) {
            let op = self.bump().unwrap();
            let bin = match op.kind {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::Le => BinaryOp::Le,
                TokenKind::Ge => BinaryOp::Ge,
                _ => unreachable!(),
            };
            let right = self.parse_add_expr()?;
            left = Expr::Binary(Spanned::new(op.span, bin), Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_add_expr(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_mul_expr()?;
        while matches!(self.peek_kind(), Some(TokenKind::Plus | TokenKind::Minus)) {
            let op = self.bump().unwrap();
            let bin = match op.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_mul_expr()?;
            left = Expr::Binary(Spanned::new(op.span, bin), Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_mul_expr(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_prefix_expr()?;
        while matches!(
            self.peek_kind(),
            Some(
                TokenKind::Star
                    | TokenKind::Slash
                    | TokenKind::SlashSlash
                    | TokenKind::Percent
                    | TokenKind::Caret
            )
        ) {
            let op = self.bump().unwrap();
            let bin = match op.kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::SlashSlash => BinaryOp::IntDiv,
                TokenKind::Percent => BinaryOp::Mod,
                TokenKind::Caret => BinaryOp::Pow,
                _ => unreachable!(),
            };
            let right = self.parse_prefix_expr()?;
            left = Expr::Binary(Spanned::new(op.span, bin), Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_prefix_expr(&mut self) -> Result<Expr, ()> {
        if self.check(&TokenKind::Minus) {
            let op = self.bump().unwrap();
            let inner = self.parse_prefix_expr()?;
            return Ok(Expr::Unary(
                Spanned::new(op.span, UnaryOp::Neg),
                Box::new(inner),
            ));
        }
        if self.check(&TokenKind::Not) {
            let op = self.bump().unwrap();
            let inner = self.parse_prefix_expr()?;
            return Ok(Expr::Unary(
                Spanned::new(op.span, UnaryOp::Not),
                Box::new(inner),
            ));
        }
        self.parse_postfix_expr()
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr, ()> {
        let mut expr = self.parse_primary_expr()?;
        loop {
            if self.check(&TokenKind::LParen) {
                let lparen = self.bump().unwrap();
                let mut args = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if self.check(&TokenKind::Comma) {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                }
                self.expect_kind(TokenKind::RParen, ")").ok_or(())?;
                let name = match &expr {
                    Expr::Ident(i) => i.value.clone(),
                    _ => {
                        self.error(
                            ParseErrorKind::InvalidSyntax,
                            lparen.span,
                            "expected call on identifier",
                        );
                        String::new()
                    }
                };
                expr = Expr::Call(Spanned::new(lparen.span, name), args);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, ()> {
        self.skip_lex_errors();
        let Some(t) = self.peek().cloned() else {
            return Err(());
        };
        match &t.kind {
            TokenKind::IntLit(n) => {
                self.bump();
                Ok(Expr::IntLit(Spanned::new(t.span, *n)))
            }
            TokenKind::True => {
                self.bump();
                Ok(Expr::BoolLit(Spanned::new(t.span, true)))
            }
            TokenKind::False => {
                self.bump();
                Ok(Expr::BoolLit(Spanned::new(t.span, false)))
            }
            TokenKind::StringLit(s) => {
                self.bump();
                Ok(Expr::StringLit(Spanned::new(t.span, s.clone())))
            }
            TokenKind::Ident(s) => {
                self.bump();
                Ok(Expr::Ident(Spanned::new(t.span, s.clone())))
            }
            TokenKind::LParen => {
                self.bump();
                let e = self.parse_expr()?;
                self.expect_kind(TokenKind::RParen, ")").ok_or(())?;
                Ok(e)
            }
            TokenKind::Arrow => {
                let block = self.parse_block_body().ok_or(())?;
                Ok(Expr::Block(Box::new(block)))
            }
            _ => {
                self.error(
                    ParseErrorKind::UnexpectedToken,
                    t.span,
                    "expected expression",
                );
                Err(())
            }
        }
    }
}

fn expr_span(e: &Expr) -> Span {
    match e {
        Expr::IntLit(s) => s.span,
        Expr::BoolLit(s) => s.span,
        Expr::StringLit(s) => s.span,
        Expr::Ident(s) => s.span,
        Expr::Unary(s, inner) => Span {
            file: s.span.file,
            start: s.span.start,
            end: expr_span(inner).end,
        },
        Expr::Binary(s, l, r) => Span {
            file: s.span.file,
            start: expr_span(l).start.min(s.span.start),
            end: expr_span(r).end.max(s.span.end),
        },
        Expr::Call(s, args) => {
            let end = args
                .last()
                .map(expr_span)
                .map(|s| s.end)
                .unwrap_or(s.span.end);
            Span {
                file: s.span.file,
                start: s.span.start,
                end,
            }
        }
        Expr::Block(b) => b.span,
    }
}

fn type_span(ty: &TypeExpr) -> Span {
    match ty {
        TypeExpr::Named(n) => n.span,
    }
}

/// Parse a token stream into a (possibly partial) AST and accumulated errors.
pub fn parse_tokens(tokens: &[Token]) -> ParseResult {
    let mut p = Parser::new(tokens);
    let program = p.parse_program();
    ParseResult {
        program,
        errors: p.errors,
    }
}
