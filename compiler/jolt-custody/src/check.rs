use std::collections::HashMap;

use jolt_ast::{
    Assign, BinaryOp, Binding, Block, Expr, FnBody, FnDecl, ForPattern, ForStmt, IfStmt, Item,
    LoopStmt, Program, ReturnStmt, Stmt, TypeExpr,
};
use jolt_diagnostics::Diagnostics;
use jolt_source::Span;
use jolt_types::Ty;

use crate::error::{
    claim_while_borrowed, shared_while_claimed, use_after_move, use_while_borrowed,
};
use crate::liveness;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OwnerState {
    Valid,
    Moved,
    Borrowed { shared: u32, claimed: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BorrowKind {
    Shared,
    Claim,
}

#[derive(Debug, Clone)]
struct VarInfo {
    ty: Ty,
    owner: OwnerState,
    /// When this binding is a borrow handle, the source owner name.
    borrow_source: Option<String>,
    borrow_kind: Option<BorrowKind>,
    mutable: bool,
}

#[derive(Clone)]
struct FnSig {
    params: Vec<Ty>,
}

struct Custodian {
    diagnostics: Diagnostics,
    scopes: Vec<HashMap<String, VarInfo>>,
    functions: HashMap<String, FnSig>,
}

impl Custodian {
    fn new() -> Self {
        Self {
            diagnostics: Diagnostics::default(),
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
        }
    }

    fn finish(self) -> Diagnostics {
        self.diagnostics
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        let scope = self.scopes.pop().expect("scope");
        for (_, info) in scope {
            if let (Some(source), Some(kind)) = (&info.borrow_source, info.borrow_kind) {
                self.release_borrow(source, kind);
            }
        }
    }

    fn declare(
        &mut self,
        name: &str,
        ty: Ty,
        mutable: bool,
        borrow_source: Option<String>,
        borrow_kind: Option<BorrowKind>,
    ) {
        self.scopes.last_mut().expect("scope").insert(
            name.to_string(),
            VarInfo {
                ty,
                owner: OwnerState::Valid,
                borrow_source,
                borrow_kind,
                mutable,
            },
        );
    }

    fn lookup_ty(&self, name: &str) -> Option<Ty> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info.ty);
            }
        }
        None
    }

    fn lookup_mutable(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return info.mutable;
            }
        }
        false
    }

    fn owner_state_mut(&mut self, name: &str) -> Option<&mut OwnerState> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                if info.borrow_source.is_none() {
                    return Some(&mut info.owner);
                }
            }
        }
        None
    }

    fn use_owned(&mut self, name: &str, span: Span) {
        let is_copy = self.lookup_ty(name).is_some_and(|ty| ty.is_copy());
        let Some(state) = self.owner_state_mut(name) else {
            return;
        };
        match *state {
            OwnerState::Moved => {
                self.diagnostics.push(use_after_move(span, name));
            }
            OwnerState::Borrowed { .. } => {
                self.diagnostics.push(use_while_borrowed(span, name));
            }
            OwnerState::Valid => {
                if !is_copy {
                    *state = OwnerState::Moved;
                }
            }
        }
    }

    fn add_shared_borrow(&mut self, name: &str, span: Span) {
        let Some(state) = self.owner_state_mut(name) else {
            return;
        };
        match *state {
            OwnerState::Moved => self.diagnostics.push(use_after_move(span, name)),
            OwnerState::Borrowed { claimed: true, .. } => {
                self.diagnostics.push(shared_while_claimed(span, name));
            }
            OwnerState::Borrowed {
                shared,
                claimed: false,
            } => {
                *state = OwnerState::Borrowed {
                    shared: shared + 1,
                    claimed: false,
                };
            }
            OwnerState::Valid => {
                *state = OwnerState::Borrowed {
                    shared: 1,
                    claimed: false,
                };
            }
        }
    }

    fn add_claim(&mut self, name: &str, span: Span) {
        let Some(state) = self.owner_state_mut(name) else {
            return;
        };
        match *state {
            OwnerState::Moved => self.diagnostics.push(use_after_move(span, name)),
            OwnerState::Borrowed { .. } => {
                self.diagnostics.push(claim_while_borrowed(span, name));
            }
            OwnerState::Valid => {
                *state = OwnerState::Borrowed {
                    shared: 0,
                    claimed: true,
                };
            }
        }
    }

    fn release_borrow(&mut self, source: &str, kind: BorrowKind) {
        let Some(state) = self.owner_state_mut(source) else {
            return;
        };
        match *state {
            OwnerState::Borrowed {
                shared: _,
                claimed: true,
            } if kind == BorrowKind::Claim => {
                *state = OwnerState::Valid;
            }
            OwnerState::Borrowed {
                shared: 1,
                claimed: false,
            } if kind == BorrowKind::Shared => {
                *state = OwnerState::Valid;
            }
            OwnerState::Borrowed {
                shared,
                claimed: false,
            } if kind == BorrowKind::Shared && shared > 1 => {
                *state = OwnerState::Borrowed {
                    shared: shared - 1,
                    claimed: false,
                };
            }
            _ => {}
        }
    }

    fn expire_handles(&mut self, stmt_idx: usize, last_uses: &HashMap<String, usize>) {
        let names: Vec<String> = self
            .scopes
            .last()
            .map(|scope| {
                scope
                    .iter()
                    .filter(|(name, info)| {
                        info.borrow_source.is_some() && last_uses.get(*name) == Some(&stmt_idx)
                    })
                    .map(|(name, _)| name.clone())
                    .collect()
            })
            .unwrap_or_default();
        for name in names {
            let (source, kind) = {
                let scope = self.scopes.last().expect("scope");
                let info = scope.get(&name).expect("handle");
                (
                    info.borrow_source.clone().expect("source"),
                    info.borrow_kind.expect("kind"),
                )
            };
            self.release_borrow(&source, kind);
        }
    }

    fn type_from_expr(&self, ty: &TypeExpr) -> Option<Ty> {
        match ty {
            TypeExpr::Named(n) => match n.value.as_str() {
                "Int" => Some(Ty::Int),
                "Bool" => Some(Ty::Bool),
                "None" => Some(Ty::None),
                "String" => Some(Ty::String),
                _ => None,
            },
        }
    }

    fn register_fn(&mut self, f: &FnDecl) {
        let params = f
            .params
            .iter()
            .filter_map(|p| self.type_from_expr(&p.ty))
            .collect();
        self.functions.insert(f.name.name.clone(), FnSig { params });
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

    fn check_item(&mut self, item: &Item) {
        let Item::Fn(f) = item;
        self.check_fn(f);
    }

    fn check_fn(&mut self, f: &FnDecl) {
        let sig = self
            .functions
            .get(&f.name.name)
            .cloned()
            .unwrap_or(FnSig { params: vec![] });
        self.push_scope();
        for (param, ty) in f.params.iter().zip(sig.params.iter()) {
            self.declare(&param.name.name, *ty, false, None, None);
        }
        match &f.body {
            FnBody::Expr(e) => {
                self.check_expr(e);
            }
            FnBody::Block(b) => self.check_block(b),
        }
        self.pop_scope();
    }

    fn check_block(&mut self, block: &Block) {
        self.push_scope();
        let last_uses = liveness::block_last_uses(block);
        let n = block.stmts.len();
        for (i, stmt) in block.stmts.iter().enumerate() {
            let is_value_tail =
                block.tail.is_none() && i + 1 == n && matches!(stmt, Stmt::If(_) | Stmt::Expr(_));
            if !is_value_tail {
                self.check_stmt(stmt);
                self.expire_handles(i, &last_uses);
            }
        }
        if let Some(tail) = &block.tail {
            self.check_expr(tail);
            self.expire_handles(n, &last_uses);
        } else if let Some(last) = block.stmts.last() {
            match last {
                Stmt::If(i) => {
                    self.check_if_value(i);
                    self.expire_handles(n.saturating_sub(1), &last_uses);
                }
                Stmt::Expr(e) => {
                    self.check_expr(e);
                    self.expire_handles(n.saturating_sub(1), &last_uses);
                }
                _ => {}
            }
        }
        self.pop_scope();
    }

    fn check_block_inner(&mut self, block: &Block) {
        self.check_block(block);
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
        let (ty, borrow_source, borrow_kind) = self.check_init_for_binding(&binding.value);
        let ty = binding
            .ty
            .as_ref()
            .and_then(|t| self.type_from_expr(t))
            .unwrap_or(ty);
        if !ty.is_error() {
            self.declare(
                &binding.name.name,
                ty,
                binding.mutable,
                borrow_source,
                borrow_kind,
            );
        }
    }

    fn check_init_for_binding(&mut self, expr: &Expr) -> (Ty, Option<String>, Option<BorrowKind>) {
        if let Expr::Call(callee, args) = expr {
            if callee.value == "borrow" {
                if let Some(Expr::Ident(id)) = args.first() {
                    self.add_shared_borrow(&id.value, id.span);
                    return (
                        Ty::BorrowString,
                        Some(id.value.clone()),
                        Some(BorrowKind::Shared),
                    );
                }
            }
            if callee.value == "claim" {
                if let Some(Expr::Ident(id)) = args.first() {
                    self.add_claim(&id.value, id.span);
                    return (
                        Ty::ClaimString,
                        Some(id.value.clone()),
                        Some(BorrowKind::Claim),
                    );
                }
            }
        }
        (self.check_expr(expr), None, None)
    }

    fn check_assign(&mut self, assign: &Assign) {
        self.check_expr(&assign.value);
    }

    fn check_if_stmt(&mut self, if_stmt: &IfStmt) {
        self.check_expr(&if_stmt.cond);
        self.check_block_inner(&if_stmt.then_block);
        if let Some(else_block) = &if_stmt.else_block {
            self.check_block_inner(else_block);
        }
    }

    fn check_if_value(&mut self, if_stmt: &IfStmt) {
        self.check_if_stmt(if_stmt);
    }

    fn check_loop(&mut self, loop_stmt: &LoopStmt) {
        self.check_block_inner(&loop_stmt.body);
    }

    fn check_for(&mut self, for_stmt: &ForStmt) {
        self.check_expr(&for_stmt.iter);
        self.push_scope();
        match &for_stmt.pattern {
            ForPattern::Ident(id) => self.declare(&id.name, Ty::Int, false, None, None),
            ForPattern::Wildcard(_) => {}
        }
        self.check_block_inner(&for_stmt.body);
        self.pop_scope();
    }

    fn check_return(&mut self, ret: &ReturnStmt) {
        if let Some(v) = &ret.value {
            self.check_expr(v);
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Ty {
        match expr {
            Expr::IntLit(_) => Ty::Int,
            Expr::BoolLit(_) => Ty::Bool,
            Expr::StringLit(_) => Ty::String,
            Expr::Ident(id) => {
                if let Some(ty) = self.lookup_ty(&id.value) {
                    if ty.is_borrow_handle() {
                        // Borrow handles are copy; no owner state change.
                    } else if !ty.is_copy() {
                        self.use_owned(&id.value, id.span);
                    }
                }
                self.lookup_ty(&id.value).unwrap_or(Ty::Error)
            }
            Expr::Unary(_, inner) => self.check_expr(inner),
            Expr::Binary(op, left, right) => {
                self.check_expr(left);
                self.check_expr(right);
                match op.value {
                    BinaryOp::Or | BinaryOp::And => Ty::Bool,
                    BinaryOp::Eq
                    | BinaryOp::NotEq
                    | BinaryOp::Lt
                    | BinaryOp::Gt
                    | BinaryOp::Le
                    | BinaryOp::Ge => Ty::Bool,
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::IntDiv
                    | BinaryOp::Mod
                    | BinaryOp::Pow => Ty::Int,
                }
            }
            Expr::Call(callee, args) => self.check_call(callee.span, &callee.value, args),
            Expr::Block(b) => {
                self.check_block(b);
                Ty::None
            }
        }
    }

    fn check_call(&mut self, span: Span, name: &str, args: &[Expr]) -> Ty {
        if name == "println" {
            if let Some(arg) = args.first() {
                self.check_expr(arg);
            }
            return Ty::None;
        }
        if name == "borrow" {
            if let Some(Expr::Ident(id)) = args.first() {
                self.add_shared_borrow(&id.value, id.span);
            }
            return Ty::BorrowString;
        }
        if name == "claim" {
            if let Some(Expr::Ident(id)) = args.first() {
                let _ = self.lookup_mutable(&id.value);
                self.add_claim(&id.value, id.span);
            }
            return Ty::ClaimString;
        }
        if name == "deref" {
            if let Some(arg) = args.first() {
                self.check_expr(arg);
            }
            return Ty::String;
        }
        if let Some(sig) = self.functions.get(name).cloned() {
            for (arg, _) in args.iter().zip(sig.params.iter()) {
                self.check_expr(arg);
            }
            for arg in args.iter().skip(sig.params.len()) {
                self.check_expr(arg);
            }
            return Ty::None;
        }
        for arg in args {
            self.check_expr(arg);
        }
        let _ = span;
        Ty::Error
    }
}

pub fn check_program(program: &Program) -> Diagnostics {
    let mut c = Custodian::new();
    c.check_program(program);
    c.finish()
}
