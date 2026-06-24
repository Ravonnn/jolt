use jolt_ast::{
    BinaryOp, Block, Expr, FnBody, FnDecl, ForPattern, ForStmt, IfStmt, Item, LoopStmt, Program,
    Stmt,
};
use jolt_diagnostics::{Diagnostic, Diagnostics};
use jolt_source::Span;

use crate::mir::{LocalId, MirFn, MirInstr, MirModule};

pub const ENTRY_FN: &str = "main";

pub fn lower_program(program: &Program) -> (MirModule, Diagnostics) {
    let mut diagnostics = Diagnostics::default();
    let mut functions = Vec::new();

    for item in &program.items {
        let Item::Fn(f) = item;
        if let Some(mir_fn) = lower_fn(f, &mut diagnostics) {
            functions.push(mir_fn);
        }
    }

    (
        MirModule {
            entry: ENTRY_FN.to_string(),
            functions,
        },
        diagnostics,
    )
}

struct LoopFrame {
    head: u32,
    break_patches: Vec<usize>,
    next_patches: Vec<usize>,
}

struct LowerCtx<'a> {
    diagnostics: &'a mut Diagnostics,
    locals: std::collections::HashMap<String, LocalId>,
    next_local: LocalId,
    body: Vec<MirInstr>,
    loop_stack: Vec<LoopFrame>,
}

impl LowerCtx<'_> {
    fn alloc_local(&mut self, name: &str) -> LocalId {
        let id = self.next_local;
        self.next_local += 1;
        self.locals.insert(name.to_string(), id);
        id
    }

    fn fresh_local(&mut self) -> LocalId {
        let id = self.next_local;
        self.next_local += 1;
        id
    }

    fn lookup(&self, name: &str) -> Option<LocalId> {
        self.locals.get(name).copied()
    }

    fn emit(&mut self, instr: MirInstr) {
        self.body.push(instr);
    }

    fn pc(&self) -> u32 {
        self.body.len() as u32
    }

    fn error(&mut self, span: Span, msg: impl Into<String>) {
        self.diagnostics.push(Diagnostic::error(span, msg));
    }

    fn patch_jump(&mut self, idx: usize, target: u32) {
        if let MirInstr::Jump { target: t } = &mut self.body[idx] {
            *t = target;
        }
    }

    fn patch_branch(&mut self, idx: usize, then_pc: u32, else_pc: u32) {
        if let MirInstr::BranchIf {
            then_pc: t,
            else_pc: e,
            ..
        } = &mut self.body[idx]
        {
            *t = then_pc;
            *e = else_pc;
        }
    }
}

fn lower_fn(f: &FnDecl, diagnostics: &mut Diagnostics) -> Option<MirFn> {
    let mut ctx = LowerCtx {
        diagnostics,
        locals: std::collections::HashMap::new(),
        next_local: 0,
        body: Vec::new(),
        loop_stack: Vec::new(),
    };

    for param in &f.params {
        ctx.alloc_local(&param.name.name);
    }
    let param_count = f.params.len();

    match &f.body {
        FnBody::Expr(e) => {
            let v = lower_expr(&mut ctx, e);
            ctx.emit(MirInstr::Return { value: v });
        }
        FnBody::Block(b) => lower_block(&mut ctx, b, true),
    }

    if !ctx.diagnostics.is_empty() {
        return None;
    }

    Some(MirFn {
        name: f.name.name.clone(),
        param_count,
        local_count: ctx.next_local as usize,
        body: ctx.body,
    })
}

fn lower_block(ctx: &mut LowerCtx<'_>, block: &Block, is_fn_body: bool) {
    let n = block.stmts.len();
    for (i, stmt) in block.stmts.iter().enumerate() {
        let is_value_tail =
            block.tail.is_none() && i + 1 == n && matches!(stmt, Stmt::If(_) | Stmt::Expr(_));
        if !is_value_tail {
            lower_stmt(ctx, stmt);
        }
    }

    if let Some(tail) = &block.tail {
        let v = lower_expr(ctx, tail);
        if is_fn_body {
            ctx.emit(MirInstr::Return { value: v });
        }
    } else if let Some(last) = block.stmts.last() {
        let is_value_tail = block.tail.is_none() && matches!(last, Stmt::If(_) | Stmt::Expr(_));
        match last {
            Stmt::Expr(e) if is_value_tail => {
                let v = lower_expr(ctx, e);
                if is_fn_body {
                    ctx.emit(MirInstr::Return { value: v });
                }
            }
            Stmt::If(i) if is_value_tail => {
                let dest = ctx.fresh_local();
                lower_if(ctx, i, Some(dest));
                if is_fn_body {
                    ctx.emit(MirInstr::Return { value: Some(dest) });
                }
            }
            _ => {
                if is_fn_body {
                    ctx.emit(MirInstr::Return { value: None });
                }
            }
        }
    } else if is_fn_body {
        ctx.emit(MirInstr::Return { value: None });
    }
}

fn lower_block_value(ctx: &mut LowerCtx<'_>, block: &Block) -> Option<LocalId> {
    let n = block.stmts.len();
    for (i, stmt) in block.stmts.iter().enumerate() {
        let is_value_tail =
            block.tail.is_none() && i + 1 == n && matches!(stmt, Stmt::If(_) | Stmt::Expr(_));
        if !is_value_tail {
            lower_stmt(ctx, stmt);
        }
    }

    if let Some(tail) = &block.tail {
        lower_expr(ctx, tail)
    } else if let Some(last) = block.stmts.last() {
        let is_value_tail = block.tail.is_none() && matches!(last, Stmt::If(_) | Stmt::Expr(_));
        match last {
            Stmt::Expr(e) if is_value_tail => lower_expr(ctx, e),
            Stmt::If(i) if is_value_tail => {
                let dest = ctx.fresh_local();
                lower_if(ctx, i, Some(dest));
                Some(dest)
            }
            _ => None,
        }
    } else {
        None
    }
}

fn lower_if(ctx: &mut LowerCtx<'_>, if_stmt: &IfStmt, result: Option<LocalId>) {
    let cond = match lower_expr(ctx, &if_stmt.cond) {
        Some(c) => c,
        None => return,
    };
    let branch_idx = ctx.body.len();
    ctx.emit(MirInstr::BranchIf {
        cond,
        then_pc: 0,
        else_pc: 0,
    });

    let then_pc = ctx.pc();
    if let Some(dest) = result {
        if let Some(v) = lower_block_value(ctx, &if_stmt.then_block) {
            ctx.emit(MirInstr::CopyLocal { dest, src: v });
        }
    } else {
        lower_block(ctx, &if_stmt.then_block, false);
    }
    let jump_merge = ctx.body.len();
    ctx.emit(MirInstr::Jump { target: 0 });

    let else_pc = if let Some(else_block) = &if_stmt.else_block {
        let pc = ctx.pc();
        if let Some(dest) = result {
            if let Some(v) = lower_block_value(ctx, else_block) {
                ctx.emit(MirInstr::CopyLocal { dest, src: v });
            }
        } else {
            lower_block(ctx, else_block, false);
        }
        Some(pc)
    } else {
        None
    };

    let merge_pc = ctx.pc();
    ctx.patch_jump(jump_merge, merge_pc);
    let else_target = else_pc.unwrap_or(merge_pc);
    ctx.patch_branch(branch_idx, then_pc, else_target);
}

fn lower_loop(ctx: &mut LowerCtx<'_>, loop_stmt: &LoopStmt) {
    let head_pc = ctx.pc();
    ctx.loop_stack.push(LoopFrame {
        head: head_pc,
        break_patches: Vec::new(),
        next_patches: Vec::new(),
    });
    lower_block(ctx, &loop_stmt.body, false);
    let mut frame = ctx.loop_stack.pop().expect("loop frame");
    ctx.emit(MirInstr::Jump { target: head_pc });
    let end_pc = ctx.pc();
    for idx in frame.break_patches.drain(..) {
        ctx.patch_jump(idx, end_pc);
    }
    for idx in frame.next_patches.drain(..) {
        ctx.patch_jump(idx, frame.head);
    }
}

fn lower_for(ctx: &mut LowerCtx<'_>, for_stmt: &ForStmt) {
    let iter_local = match lower_expr(ctx, &for_stmt.iter) {
        Some(l) => l,
        None => return,
    };

    let x = match &for_stmt.pattern {
        ForPattern::Ident(id) => ctx.alloc_local(&id.name),
        ForPattern::Wildcard(span) => {
            let id = ctx.fresh_local();
            if ctx.diagnostics.is_empty() {
                // bind wildcard to anonymous slot
            }
            let _ = span;
            id
        }
    };

    let zero = emit_const_int(ctx, 0);
    ctx.emit(MirInstr::CopyLocal { dest: x, src: zero });

    let head_pc = ctx.pc();
    ctx.loop_stack.push(LoopFrame {
        head: head_pc,
        break_patches: Vec::new(),
        next_patches: Vec::new(),
    });

    let ge = emit_cmp_call(ctx, "__ge", x, iter_local);
    let branch_idx = ctx.body.len();
    ctx.emit(MirInstr::BranchIf {
        cond: ge,
        then_pc: 0,
        else_pc: 0,
    });

    let body_pc = ctx.pc();
    lower_block(ctx, &for_stmt.body, false);
    emit_increment(ctx, x);
    ctx.emit(MirInstr::Jump { target: head_pc });

    let end_pc = ctx.pc();
    let mut frame = ctx.loop_stack.pop().expect("for frame");
    ctx.patch_branch(branch_idx, end_pc, body_pc);
    for idx in frame.break_patches.drain(..) {
        ctx.patch_jump(idx, end_pc);
    }
    for idx in frame.next_patches.drain(..) {
        ctx.patch_jump(idx, frame.head);
    }
}

fn lower_stmt(ctx: &mut LowerCtx<'_>, stmt: &Stmt) {
    match stmt {
        Stmt::Binding(b) => {
            let dest = ctx.alloc_local(&b.name.name);
            if let Some(src) = lower_expr(ctx, &b.value) {
                ctx.emit(MirInstr::CopyLocal { dest, src });
            }
        }
        Stmt::Assign(a) => {
            if let Some(dest) = ctx.lookup(&a.name.name) {
                if let Some(src) = lower_expr(ctx, &a.value) {
                    ctx.emit(MirInstr::CopyLocal { dest, src });
                }
            } else {
                ctx.error(a.span, format!("unknown binding `{}`", a.name.name));
            }
        }
        Stmt::Expr(e) => {
            lower_expr(ctx, e);
        }
        Stmt::Return(r) => {
            let v = r.value.as_ref().and_then(|e| lower_expr(ctx, e));
            ctx.emit(MirInstr::Return { value: v });
        }
        Stmt::If(i) => lower_if(ctx, i, None),
        Stmt::Loop(l) => lower_loop(ctx, l),
        Stmt::For(f) => lower_for(ctx, f),
        Stmt::Break(b) => {
            let idx = ctx.body.len();
            ctx.emit(MirInstr::Jump { target: 0 });
            if let Some(frame) = ctx.loop_stack.last_mut() {
                frame.break_patches.push(idx);
            } else {
                ctx.error(b.span, "`break` outside loop");
            }
        }
        Stmt::Next(n) => {
            let idx = ctx.body.len();
            ctx.emit(MirInstr::Jump { target: 0 });
            if let Some(frame) = ctx.loop_stack.last_mut() {
                frame.next_patches.push(idx);
            } else {
                ctx.error(n.span, "`next` outside loop");
            }
        }
    }
}

fn emit_const_int(ctx: &mut LowerCtx<'_>, value: i64) -> LocalId {
    let dest = ctx.fresh_local();
    ctx.emit(MirInstr::ConstInt { dest, value });
    dest
}

fn emit_cmp_call(ctx: &mut LowerCtx<'_>, callee: &str, left: LocalId, right: LocalId) -> LocalId {
    let dest = ctx.fresh_local();
    ctx.emit(MirInstr::Call {
        dest: Some(dest),
        callee: callee.to_string(),
        args: vec![left, right],
    });
    dest
}

fn emit_increment(ctx: &mut LowerCtx<'_>, local: LocalId) {
    let one = emit_const_int(ctx, 1);
    let sum = ctx.fresh_local();
    ctx.emit(MirInstr::Call {
        dest: Some(sum),
        callee: "__add".to_string(),
        args: vec![local, one],
    });
    ctx.emit(MirInstr::CopyLocal {
        dest: local,
        src: sum,
    });
}

fn lower_expr(ctx: &mut LowerCtx<'_>, expr: &Expr) -> Option<LocalId> {
    match expr {
        Expr::IntLit(n) => {
            let dest = ctx.fresh_local();
            ctx.emit(MirInstr::ConstInt {
                dest,
                value: n.value,
            });
            Some(dest)
        }
        Expr::BoolLit(b) => {
            let dest = ctx.fresh_local();
            ctx.emit(MirInstr::ConstBool {
                dest,
                value: b.value,
            });
            Some(dest)
        }
        Expr::StringLit(s) => {
            let dest = ctx.fresh_local();
            ctx.emit(MirInstr::ConstString {
                dest,
                value: s.value.clone(),
            });
            Some(dest)
        }
        Expr::Ident(id) => {
            if let Some(src) = ctx.lookup(&id.value) {
                let dest = ctx.fresh_local();
                ctx.emit(MirInstr::CopyLocal { dest, src });
                Some(dest)
            } else {
                ctx.error(id.span, format!("unknown binding `{}`", id.value));
                None
            }
        }
        Expr::Unary(op, inner) => {
            let inner_id = lower_expr(ctx, inner)?;
            let dest = ctx.fresh_local();
            let callee = match op.value {
                jolt_ast::UnaryOp::Neg => "__neg",
                jolt_ast::UnaryOp::Not => "__not",
            };
            ctx.emit(MirInstr::Call {
                dest: Some(dest),
                callee: callee.to_string(),
                args: vec![inner_id],
            });
            Some(dest)
        }
        Expr::Binary(op, left, right) => {
            let l = lower_expr(ctx, left)?;
            let r = lower_expr(ctx, right)?;
            let dest = ctx.fresh_local();
            let callee = match op.value {
                BinaryOp::Add => "__add",
                BinaryOp::Sub => "__sub",
                BinaryOp::Mul => "__mul",
                BinaryOp::Div => "__div",
                BinaryOp::Eq => "__eq",
                BinaryOp::NotEq => "__ne",
                BinaryOp::Lt => "__lt",
                BinaryOp::Gt => "__gt",
                BinaryOp::Le => "__le",
                BinaryOp::Ge => "__ge",
                BinaryOp::And => "__and",
                BinaryOp::Or => "__or",
                _ => {
                    ctx.error(op.span, format!("binary op {:?} not lowered yet", op.value));
                    return None;
                }
            };
            ctx.emit(MirInstr::Call {
                dest: Some(dest),
                callee: callee.to_string(),
                args: vec![l, r],
            });
            Some(dest)
        }
        Expr::Call(callee, args) => {
            let arg_ids: Vec<LocalId> = args.iter().filter_map(|a| lower_expr(ctx, a)).collect();
            let dest = ctx.fresh_local();
            ctx.emit(MirInstr::Call {
                dest: Some(dest),
                callee: callee.value.clone(),
                args: arg_ids,
            });
            Some(dest)
        }
        Expr::Block(b) => lower_block_value(ctx, b),
    }
}
