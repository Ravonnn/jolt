use std::fmt;

use crate::*;

/// Pretty-print an AST for tests (not canonical source).
pub fn debug_print(program: &Program) -> String {
    let mut out = String::new();
    write_program(&mut out, program).ok();
    out
}

fn write_program(w: &mut impl fmt::Write, program: &Program) -> fmt::Result {
    for item in &program.items {
        match item {
            Item::Fn(f) => write_fn(w, f, 0)?,
        }
    }
    Ok(())
}

fn write_fn(w: &mut impl fmt::Write, f: &FnDecl, indent: usize) -> fmt::Result {
    let pad = "  ".repeat(indent);
    write!(w, "{pad}@{}(", f.name.name)?;
    for (i, p) in f.params.iter().enumerate() {
        if i > 0 {
            write!(w, ", ")?;
        }
        write!(w, "{}: ", p.name.name)?;
        write_type(w, &p.ty)?;
    }
    write!(w, ")")?;
    if let Some(rt) = &f.return_type {
        write!(w, " ")?;
        write_type(w, rt)?;
    }
    writeln!(w)?;
    write_fn_body(w, &f.body, indent)?;
    Ok(())
}

fn write_type(w: &mut impl fmt::Write, ty: &TypeExpr) -> fmt::Result {
    match ty {
        TypeExpr::Named(n) => write!(w, "{}", n.value),
    }
}

fn write_fn_body(w: &mut impl fmt::Write, body: &FnBody, indent: usize) -> fmt::Result {
    match body {
        FnBody::Expr(e) => {
            let pad = "  ".repeat(indent);
            write!(w, "{pad}-> ")?;
            write_expr(w, e)?;
            writeln!(w, " ;;")?;
        }
        FnBody::Block(b) => write_block(w, b, indent)?,
    }
    Ok(())
}

fn write_block(w: &mut impl fmt::Write, b: &Block, indent: usize) -> fmt::Result {
    let pad = "  ".repeat(indent);
    writeln!(w, "{pad}->")?;
    for s in &b.stmts {
        write_stmt(w, s, indent + 1)?;
    }
    if let Some(t) = &b.tail {
        let p = "  ".repeat(indent + 1);
        write!(w, "{p}")?;
        write_expr(w, t)?;
        writeln!(w)?;
    }
    writeln!(w, "{pad};;")?;
    Ok(())
}

fn write_stmt(w: &mut impl fmt::Write, s: &Stmt, indent: usize) -> fmt::Result {
    let pad = "  ".repeat(indent);
    match s {
        Stmt::Binding(b) => {
            write!(
                w,
                "{pad}{}{} ",
                if b.mutable { "$$" } else { "$" },
                b.name.name
            )?;
            if let Some(ty) = &b.ty {
                write_type(w, ty)?;
                write!(w, " ")?;
            }
            write!(w, "= ")?;
            write_expr(w, &b.value)?;
            writeln!(w, ";")?;
        }
        Stmt::Assign(a) => {
            write!(w, "{pad}{} = ", a.name.name)?;
            write_expr(w, &a.value)?;
            writeln!(w, ";")?;
        }
        Stmt::Expr(e) => {
            write!(w, "{pad}")?;
            write_expr(w, e)?;
            writeln!(w, ";")?;
        }
        Stmt::If(i) => {
            write!(w, "{pad}if ")?;
            write_expr(w, &i.cond)?;
            writeln!(w)?;
            write_block(w, &i.then_block, indent + 1)?;
            if let Some(el) = &i.else_block {
                writeln!(w, "{pad}else")?;
                write_block(w, el, indent + 1)?;
            }
        }
        Stmt::Loop(l) => {
            writeln!(w, "{pad}loop")?;
            write_block(w, &l.body, indent + 1)?;
        }
        Stmt::For(f) => {
            write!(w, "{pad}for ")?;
            match &f.pattern {
                ForPattern::Ident(id) => write!(w, "{}", id.name)?,
                ForPattern::Wildcard(_) => write!(w, "_")?,
            }
            write!(w, " in ")?;
            write_expr(w, &f.iter)?;
            writeln!(w)?;
            write_block(w, &f.body, indent + 1)?;
        }
        Stmt::Return(r) => {
            write!(w, "{pad}return")?;
            if let Some(v) = &r.value {
                write!(w, " ")?;
                write_expr(w, v)?;
            }
            writeln!(w, ";")?;
        }
        Stmt::Break(_) => writeln!(w, "{pad}break;")?,
        Stmt::Next(_) => writeln!(w, "{pad}next;")?,
    }
    Ok(())
}

fn write_expr(w: &mut impl fmt::Write, e: &Expr) -> fmt::Result {
    match e {
        Expr::IntLit(n) => write!(w, "{}", n.value),
        Expr::BoolLit(b) => write!(w, "{}", b.value),
        Expr::StringLit(s) => write!(w, "\"{}\"", s.value),
        Expr::Ident(i) => write!(w, "{}", i.value),
        Expr::Unary(op, inner) => {
            let s = match op.value {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
            };
            write!(w, "{s}")?;
            write_expr(w, inner)
        }
        Expr::Binary(op, l, r) => {
            write!(w, "(")?;
            write_expr(w, l)?;
            write!(w, " {} ", bin_op_str(op.value))?;
            write_expr(w, r)?;
            write!(w, ")")
        }
        Expr::Call(name, args) => {
            write!(w, "{}(", name.value)?;
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                write_expr(w, a)?;
            }
            write!(w, ")")
        }
        Expr::Block(b) => {
            write!(w, "{{ ")?;
            for s in &b.stmts {
                write_stmt(w, s, 0)?;
            }
            if let Some(t) = &b.tail {
                write_expr(w, t)?;
            }
            write!(w, " }}")
        }
    }
}

fn bin_op_str(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Or => "||",
        BinaryOp::And => "&&",
        BinaryOp::Eq => "==",
        BinaryOp::NotEq => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::Le => "<=",
        BinaryOp::Ge => ">=",
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::IntDiv => "//",
        BinaryOp::Mod => "%",
        BinaryOp::Pow => "^",
    }
}
