//! MIR types for the Tiny subset (Phase 3a).

/// Index into a function's local slot table (params occupy `0..param_count`).
pub type LocalId = u32;

/// A lowered program ready for interpretation.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct MirModule {
    pub entry: String,
    pub functions: Vec<MirFn>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct MirFn {
    pub name: String,
    pub param_count: usize,
    pub local_count: usize,
    pub body: Vec<MirInstr>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum MirInstr {
    ConstInt {
        dest: LocalId,
        value: i64,
    },
    ConstBool {
        dest: LocalId,
        value: bool,
    },
    ConstString {
        dest: LocalId,
        value: String,
    },
    CopyLocal {
        dest: LocalId,
        src: LocalId,
    },
    Call {
        dest: Option<LocalId>,
        callee: String,
        args: Vec<LocalId>,
    },
    Return {
        value: Option<LocalId>,
    },
    BranchIf {
        cond: LocalId,
        then_pc: u32,
        else_pc: u32,
    },
    Jump {
        target: u32,
    },
}

impl MirModule {
    pub fn entry_fn(&self) -> Option<&MirFn> {
        self.functions.iter().find(|f| f.name == self.entry)
    }
}
