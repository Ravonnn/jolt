use jolt_mir::{mir_file, MirResult};
use jolt_query::{hash_bytes, QueryEngine, QueryName};
use jolt_source::FileId;

use crate::interp::{interpret, InterpretError, InterpretResult};

/// Query name for per-file interpretation (depends on [`MIR_FILE`]).
pub const RUN_FILE: QueryName = QueryName("run_file");

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct RunResult {
    pub mir: MirResult,
    pub interpret: Result<InterpretResult, InterpretError>,
}

impl RunResult {
    pub fn is_ok(&self) -> bool {
        self.mir.is_ok() && self.interpret.is_ok()
    }

    pub fn stdout(&self) -> Option<&str> {
        self.interpret.as_ref().ok().map(|r| r.stdout.as_str())
    }
}

pub fn run_file(engine: &mut QueryEngine, file: FileId, source: &str) -> RunResult {
    let input = hash_bytes(source.as_bytes());
    engine.query(RUN_FILE, input, &[input], |eng| {
        let mir = mir_file(eng, file, source);
        run_mir(&mir)
    })
}

pub fn run_mir(mir: &MirResult) -> RunResult {
    if !mir.is_ok() {
        return RunResult {
            mir: mir.clone(),
            interpret: Err(InterpretError::Runtime(
                "skipped interpretation: MIR lowering failed".to_string(),
            )),
        };
    }
    if !mir.has_entry() {
        return RunResult {
            mir: mir.clone(),
            interpret: Err(InterpretError::Runtime(
                "missing entry function `main`".to_string(),
            )),
        };
    }
    let interpret = interpret(&mir.module);
    RunResult {
        mir: mir.clone(),
        interpret,
    }
}
