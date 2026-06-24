use std::fmt::{self, Write};

use jolt_ast::{
    Assign, BinaryOp, Binding, Block, Expr, FnBody, FnDecl, ForPattern, ForStmt, IfStmt, Item,
    LoopStmt, Program, ReturnStmt, Stmt, TypeExpr, UnaryOp,
};

const INDENT: &str = "    ";

/// Canonical Tiny source for a parsed program (ends with a trailing newline).
pub fn print_program(program: &Program) -> String {
    let mut out = String::new();
    for (i, item) in program.items.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        match item {
            Item::Fn(f) => {
                print_fn(&mut out, f).expect("fmt");
            }
        }
    }
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn print_fn(w: &mut String, f: &FnDecl) -> fmt::Result {
    if !f.attrs.is_empty() {
        write!(w, "[")?;
        for (i, a) in f.attrs.iter().enumerate() {
            if i > 0 {
                write!(w, ", ")?;
            }
            write!(w, "{}", a.name)?;
        }
        writeln!(w, "]")?;
    }
    write!(w, "@{}(", f.name.name)?;
    for (i, p) in f.params.iter().enumerate() {
        if i > 0 {
            write!(w, ", ")?;
        }
        write!(w, "{}: ", p.name.name)?;
        print_type(w, &p.ty)?;
    }
    write!(w, ")")?;
    if let Some(rt) = &f.return_type {
        write!(w, " ")?;
        print_type(w, rt)?;
    }
    match &f.body {
        FnBody::Expr(e) => {
            write!(w, " -> ")?;
            print_expr(w, e, Prec::Top)?;
            writeln!(w, " ;;")?;
        }
        FnBody::Block(b) => {
            writeln!(w, " ->")?;
            print_block_body(w, b, 1)?;
            write!(w, ";;")?;
            writeln!(w)?;
        }
    }
    Ok(())
}

fn print_type(w: &mut String, ty: &TypeExpr) -> fmt::Result {
    match ty {
        TypeExpr::Named(n) => write!(w, "{}", n.value),
    }
}

fn print_block_body(w: &mut String, b: &Block, depth: usize) -> fmt::Result {
    let pad = indent(depth);
    for stmt in &b.stmts {
        print_stmt(w, stmt, depth)?;
    }
    if let Some(tail) = &b.tail {
        write!(w, "{pad}")?;
        print_expr(w, tail, Prec::Top)?;
        writeln!(w)?;
    }
    Ok(())
}

fn print_block_expr(w: &mut String, b: &Block, depth: usize) -> fmt::Result {
    writeln!(w, "->")?;
    print_block_body(w, b, depth)?;
    write!(w, "{};;", indent(depth - 1))?;
    Ok(())
}

fn print_stmt(w: &mut String, stmt: &Stmt, depth: usize) -> fmt::Result {
    let pad = indent(depth);
    match stmt {
        Stmt::Binding(b) => print_binding(w, b, depth),
        Stmt::Assign(a) => print_assign(w, a, depth),
        Stmt::Expr(e) => {
            write!(w, "{pad}")?;
            print_expr(w, e, Prec::Top)?;
            writeln!(w, ";")?;
            Ok(())
        }
        Stmt::If(i) => print_if_stmt(w, i, depth),
        Stmt::Loop(l) => print_loop_stmt(w, l, depth),
        Stmt::For(f) => print_for_stmt(w, f, depth),
        Stmt::Return(r) => print_return(w, r, depth),
        Stmt::Break(_) => writeln!(w, "{pad}break;"),
        Stmt::Next(_) => writeln!(w, "{pad}next;"),
    }
}

fn print_binding(w: &mut String, b: &Binding, depth: usize) -> fmt::Result {
    let pad = indent(depth);
    write!(
        w,
        "{pad}{}{}",
        if b.mutable { "$$" } else { "$" },
        b.name.name
    )?;
    if let Some(ty) = &b.ty {
        write!(w, ": ")?;
        print_type(w, ty)?;
    }
    write!(w, " = ")?;
    print_expr(w, &b.value, Prec::Top)?;
    writeln!(w, ";")?;
    Ok(())
}

fn print_assign(w: &mut String, a: &Assign, depth: usize) -> fmt::Result {
    let pad = indent(depth);
    write!(w, "{pad}{} = ", a.name.name)?;
    print_expr(w, &a.value, Prec::Top)?;
    writeln!(w, ";")?;
    Ok(())
}

fn print_if_stmt(w: &mut String, if_stmt: &IfStmt, depth: usize) -> fmt::Result {
    let pad = indent(depth);
    write!(w, "{pad}if ")?;
    print_expr(w, &if_stmt.cond, Prec::Top)?;
    writeln!(w, " ->")?;
    print_block_body(w, &if_stmt.then_block, depth + 1)?;
    if let Some(else_block) = &if_stmt.else_block {
        write!(w, "{pad};; else ->")?;
        writeln!(w)?;
        print_block_body(w, else_block, depth + 1)?;
    }
    writeln!(w, "{pad};;")?;
    Ok(())
}

fn print_loop_stmt(w: &mut String, loop_stmt: &LoopStmt, depth: usize) -> fmt::Result {
    let pad = indent(depth);
    writeln!(w, "{pad}loop ->")?;
    print_block_body(w, &loop_stmt.body, depth + 1)?;
    writeln!(w, "{pad};;")?;
    Ok(())
}

fn print_for_stmt(w: &mut String, for_stmt: &ForStmt, depth: usize) -> fmt::Result {
    let pad = indent(depth);
    write!(w, "{pad}for ")?;
    match &for_stmt.pattern {
        ForPattern::Ident(id) => write!(w, "{}", id.name)?,
        ForPattern::Wildcard(_) => write!(w, "_")?,
    }
    write!(w, " in ")?;
    print_expr(w, &for_stmt.iter, Prec::Top)?;
    writeln!(w, " ->")?;
    print_block_body(w, &for_stmt.body, depth + 1)?;
    writeln!(w, "{pad};;")?;
    Ok(())
}

fn print_return(w: &mut String, ret: &ReturnStmt, depth: usize) -> fmt::Result {
    let pad = indent(depth);
    write!(w, "{pad}return")?;
    if let Some(v) = &ret.value {
        write!(w, " ")?;
        print_expr(w, v, Prec::Top)?;
    }
    writeln!(w, ";")?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    Top = 0,
    Or = 1,
    And = 2,
    Eq = 3,
    Compare = 4,
    Add = 5,
    Mul = 6,
    Unary = 7,
    Atom = 8,
}

fn print_expr(w: &mut String, e: &Expr, parent: Prec) -> fmt::Result {
    match e {
        Expr::IntLit(n) => write!(w, "{}", n.value),
        Expr::BoolLit(b) => write!(w, "{}", b.value),
        Expr::StringLit(s) => write!(w, "\"{}\"", escape_string(&s.value)),
        Expr::Ident(i) => write!(w, "{}", i.value),
        Expr::Unary(op, inner) => {
            let prec = Prec::Unary;
            if parent > prec {
                write!(w, "(")?;
            }
            let sym = match op.value {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
            };
            write!(w, "{sym}")?;
            print_expr(w, inner, prec)?;
            if parent > prec {
                write!(w, ")")?;
            }
            Ok(())
        }
        Expr::Binary(op, left, right) => {
            let prec = bin_prec(op.value);
            if parent > prec {
                write!(w, "(")?;
            }
            print_expr(w, left, prec)?;
            write!(w, " {} ", bin_op_str(op.value))?;
            print_expr(w, right, next_prec(op.value))?;
            if parent > prec {
                write!(w, ")")?;
            }
            Ok(())
        }
        Expr::Call(name, args) => {
            if parent > Prec::Atom {
                write!(w, "(")?;
            }
            write!(w, "{}", name.value)?;
            write!(w, "(")?;
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                print_expr(w, a, Prec::Top)?;
            }
            write!(w, ")")?;
            if parent > Prec::Atom {
                write!(w, ")")?;
            }
            Ok(())
        }
        Expr::Block(b) => {
            if parent > Prec::Atom {
                write!(w, "(")?;
            }
            print_block_expr(w, b, 1)?;
            if parent > Prec::Atom {
                write!(w, ")")?;
            }
            Ok(())
        }
    }
}

fn bin_prec(op: BinaryOp) -> Prec {
    match op {
        BinaryOp::Or => Prec::Or,
        BinaryOp::And => Prec::And,
        BinaryOp::Eq | BinaryOp::NotEq => Prec::Eq,
        BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => Prec::Compare,
        BinaryOp::Add | BinaryOp::Sub => Prec::Add,
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::IntDiv | BinaryOp::Mod | BinaryOp::Pow => {
            Prec::Mul
        }
    }
}

fn next_prec(op: BinaryOp) -> Prec {
    match op {
        BinaryOp::Or => Prec::And,
        BinaryOp::And => Prec::Eq,
        BinaryOp::Eq | BinaryOp::NotEq => Prec::Compare,
        BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => Prec::Add,
        BinaryOp::Add | BinaryOp::Sub => Prec::Mul,
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::IntDiv | BinaryOp::Mod | BinaryOp::Pow => {
            Prec::Unary
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

fn indent(depth: usize) -> String {
    INDENT.repeat(depth)
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
