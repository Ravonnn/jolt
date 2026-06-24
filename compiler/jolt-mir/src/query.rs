use jolt_custody::{custody_file, CustodyResult};
use jolt_query::{hash_bytes, QueryEngine, QueryName};
use jolt_source::FileId;

use crate::lower::lower_program;
use crate::MirResult;

/// Query name for per-file MIR lowering (depends on [`CUSTODY_FILE`]).
pub const MIR_FILE: QueryName = QueryName("mir_file");

pub fn mir_file(engine: &mut QueryEngine, file: FileId, source: &str) -> MirResult {
    let input = hash_bytes(source.as_bytes());
    engine.query(MIR_FILE, input, &[input], |eng| {
        let custody = custody_file(eng, file, source);
        mir_custodied(&custody)
    })
}

pub fn mir_custodied(custody: &CustodyResult) -> MirResult {
    if !custody.is_ok() {
        return MirResult {
            program: custody.program.clone(),
            module: crate::MirModule {
                entry: crate::lower::ENTRY_FN.to_string(),
                functions: vec![],
            },
            diagnostics: Default::default(),
        };
    }
    let (module, diagnostics) = lower_program(&custody.program);
    MirResult {
        program: custody.program.clone(),
        module,
        diagnostics,
    }
}
