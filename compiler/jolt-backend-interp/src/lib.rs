//! MIR interpreter backend for Tiny (Phase 3a).

mod interp;
mod query;
mod value;

#[cfg(test)]
mod tests;

pub use interp::{interpret, interpret_named_fn, InterpretError, InterpretResult};
pub use jolt_mir;
pub use query::{run_file, run_mir, RunResult, RUN_FILE};
pub use value::Value;

pub const STAGE: &str = "backend-interp";
