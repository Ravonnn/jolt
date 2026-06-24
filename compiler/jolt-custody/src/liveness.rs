use std::collections::HashMap;

use jolt_ast::{Block, Expr, Stmt};

/// Last statement index (or `stmts.len()` for `tail`) where each binding name is referenced.
pub fn block_last_uses(block: &Block) -> HashMap<String, usize> {
    let mut uses = HashMap::new();
    for (i, stmt) in block.stmts.iter().enumerate() {
        record_stmt(stmt, i, &mut uses);
    }
    let tail_idx = block.stmts.len();
    if let Some(tail) = &block.tail {
        record_expr(tail, tail_idx, &mut uses);
    } else if let Some(last) = block.stmts.last() {
        match last {
            Stmt::If(i) => {
                record_expr(&i.cond, tail_idx, &mut uses);
                record_block(&i.then_block, tail_idx, &mut uses);
                if let Some(else_b) = &i.else_block {
                    record_block(else_b, tail_idx, &mut uses);
                }
            }
            Stmt::Expr(e) => record_expr(e, tail_idx, &mut uses),
            _ => {}
        }
    }
    uses
}

fn record_block(block: &Block, at: usize, uses: &mut HashMap<String, usize>) {
    for stmt in &block.stmts {
        record_stmt(stmt, at, uses);
    }
    if let Some(tail) = &block.tail {
        record_expr(tail, at, uses);
    }
}

fn record_stmt(stmt: &Stmt, idx: usize, uses: &mut HashMap<String, usize>) {
    match stmt {
        Stmt::Binding(b) => record_expr(&b.value, idx, uses),
        Stmt::Assign(a) => record_expr(&a.value, idx, uses),
        Stmt::Expr(e) => record_expr(e, idx, uses),
        Stmt::If(i) => {
            record_expr(&i.cond, idx, uses);
            record_block(&i.then_block, idx, uses);
            if let Some(else_b) = &i.else_block {
                record_block(else_b, idx, uses);
            }
        }
        Stmt::Loop(l) => record_block(&l.body, idx, uses),
        Stmt::For(f) => {
            record_expr(&f.iter, idx, uses);
            record_block(&f.body, idx, uses);
        }
        Stmt::Return(r) => {
            if let Some(v) = &r.value {
                record_expr(v, idx, uses);
            }
        }
        Stmt::Break(_) | Stmt::Next(_) => {}
    }
}

fn record_expr(expr: &Expr, idx: usize, uses: &mut HashMap<String, usize>) {
    match expr {
        Expr::Ident(sp) => {
            uses.insert(sp.value.clone(), idx);
        }
        Expr::Unary(_, inner) => record_expr(inner, idx, uses),
        Expr::Binary(_, left, right) => {
            record_expr(left, idx, uses);
            record_expr(right, idx, uses);
        }
        Expr::Call(_, args) => {
            for arg in args {
                record_expr(arg, idx, uses);
            }
        }
        Expr::Block(b) => record_block(b, idx, uses),
        Expr::IntLit(_) | Expr::BoolLit(_) | Expr::StringLit(_) => {}
    }
}
