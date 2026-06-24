use jolt_ast::{span_between, BinOp, Expr, UnaryOp};
use jolt_lexer::TokenKind;
use jolt_source::Span;

use crate::cursor::ParserCursor;
use crate::stmt;

/// Pratt parser for Tiny expressions (`jolt-grammar.md` §9).
pub fn parse_expr(cur: &mut ParserCursor<'_>) -> Option<Expr> {
    parse_expr_bp(cur, 0)
}

fn parse_expr_bp(cur: &mut ParserCursor<'_>, min_bp: u8) -> Option<Expr> {
    let mut left = parse_prefix(cur)?;
    let left_start = expr_span(&left);

    loop {
        let (op, l_bp, r_bp) = match cur.peek_kind()? {
            TokenKind::OrOr => (Some(BinOp::Or), 10, 11),
            TokenKind::AndAnd => (Some(BinOp::And), 20, 21),
            TokenKind::EqEq => (Some(BinOp::Eq), 30, 31),
            TokenKind::NotEq => (Some(BinOp::NotEq), 30, 31),
            TokenKind::Lt => (Some(BinOp::Lt), 40, 41),
            TokenKind::Gt => (Some(BinOp::Gt), 40, 41),
            TokenKind::Le => (Some(BinOp::Le), 40, 41),
            TokenKind::Ge => (Some(BinOp::Ge), 40, 41),
            TokenKind::Plus => (Some(BinOp::Add), 50, 51),
            TokenKind::Minus => (Some(BinOp::Sub), 50, 51),
            TokenKind::Star => (Some(BinOp::Mul), 60, 61),
            TokenKind::Slash => (Some(BinOp::Div), 60, 61),
            TokenKind::SlashSlash => (Some(BinOp::IntDiv), 60, 61),
            TokenKind::Percent => (Some(BinOp::Mod), 60, 61),
            TokenKind::Caret => (Some(BinOp::Pow), 60, 59), // right-assoc
            TokenKind::LParen => {
                left = parse_call(cur, left)?;
                continue;
            }
            _ => break,
        };

        let Some(op) = op else { break };
        if l_bp < min_bp {
            break;
        }
        cur.bump();
        let right = parse_expr_bp(cur, r_bp)?;
        let right_span = expr_span(&right);
        left = Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span: span_between(left_start, right_span),
        };
    }

    Some(left)
}

fn parse_prefix(cur: &mut ParserCursor<'_>) -> Option<Expr> {
    let start = cur.current_span();
    match cur.peek_kind()? {
        TokenKind::Minus => {
            cur.bump();
            let inner = parse_expr_bp(cur, 70)?;
            let sp = span_between(start, expr_span(&inner));
            Some(Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(inner),
                span: sp,
            })
        }
        TokenKind::Not => {
            cur.bump();
            let inner = parse_expr_bp(cur, 70)?;
            let sp = span_between(start, expr_span(&inner));
            Some(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(inner),
                span: sp,
            })
        }
        TokenKind::IntLit(n) => {
            let t = cur.bump()?;
            Some(Expr::IntLit {
                value: n,
                span: t.span,
            })
        }
        TokenKind::True => {
            let t = cur.bump()?;
            Some(Expr::BoolLit {
                value: true,
                span: t.span,
            })
        }
        TokenKind::False => {
            let t = cur.bump()?;
            Some(Expr::BoolLit {
                value: false,
                span: t.span,
            })
        }
        TokenKind::StringLit(ref s) => {
            let t = cur.bump()?;
            Some(Expr::StringLit {
                value: s.clone(),
                span: t.span,
            })
        }
        TokenKind::Ident(ref name) => {
            let t = cur.bump()?;
            if cur.peek_kind() == Some(TokenKind::LParen) {
                return parse_call_after_name(cur, name.clone(), t.span);
            }
            Some(Expr::Ident {
                name: name.clone(),
                span: t.span,
            })
        }
        TokenKind::Arrow => {
            let block = stmt::parse_block(cur);
            Some(Expr::Block(Box::new(block)))
        }
        TokenKind::LParen => {
            cur.bump();
            let e = parse_expr(cur)?;
            cur.expect(TokenKind::RParen, "expected ')'");
            Some(e)
        }
        _ => None,
    }
}

fn parse_call(cur: &mut ParserCursor<'_>, callee_expr: Expr) -> Option<Expr> {
    let (name, start) = match callee_expr {
        Expr::Ident { name, span } => (name, span),
        other => {
            let sp = expr_span(&other);
            cur.push_error(crate::error::ParseError::new(
                crate::error::ParseErrorKind::UnexpectedToken,
                "only identifier calls supported",
                sp,
            ));
            return Some(other);
        }
    };
    parse_call_after_name(cur, name, start)
}

fn parse_call_after_name(
    cur: &mut ParserCursor<'_>,
    callee: String,
    start: Span,
) -> Option<Expr> {
    cur.expect(TokenKind::LParen, "expected '('")?;
    let mut args = Vec::new();
    if cur.peek_kind() != Some(TokenKind::RParen) {
        loop {
            let arg = parse_expr(cur)?;
            args.push(arg);
            if cur.peek_kind() == Some(TokenKind::Comma) {
                cur.bump();
                continue;
            }
            break;
        }
    }
    let close = cur.expect(TokenKind::RParen, "expected ')'")?;
    let span = span_between(start, close.span);
    Some(Expr::Call {
        callee,
        args,
        span,
    })
}

pub fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::IntLit { span, .. }
        | Expr::BoolLit { span, .. }
        | Expr::StringLit { span, .. }
        | Expr::Ident { span, .. }
        | Expr::Unary { span, .. }
        | Expr::Binary { span, .. }
        | Expr::Call { span, .. } => *span,
        Expr::Block(b) => b.span,
    }
}
