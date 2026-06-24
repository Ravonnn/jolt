use jolt_ast::{span_between, FnBody, FnDecl, Item, Param, Program};
use jolt_lexer::TokenKind;

use crate::cursor::ParserCursor;
use crate::error::{ParseError, ParseErrorKind};
use crate::expr::parse_expr;
use crate::stmt::{parse_block_inner, parse_type};

pub fn parse_program(cur: &mut ParserCursor<'_>) -> Program {
    let start = cur.current_span();
    let mut items = Vec::new();

    while !cur.is_at_end() {
        if let Some(item) = parse_item(cur) {
            items.push(item);
        } else if cur.is_at_end() {
            break;
        } else {
            cur.push_error(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "expected function declaration",
                cur.current_span(),
            ));
            cur.synchronize();
            if cur.peek_kind() == Some(TokenKind::At) {
                continue;
            }
            if !cur.is_at_end() {
                cur.bump();
            }
        }
    }

    let span = if items.is_empty() {
        start
    } else {
        let first = item_span(&items[0]);
        let last = item_span(items.last().unwrap());
        span_between(first, last)
    };

    Program { items, span }
}

pub fn parse_item(cur: &mut ParserCursor<'_>) -> Option<Item> {
    let start = cur.current_span();
    let attrs = parse_attrs(cur)?;
    cur.expect(TokenKind::At, "expected '@' before function name")?;
    let (name, _) = cur.expect_ident("expected function name after @")?;
    cur.expect(TokenKind::LParen, "expected '('")?;

    let mut params = Vec::new();
    if cur.peek_kind() != Some(TokenKind::RParen) {
        loop {
            let pstart = cur.current_span();
            let (pname, _) = cur.expect_ident("expected parameter name")?;
            cur.expect(TokenKind::Colon, "expected ':' after parameter name")?;
            let ty = parse_type(cur)?;
            params.push(Param {
                name: pname,
                ty,
                span: cur.span_from(pstart),
            });
            if cur.peek_kind() == Some(TokenKind::Comma) {
                cur.bump();
                continue;
            }
            break;
        }
    }
    cur.expect(TokenKind::RParen, "expected ')'")?;

    let return_type = if matches!(cur.peek_kind(), Some(TokenKind::Ident(_)))
        && cur.peek_kind_at(1) == Some(TokenKind::Arrow)
    {
        parse_type(cur)
    } else {
        None
    };

    let body = parse_fn_body(cur)?;
    let span = cur.span_from(start);
    Some(Item::Fn(FnDecl {
        attrs,
        name,
        params,
        return_type,
        body,
        span,
    }))
}

fn parse_attrs(cur: &mut ParserCursor<'_>) -> Option<Vec<jolt_ast::Ident>> {
    let mut attrs = Vec::new();
    while cur.peek_kind() == Some(TokenKind::LBracket) {
        cur.bump();
        if cur.peek_kind() != Some(TokenKind::RBracket) {
            loop {
                let (name, _) = cur.expect_ident("expected attribute name")?;
                attrs.push(name);
                if cur.peek_kind() == Some(TokenKind::Comma) {
                    cur.bump();
                    continue;
                }
                break;
            }
        }
        cur.expect(TokenKind::RBracket, "expected ']' after attributes")?;
    }
    Some(attrs)
}

fn parse_fn_body(cur: &mut ParserCursor<'_>) -> Option<FnBody> {
    cur.expect(TokenKind::Arrow, "expected '->' before function body")?;
    let start = cur.current_span();

    if is_stmt_start(cur.peek_kind()) {
        let block = parse_block_inner(cur, start);
        return Some(FnBody::Block(block));
    }

    if is_expr_start(cur.peek_kind()) {
        if let Some(expr) = parse_expr(cur) {
            cur.expect(TokenKind::SemiSemi, "expected ';;' after expression body");
            return Some(FnBody::Expr(expr));
        }
    }

    let block = parse_block_inner(cur, start);
    Some(FnBody::Block(block))
}

fn is_expr_start(kind: Option<TokenKind>) -> bool {
    matches!(
        kind,
        Some(TokenKind::IntLit(_))
            | Some(TokenKind::True)
            | Some(TokenKind::False)
            | Some(TokenKind::StringLit(_))
            | Some(TokenKind::Ident(_))
            | Some(TokenKind::Minus)
            | Some(TokenKind::Not)
            | Some(TokenKind::LParen)
            | Some(TokenKind::Arrow)
    )
}

fn is_stmt_start(kind: Option<TokenKind>) -> bool {
    matches!(
        kind,
        Some(TokenKind::Dollar)
            | Some(TokenKind::DollarDollar)
            | Some(TokenKind::If)
            | Some(TokenKind::Loop)
            | Some(TokenKind::For)
            | Some(TokenKind::Return)
            | Some(TokenKind::Break)
            | Some(TokenKind::Next)
    )
}

fn item_span(item: &Item) -> jolt_source::Span {
    match item {
        Item::Fn(f) => f.span,
    }
}
