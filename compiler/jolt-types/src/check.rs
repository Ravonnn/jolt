use std::collections::HashMap;

use jolt_ast::{
    Assign, BinaryOp, Binding, Block, Expr, FnBody, FnDecl, ForPattern, ForStmt, IfStmt, Item,
    LoopStmt, Program, ReturnStmt, Stmt, TypeExpr, UnaryOp,
};
use jolt_diagnostics::{Diagnostic, Diagnostics};
use jolt_source::Span;

use crate::error::{
    assign_mismatch, binding_annotation_mismatch, binop_mismatch, branch_mismatch,
    cannot_borrow_copy, claim_requires_mutable, condition_not_bool, deref_requires_borrow,
    for_iter_not_int, mismatch, println_arg_mismatch, return_type_mismatch, unknown_function,
    unknown_type, wrong_arg_count,
};
use crate::ty::Ty;

#[derive(Debug, Clone)]
struct FnSig {
    params: Vec<Ty>,
    ret: Ty,
}

#[derive(Debug, Clone, Copy)]
struct LocalBinding {
    ty: Ty,
    mutable: bool,
}

struct TypeScope {
    bindings: HashMap<String, LocalBinding>,
}

struct TypeChecker {
    diagnostics: Diagnostics,
    scopes: Vec<TypeScope>,
    functions: HashMap<String, FnSig>,
    current_return: Option<Ty>,
    in_loop: u32,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            diagnostics: Diagnostics::default(),
            scopes: vec![TypeScope {
                bindings: HashMap::new(),
            }],
            functions: HashMap::new(),
            current_return: None,
            in_loop: 0,
        }
    }

    fn finish(self) -> Diagnostics {
        self.diagnostics
    }

    fn push_scope(&mut self) {
        self.scopes.push(TypeScope {
            bindings: HashMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        debug_assert!(self.scopes.len() > 1);
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, ty: Ty, mutable: bool) {
        self.scopes
            .last_mut()
            .expect("scope")
            .bindings
            .insert(name.to_string(), LocalBinding { ty, mutable });
    }

    fn lookup(&self, name: &str) -> Option<LocalBinding> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.bindings.get(name) {
                return Some(*binding);
            }
        }
        None
    }

    fn lookup_ty(&self, name: &str) -> Option<Ty> {
        self.lookup(name).map(|b| b.ty)
    }

    fn emit(&mut self, diag: Diagnostic) {
        self.diagnostics.push(diag);
    }

    fn type_from_expr(&mut self, ty: &TypeExpr) -> Option<Ty> {
        match ty {
            TypeExpr::Named(n) => match n.value.as_str() {
                "Int" => Some(Ty::Int),
                "Bool" => Some(Ty::Bool),
                "None" => Some(Ty::None),
                "String" => Some(Ty::String),
                other => {
                    self.emit(unknown_type(n.span, other));
                    Some(Ty::Error)
                }
            },
        }
    }

    fn check_program(&mut self, program: &Program) {
        for item in &program.items {
            let Item::Fn(f) = item;
            self.register_fn(f);
        }
        for item in &program.items {
            self.check_item(item);
        }
    }

    fn register_fn(&mut self, f: &FnDecl) {
        let mut params = Vec::new();
        for p in &f.params {
            if let Some(ty) = self.type_from_expr(&p.ty) {
                params.push(ty);
            } else {
                params.push(Ty::Error);
            }
        }
        let ret = f
            .return_type
            .as_ref()
            .and_then(|t| self.type_from_expr(t))
            .unwrap_or(Ty::None);
        self.functions
            .insert(f.name.name.clone(), FnSig { params, ret });
    }

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => self.check_fn(f),
        }
    }

    fn check_fn(&mut self, f: &FnDecl) {
        let sig = self.functions.get(&f.name.name).cloned().unwrap_or(FnSig {
            params: vec![],
            ret: Ty::Error,
        });
        self.push_scope();
        for (param, ty) in f.params.iter().zip(sig.params.iter()) {
            self.declare(&param.name.name, *ty, false);
        }
        let saved_return = self.current_return;
        self.current_return = Some(sig.ret);
        let body_ty = match &f.body {
            FnBody::Expr(e) => self.check_expr(e),
            FnBody::Block(b) => self.check_block(b),
        };
        if !body_ty.is_error() && body_ty != sig.ret {
            self.emit(return_type_mismatch(f.span, sig.ret, body_ty));
        }
        self.current_return = saved_return;
        self.pop_scope();
    }

    fn check_block(&mut self, block: &Block) -> Ty {
        self.push_scope();
        let n = block.stmts.len();
        for (i, stmt) in block.stmts.iter().enumerate() {
            let is_value_tail =
                block.tail.is_none() && i + 1 == n && matches!(stmt, Stmt::If(_) | Stmt::Expr(_));
            if !is_value_tail {
                self.check_stmt(stmt);
            }
        }
        let ty = if let Some(tail) = &block.tail {
            self.check_expr(tail)
        } else if let Some(last) = block.stmts.last() {
            match last {
                Stmt::If(i) => self.check_if_value(i),
                Stmt::Expr(e) => self.check_expr(e),
                _ => Ty::None,
            }
        } else {
            Ty::None
        };
        self.pop_scope();
        ty
    }

    fn check_block_inner(&mut self, block: &Block) -> Ty {
        self.push_scope();
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
        let ty = if let Some(tail) = &block.tail {
            self.check_expr(tail)
        } else if let Some(last) = block.stmts.last() {
            match last {
                Stmt::If(i) => self.check_if_value(i),
                Stmt::Expr(e) => self.check_expr(e),
                _ => Ty::None,
            }
        } else {
            Ty::None
        };
        self.pop_scope();
        ty
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Binding(b) => self.check_binding(b),
            Stmt::Assign(a) => self.check_assign(a),
            Stmt::Expr(e) => {
                self.check_expr(e);
            }
            Stmt::If(i) => self.check_if_stmt(i),
            Stmt::Loop(l) => self.check_loop(l),
            Stmt::For(f) => self.check_for(f),
            Stmt::Return(r) => self.check_return(r),
            Stmt::Break(_) | Stmt::Next(_) => {}
        }
    }

    fn check_binding(&mut self, binding: &Binding) {
        let inferred = self.check_expr(&binding.value);
        let ty = if let Some(ann) = &binding.ty {
            let expected = self.type_from_expr(ann).unwrap_or(Ty::Error);
            if !inferred.is_error() && !expected.is_error() && inferred != expected {
                self.emit(binding_annotation_mismatch(
                    binding.span,
                    expected,
                    inferred,
                ));
            }
            if expected.is_error() {
                inferred
            } else {
                expected
            }
        } else {
            inferred
        };
        if !ty.is_error() {
            self.declare(&binding.name.name, ty, binding.mutable);
        }
    }

    fn check_assign(&mut self, assign: &Assign) {
        let value_ty = self.check_expr(&assign.value);
        if let Some(binding_ty) = self.lookup_ty(&assign.name.name) {
            if !value_ty.is_error() && value_ty != binding_ty {
                self.emit(assign_mismatch(assign.span, binding_ty, value_ty));
            }
        }
    }

    fn check_if_stmt(&mut self, if_stmt: &IfStmt) {
        let cond_ty = self.check_expr(&if_stmt.cond);
        if !cond_ty.is_error() && cond_ty != Ty::Bool {
            self.emit(condition_not_bool(if_stmt.cond.span(), cond_ty));
        }
        self.check_block_inner(&if_stmt.then_block);
        if let Some(else_block) = &if_stmt.else_block {
            self.check_block_inner(else_block);
        }
    }

    fn check_if_value(&mut self, if_stmt: &IfStmt) -> Ty {
        let cond_ty = self.check_expr(&if_stmt.cond);
        if !cond_ty.is_error() && cond_ty != Ty::Bool {
            self.emit(condition_not_bool(if_stmt.cond.span(), cond_ty));
        }
        let then_ty = self.check_block_inner(&if_stmt.then_block);
        let else_ty = if let Some(else_block) = &if_stmt.else_block {
            self.check_block_inner(else_block)
        } else {
            Ty::None
        };
        if !then_ty.is_error() && !else_ty.is_error() && then_ty != else_ty {
            self.emit(branch_mismatch(if_stmt.span, then_ty, else_ty));
            Ty::Error
        } else {
            Ty::unify(then_ty, else_ty)
        }
    }

    fn check_loop(&mut self, loop_stmt: &LoopStmt) {
        self.in_loop += 1;
        self.check_block_inner(&loop_stmt.body);
        self.in_loop -= 1;
    }

    /// Tiny stub: `for x in iter` requires `iter: Int`; pattern `x` is `Int`.
    fn check_for(&mut self, for_stmt: &ForStmt) {
        let iter_ty = self.check_expr(&for_stmt.iter);
        if !iter_ty.is_error() && iter_ty != Ty::Int {
            self.emit(for_iter_not_int(for_stmt.iter.span(), iter_ty));
        }
        self.push_scope();
        if let ForPattern::Ident(name) = &for_stmt.pattern {
            self.declare(&name.name, Ty::Int, false);
        }
        self.in_loop += 1;
        self.check_block_inner(&for_stmt.body);
        self.in_loop -= 1;
        self.pop_scope();
    }

    fn check_return(&mut self, ret: &ReturnStmt) {
        if let Some(value) = &ret.value {
            let ty = self.check_expr(value);
            if let Some(expected) = self.current_return {
                if !ty.is_error() && ty != expected {
                    self.emit(mismatch(ret.span, expected, ty));
                }
            }
        } else if let Some(expected) = self.current_return {
            if expected != Ty::None {
                self.emit(mismatch(ret.span, expected, Ty::None));
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Ty {
        match expr {
            Expr::IntLit(_) => Ty::Int,
            Expr::BoolLit(_) => Ty::Bool,
            Expr::StringLit(_) => Ty::String,
            Expr::Ident(sp) => self.lookup_ty(&sp.value).unwrap_or(Ty::Error),
            Expr::Unary(op, inner) => self.check_unary(op.value, op.span, inner),
            Expr::Binary(op, left, right) => self.check_binary(op.value, op.span, left, right),
            Expr::Call(name, args) => self.check_call(name.span, &name.value, args),
            Expr::Block(block) => self.check_block(block),
        }
    }

    fn check_unary(&mut self, op: UnaryOp, span: Span, inner: &Expr) -> Ty {
        let inner_ty = self.check_expr(inner);
        match op {
            UnaryOp::Neg => {
                if !inner_ty.is_error() && inner_ty != Ty::Int {
                    self.emit(mismatch(span, Ty::Int, inner_ty));
                    Ty::Error
                } else {
                    Ty::Int
                }
            }
            UnaryOp::Not => {
                if !inner_ty.is_error() && inner_ty != Ty::Bool {
                    self.emit(mismatch(span, Ty::Bool, inner_ty));
                    Ty::Error
                } else {
                    Ty::Bool
                }
            }
        }
    }

    fn check_binary(&mut self, op: BinaryOp, span: Span, left: &Expr, right: &Expr) -> Ty {
        let left_ty = self.check_expr(left);
        let right_ty = self.check_expr(right);
        if left_ty.is_error() || right_ty.is_error() {
            return Ty::Error;
        }
        let op_name = binary_op_name(op);
        match op {
            BinaryOp::Or | BinaryOp::And => {
                if left_ty != Ty::Bool || right_ty != Ty::Bool {
                    self.emit(binop_mismatch(span, op_name, left_ty, right_ty));
                    Ty::Error
                } else {
                    Ty::Bool
                }
            }
            BinaryOp::Eq | BinaryOp::NotEq => {
                if left_ty != right_ty {
                    self.emit(binop_mismatch(span, op_name, left_ty, right_ty));
                    Ty::Error
                } else if matches!(left_ty, Ty::Int | Ty::Bool | Ty::String) {
                    Ty::Bool
                } else {
                    self.emit(binop_mismatch(span, op_name, left_ty, right_ty));
                    Ty::Error
                }
            }
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                if left_ty != Ty::Int || right_ty != Ty::Int {
                    self.emit(binop_mismatch(span, op_name, left_ty, right_ty));
                    Ty::Error
                } else {
                    Ty::Bool
                }
            }
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::IntDiv
            | BinaryOp::Mod
            | BinaryOp::Pow => {
                if left_ty != Ty::Int || right_ty != Ty::Int {
                    self.emit(binop_mismatch(span, op_name, left_ty, right_ty));
                    Ty::Error
                } else {
                    Ty::Int
                }
            }
        }
    }

    fn check_call(&mut self, span: Span, name: &str, args: &[Expr]) -> Ty {
        if name == "println" {
            if args.len() != 1 {
                self.emit(wrong_arg_count(span, "println", 1, args.len()));
                return Ty::Error;
            }
            let arg_ty = self.check_expr(&args[0]);
            if !arg_ty.is_error() && arg_ty != Ty::String {
                self.emit(println_arg_mismatch(args[0].span(), arg_ty));
            }
            return Ty::None;
        }
        if name == "assert_eq" || name == "assert_eq!" {
            if args.len() != 2 {
                self.emit(wrong_arg_count(span, name, 2, args.len()));
                return Ty::Error;
            }
            let left = self.check_expr(&args[0]);
            let right = self.check_expr(&args[1]);
            if !left.is_error() && !right.is_error() && left != right {
                self.emit(mismatch(args[1].span(), left, right));
            }
            return Ty::None;
        }
        if name == "borrow" {
            if args.len() != 1 {
                self.emit(wrong_arg_count(span, "borrow", 1, args.len()));
                return Ty::Error;
            }
            let arg_ty = self.check_expr(&args[0]);
            if !arg_ty.is_error() && arg_ty.is_copy() {
                self.emit(cannot_borrow_copy(args[0].span(), arg_ty));
                return Ty::Error;
            }
            return Ty::BorrowString;
        }
        if name == "claim" {
            if args.len() != 1 {
                self.emit(wrong_arg_count(span, "claim", 1, args.len()));
                return Ty::Error;
            }
            if let Expr::Ident(id) = &args[0] {
                if let Some(binding) = self.lookup(&id.value) {
                    if !binding.mutable {
                        self.emit(claim_requires_mutable(id.span, &id.value));
                    }
                }
            }
            let arg_ty = self.check_expr(&args[0]);
            if !arg_ty.is_error() && arg_ty.is_copy() {
                self.emit(cannot_borrow_copy(args[0].span(), arg_ty));
                return Ty::Error;
            }
            return Ty::ClaimString;
        }
        if name == "deref" {
            if args.len() != 1 {
                self.emit(wrong_arg_count(span, "deref", 1, args.len()));
                return Ty::Error;
            }
            let arg_ty = self.check_expr(&args[0]);
            if arg_ty.is_error() {
                return Ty::Error;
            }
            if arg_ty.pointee().is_some() {
                return Ty::String;
            }
            self.emit(deref_requires_borrow(args[0].span(), arg_ty));
            return Ty::Error;
        }
        let Some(sig) = self.functions.get(name).cloned() else {
            self.emit(unknown_function(span, name));
            return Ty::Error;
        };
        if args.len() != sig.params.len() {
            self.emit(wrong_arg_count(span, name, sig.params.len(), args.len()));
            return Ty::Error;
        }
        for (arg, expected) in args.iter().zip(sig.params.iter()) {
            let arg_ty = self.check_expr(arg);
            if !arg_ty.is_error() && arg_ty != *expected {
                self.emit(mismatch(arg.span(), *expected, arg_ty));
            }
        }
        sig.ret
    }
}

fn binary_op_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Or => "||",
        BinaryOp::And => "&&",
        BinaryOp::Eq => "==",
        BinaryOp::NotEq => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::Le => "<=",
        BinaryOp::Ge => ">=",
        BinaryOp::Add => "add",
        BinaryOp::Sub => "subtract",
        BinaryOp::Mul => "multiply",
        BinaryOp::Div => "divide",
        BinaryOp::IntDiv => "int-divide",
        BinaryOp::Mod => "mod",
        BinaryOp::Pow => "pow",
    }
}

trait ExprSpan {
    fn span(&self) -> Span;
}

impl ExprSpan for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::IntLit(s) => s.span,
            Expr::BoolLit(s) => s.span,
            Expr::StringLit(s) => s.span,
            Expr::Ident(s) => s.span,
            Expr::Unary(op, _) => op.span,
            Expr::Binary(op, _, _) => op.span,
            Expr::Call(name, _) => name.span,
            Expr::Block(b) => b.span,
        }
    }
}

/// Type-check a resolved program.
pub fn check_program(program: &Program) -> Diagnostics {
    let mut checker = TypeChecker::new();
    checker.check_program(program);
    checker.finish()
}
