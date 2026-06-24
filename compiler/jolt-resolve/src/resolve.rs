use jolt_ast::{
    Assign, Binding, Block, Expr, FnBody, FnDecl, ForPattern, ForStmt, Ident, IfStmt, Item,
    LoopStmt, Program, Stmt,
};
use jolt_source::Span;

use crate::error::{ResolveError, ResolveErrorKind};
use crate::scope::{BindingInfo, BindingOrigin, ScopeStack};

struct Resolver {
    scopes: ScopeStack,
    errors: Vec<ResolveError>,
}

impl Resolver {
    fn new() -> Self {
        Self {
            scopes: ScopeStack::new(),
            errors: Vec::new(),
        }
    }

    fn finish(self, _program: Program) -> (Vec<ResolveError>, Vec<BindingInfo>) {
        (self.errors, self.scopes.symbols)
    }

    fn error(&mut self, kind: ResolveErrorKind, span: Span, message: impl Into<String>) {
        self.errors
            .push(ResolveError::new(kind, span, message.into()));
    }

    fn resolve_program(&mut self, program: &Program) {
        for item in &program.items {
            self.resolve_item(item);
        }
    }

    fn resolve_item(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => self.resolve_fn(f),
        }
    }

    fn resolve_fn(&mut self, f: &FnDecl) {
        self.scopes.push_scope();
        for param in &f.params {
            self.declare_param(&param.name);
        }
        match &f.body {
            FnBody::Expr(expr) => self.resolve_expr(expr),
            FnBody::Block(block) => self.resolve_block(block),
        }
        self.scopes.pop_scope();
    }

    fn declare_param(&mut self, name: &Ident) {
        if self.scopes.contains_in_current(&name.name) {
            self.error(
                ResolveErrorKind::DuplicateBinding,
                name.span,
                format!("duplicate binding '{}' in this scope", name.name),
            );
            return;
        }
        self.scopes
            .declare(name.name.clone(), false, name.span, BindingOrigin::Param);
    }

    fn declare_binding(&mut self, binding: &Binding) {
        if self.scopes.contains_in_current(&binding.name.name) {
            self.error(
                ResolveErrorKind::DuplicateBinding,
                binding.span,
                format!("duplicate binding '{}' in this scope", binding.name.name),
            );
            return;
        }
        self.resolve_expr(&binding.value);
        let origin = if binding.mutable {
            BindingOrigin::MutableBinding
        } else {
            BindingOrigin::ImmutableBinding
        };
        self.scopes.declare(
            binding.name.name.clone(),
            binding.mutable,
            binding.name.span,
            origin,
        );
    }

    fn resolve_assign(&mut self, assign: &Assign) {
        self.resolve_expr(&assign.value);
        let Some(info) = self.scopes.lookup(&assign.name.name) else {
            self.error(
                ResolveErrorKind::UndefinedName,
                assign.name.span,
                format!("undefined name '{}'", assign.name.name),
            );
            return;
        };
        if !info.mutable {
            let message = match info.origin {
                BindingOrigin::Param => {
                    format!("cannot reassign parameter '{}'", assign.name.name)
                }
                BindingOrigin::ForLoop => {
                    format!("cannot reassign for-loop binding '{}'", assign.name.name)
                }
                BindingOrigin::ImmutableBinding => format!(
                    "cannot reassign immutable binding '{}' (declared with $)",
                    assign.name.name
                ),
                BindingOrigin::MutableBinding => {
                    format!("cannot reassign binding '{}'", assign.name.name)
                }
            };
            let kind = match info.origin {
                BindingOrigin::MutableBinding => ResolveErrorKind::InvalidReassign,
                _ => ResolveErrorKind::ImmutableAssign,
            };
            self.error(kind, assign.span, message);
        }
    }

    fn resolve_ident(&mut self, span: Span, name: &str) {
        if self.scopes.lookup(name).is_none() {
            self.error(
                ResolveErrorKind::UndefinedName,
                span,
                format!("undefined name '{name}'"),
            );
        }
    }

    fn resolve_block(&mut self, block: &Block) {
        self.scopes.push_scope();
        self.resolve_block_contents(block);
        self.scopes.pop_scope();
    }

    fn resolve_block_contents(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.resolve_stmt(stmt);
        }
        if let Some(tail) = &block.tail {
            self.resolve_expr(tail);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Binding(b) => self.declare_binding(b),
            Stmt::Assign(a) => self.resolve_assign(a),
            Stmt::Expr(e) => self.resolve_expr(e),
            Stmt::If(i) => self.resolve_if(i),
            Stmt::Loop(l) => self.resolve_loop(l),
            Stmt::For(f) => self.resolve_for(f),
            Stmt::Return(r) => {
                if let Some(v) = &r.value {
                    self.resolve_expr(v);
                }
            }
            Stmt::Break(_) | Stmt::Next(_) => {}
        }
    }

    fn resolve_if(&mut self, if_stmt: &IfStmt) {
        self.resolve_expr(&if_stmt.cond);
        self.resolve_block(&if_stmt.then_block);
        if let Some(else_block) = &if_stmt.else_block {
            self.resolve_block(else_block);
        }
    }

    fn resolve_loop(&mut self, loop_stmt: &LoopStmt) {
        self.resolve_block(&loop_stmt.body);
    }

    fn resolve_for(&mut self, for_stmt: &ForStmt) {
        self.resolve_expr(&for_stmt.iter);
        self.scopes.push_scope();
        match &for_stmt.pattern {
            ForPattern::Ident(name) => {
                if self.scopes.contains_in_current(&name.name) {
                    self.error(
                        ResolveErrorKind::DuplicateBinding,
                        name.span,
                        format!("duplicate binding '{}' in this scope", name.name),
                    );
                } else {
                    self.scopes.declare(
                        name.name.clone(),
                        false,
                        name.span,
                        BindingOrigin::ForLoop,
                    );
                }
            }
            ForPattern::Wildcard(_) => {}
        }
        self.resolve_block_contents(&for_stmt.body);
        self.scopes.pop_scope();
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::IntLit(_) | Expr::BoolLit(_) | Expr::StringLit(_) => {}
            Expr::Ident(sp) => self.resolve_ident(sp.span, &sp.value),
            Expr::Unary(_, inner) => self.resolve_expr(inner),
            Expr::Binary(_, left, right) => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Call(_, args) => {
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            Expr::Block(block) => self.resolve_block(block),
        }
    }
}

/// Resolve names in a parsed program.
pub fn resolve_program(program: &Program) -> super::ResolveResult {
    let mut resolver = Resolver::new();
    resolver.resolve_program(program);
    let (errors, symbols) = resolver.finish(program.clone());
    super::ResolveResult {
        program: program.clone(),
        errors,
        symbols,
    }
}
