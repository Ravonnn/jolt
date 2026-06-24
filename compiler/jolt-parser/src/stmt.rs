use jolt_ast::{Assign, Binding, Block, ForPattern, ForStmt, IfStmt, Stmt, TypeExpr};
use jolt_lexer::TokenKind;

use crate::cursor::ParserCursor;
use crate::error::{ParseError, ParseErrorKind};
use crate::expr::{self, parse_expr};

pub fn parse_block(cur: &mut ParserCursor<'_>) -> Block {
    let start = cur.current_span();
    cur.expect(TokenKind::Arrow, "expected '->'");
    parse_block_inner(cur, start)
}

pub fn parse_block_inner(cur: &mut ParserCursor<'_>, start: jolt_source::Span) -> Block {
    let mut stmts = Vec::new();
    let mut tail = None;

    while cur.peek_kind() != Some(TokenKind::SemiSemi) && !cur.is_at_end() {
        if let Some(stmt) = try_parse_stmt(cur) {
            stmts.push(stmt);
            continue;
        }

        if let Some(expr) = parse_expr(cur) {
            if cur.peek_kind() == Some(TokenKind::Semi) {
                cur.bump();
                let sp = expr::expr_span(&expr);
                stmts.push(Stmt::Expr(expr));
                let _ = sp;
                continue;
            }
            if cur.peek_kind() == Some(TokenKind::SemiSemi) || cur.is_at_end() {
                tail = Some(expr);
                break;
            }
            cur.push_error(ParseError::new(
                ParseErrorKind::MissingSemi,
                "expected ';' or ';;' after expression",
                expr::expr_span(&expr),
            ));
            cur.synchronize();
            continue;
        }

        cur.push_error(ParseError::new(
            ParseErrorKind::UnexpectedToken,
            "expected statement or expression",
            cur.current_span(),
        ));
        cur.synchronize();
    }

    cur.expect(TokenKind::SemiSemi, "expected ';;'");
    let span = cur.span_from(start);
    Block {
        stmts,
        tail,
        span,
    }
}

fn try_parse_stmt(cur: &mut ParserCursor<'_>) -> Option<Stmt> {
    let start = cur.current_span();
    match cur.peek_kind()? {
        TokenKind::Dollar | TokenKind::DollarDollar => parse_binding(cur),
        TokenKind::If => parse_if(cur),
        TokenKind::Loop => {
            cur.bump();
            let body = parse_block(cur);
            Some(Stmt::Loop(body))
        }
        TokenKind::For => parse_for(cur),
        TokenKind::Return => {
            cur.bump();
            let value = if matches!(
                cur.peek_kind(),
                Some(TokenKind::Semi) | Some(TokenKind::SemiSemi)
            ) {
                None
            } else {
                parse_expr(cur)
            };
            cur.expect(TokenKind::Semi, "expected ';' after return");
            Some(Stmt::Return {
                value,
                span: cur.span_from(start),
            })
        }
        TokenKind::Break => {
            cur.bump();
            cur.expect(TokenKind::Semi, "expected ';' after break");
            Some(Stmt::Break {
                span: cur.span_from(start),
            })
        }
        TokenKind::Next => {
            cur.bump();
            cur.expect(TokenKind::Semi, "expected ';' after next");
            Some(Stmt::Next {
                span: cur.span_from(start),
            })
        }
        TokenKind::Ident(_) => {
            if cur.peek_kind_at(1) == Some(TokenKind::Eq) {
                parse_assign(cur)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_binding(cur: &mut ParserCursor<'_>) -> Option<Stmt> {
    let start = cur.current_span();
    let mutable = match cur.peek_kind()? {
        TokenKind::DollarDollar => {
            cur.bump();
            true
        }
        TokenKind::Dollar => {
            cur.bump();
            false
        }
        _ => return None,
    };
    let (name, _) = cur.expect_ident("expected name after $")?;
    let ty = if cur.peek_kind() == Some(TokenKind::Colon) {
        cur.bump();
        Some(parse_type(cur)?)
    } else {
        None
    };
    cur.expect(TokenKind::Eq, "expected '=' in binding");
    let value = parse_expr(cur)?;
    cur.expect(TokenKind::Semi, "expected ';' after binding");
    Some(Stmt::Binding(Binding {
        mutable,
        name,
        ty,
        value,
        span: cur.span_from(start),
    }))
}

fn parse_assign(cur: &mut ParserCursor<'_>) -> Option<Stmt> {
    let start = cur.current_span();
    let name_tok = cur.bump()?;
    let name = match name_tok.kind {
        TokenKind::Ident(n) => n,
        _ => return None,
    };
    cur.expect(TokenKind::Eq, "expected '='");
    let value = parse_expr(cur)?;
    cur.expect(TokenKind::Semi, "expected ';' after assignment");
    Some(Stmt::Assign(Assign {
        name,
        value,
        span: cur.span_from(start),
    }))
}

fn parse_if(cur: &mut ParserCursor<'_>) -> Option<Stmt> {
    let start = cur.current_span();
    cur.bump(); // if
    let cond = parse_expr(cur)?;
    let then_block = parse_block(cur);
    let else_block = if cur.peek_kind() == Some(TokenKind::Else) {
        cur.bump();
        if cur.peek_kind() == Some(TokenKind::If) {
            let else_if = parse_if(cur)?;
            let span = cur.span_from(start);
            return Some(Stmt::If(IfStmt {
                cond,
                then_block,
                else_block: Some(else_if_stmt_to_block(else_if, span)),
                span,
            }));
        }
        Some(parse_block(cur))
    } else {
        None
    };
    Some(Stmt::If(IfStmt {
        cond,
        then_block,
        else_block,
        span: cur.span_from(start),
    }))
}

fn else_if_stmt_to_block(stmt: Stmt, span: jolt_source::Span) -> Block {
    match stmt {
        Stmt::If(i) => Block {
            stmts: vec![Stmt::If(i)],
            tail: None,
            span,
        },
        other => Block {
            stmts: vec![other],
            tail: None,
            span,
        },
    }
}

fn parse_for(cur: &mut ParserCursor<'_>) -> Option<Stmt> {
    let start = cur.current_span();
    cur.bump();
    let pattern = if matches!(cur.peek_kind(), Some(TokenKind::Ident(_))) {
        let t = cur.bump()?;
        match t.kind {
            TokenKind::Ident(n) if n == "_" => ForPattern::Wildcard,
            TokenKind::Ident(n) => ForPattern::Ident(n),
            _ => ForPattern::Wildcard,
        }
    } else {
        ForPattern::Wildcard
    };
    cur.expect(TokenKind::In, "expected 'in' in for");
    let iter = parse_expr(cur)?;
    let body = parse_block(cur);
    Some(Stmt::For(ForStmt {
        pattern,
        iter,
        body,
        span: cur.span_from(start),
    }))
}

pub fn parse_type(cur: &mut ParserCursor<'_>) -> Option<TypeExpr> {
    let t = cur.bump()?;
    match t.kind {
        TokenKind::Ident(name) => Some(TypeExpr::Named(name)),
        _ => {
            cur.push_error(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "expected type name",
                t.span,
            ));
            None
        }
    }
}
