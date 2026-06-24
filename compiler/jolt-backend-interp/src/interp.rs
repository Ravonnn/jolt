use jolt_mir::{MirFn, MirInstr, MirModule};
use std::fmt;

use crate::value::Value;

/// Outcome of interpreting a MIR module.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct InterpretResult {
    pub stdout: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum InterpretError {
    Runtime(String),
}

impl fmt::Display for InterpretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpretError::Runtime(msg) => write!(f, "{msg}"),
        }
    }
}

pub fn interpret(module: &MirModule) -> Result<InterpretResult, InterpretError> {
    let entry = module
        .entry_fn()
        .ok_or_else(|| InterpretError::Runtime("missing entry function".to_string()))?;
    let mut stdout = String::new();
    interpret_fn(module, entry, &mut stdout, None)?;
    Ok(InterpretResult {
        stdout,
        exit_code: 0,
    })
}

/// Run a single function (used by `jolt test`).
pub fn interpret_named_fn(
    module: &MirModule,
    name: &str,
    args: &[Value],
) -> Result<(), InterpretError> {
    let func = module
        .functions
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| InterpretError::Runtime(format!("function `{name}` not found")))?;
    let mut stdout = String::new();
    interpret_fn(module, func, &mut stdout, Some(args))?;
    Ok(())
}

fn interpret_fn(
    module: &MirModule,
    func: &MirFn,
    stdout: &mut String,
    arg_values: Option<&[Value]>,
) -> Result<Value, InterpretError> {
    let mut locals = vec![Value::Unit; func.local_count];

    if let Some(args) = arg_values {
        for (i, v) in args.iter().enumerate() {
            if i < func.param_count {
                locals[i] = v.clone();
            }
        }
    }

    let mut pc = 0usize;
    while pc < func.body.len() {
        match &func.body[pc] {
            MirInstr::ConstInt { dest, value } => {
                locals[*dest as usize] = Value::Int(*value);
            }
            MirInstr::ConstBool { dest, value } => {
                locals[*dest as usize] = Value::Bool(*value);
            }
            MirInstr::ConstString { dest, value } => {
                locals[*dest as usize] = Value::String(value.clone());
            }
            MirInstr::CopyLocal { dest, src } => {
                locals[*dest as usize] = locals[*src as usize].clone();
            }
            MirInstr::Call { dest, callee, args } => {
                let arg_vals: Vec<Value> =
                    args.iter().map(|&id| locals[id as usize].clone()).collect();
                let result = if is_user_fn(callee) {
                    let callee_fn = module
                        .functions
                        .iter()
                        .find(|f| f.name == *callee)
                        .ok_or_else(|| {
                            InterpretError::Runtime(format!("unknown function `{callee}`"))
                        })?;
                    interpret_fn(module, callee_fn, stdout, Some(&arg_vals))?
                } else {
                    dispatch_builtin(callee, &arg_vals, stdout)?
                };
                if let Some(d) = dest {
                    locals[*d as usize] = result;
                }
            }
            MirInstr::Return { value } => {
                return Ok(value
                    .map(|id| locals[id as usize].clone())
                    .unwrap_or(Value::Unit));
            }
            MirInstr::BranchIf {
                cond,
                then_pc,
                else_pc,
            } => {
                let b = locals[*cond as usize]
                    .as_bool()
                    .ok_or_else(|| InterpretError::Runtime("branch expects Bool".to_string()))?;
                pc = if b {
                    *then_pc as usize
                } else {
                    *else_pc as usize
                };
                continue;
            }
            MirInstr::Jump { target } => {
                pc = *target as usize;
                continue;
            }
        }
        pc += 1;
    }

    Ok(Value::Unit)
}

fn is_user_fn(callee: &str) -> bool {
    !matches!(
        callee,
        "println"
            | "assert_eq"
            | "assert_eq!"
            | "__neg"
            | "__not"
            | "__add"
            | "__sub"
            | "__mul"
            | "__div"
            | "__eq"
            | "__ne"
            | "__lt"
            | "__gt"
            | "__le"
            | "__ge"
            | "__and"
            | "__or"
            | "borrow"
            | "claim"
            | "deref"
    )
}

fn dispatch_builtin(
    callee: &str,
    args: &[Value],
    stdout: &mut String,
) -> Result<Value, InterpretError> {
    match callee {
        "println" => {
            if args.len() != 1 {
                return Err(InterpretError::Runtime(format!(
                    "println expects 1 argument, got {}",
                    args.len()
                )));
            }
            let s = args[0]
                .as_string()
                .ok_or_else(|| InterpretError::Runtime("println expects String".to_string()))?;
            stdout.push_str(s);
            stdout.push('\n');
            Ok(Value::Unit)
        }
        "assert_eq" | "assert_eq!" => {
            if args.len() != 2 {
                return Err(InterpretError::Runtime(format!(
                    "assert_eq! expects 2 arguments, got {}",
                    args.len()
                )));
            }
            let eq = values_equal(&args[0], &args[1]);
            if eq {
                Ok(Value::Unit)
            } else {
                Err(InterpretError::Runtime(format!(
                    "assert_eq! failed: left = {:?}, right = {:?}",
                    args[0], args[1]
                )))
            }
        }
        "__neg" => {
            let n = args
                .first()
                .and_then(Value::as_int)
                .ok_or_else(|| InterpretError::Runtime("__neg expects Int".to_string()))?;
            Ok(Value::Int(-n))
        }
        "__not" => {
            let b = args
                .first()
                .and_then(Value::as_bool)
                .ok_or_else(|| InterpretError::Runtime("__not expects Bool".to_string()))?;
            Ok(Value::Bool(!b))
        }
        "__add" => int_bin(args, |a, b| a + b),
        "__sub" => int_bin(args, |a, b| a - b),
        "__mul" => int_bin(args, |a, b| a * b),
        "__div" => int_bin(args, |a, b| a / b),
        "__eq" => cmp_bin(args, |a, b| a == b),
        "__ne" => cmp_bin(args, |a, b| a != b),
        "__lt" => cmp_bin(args, |a, b| a < b),
        "__gt" => cmp_bin(args, |a, b| a > b),
        "__le" => cmp_bin(args, |a, b| a <= b),
        "__ge" => cmp_bin(args, |a, b| a >= b),
        "__and" => bool_bin(args, |a, b| a && b),
        "__or" => bool_bin(args, |a, b| a || b),
        "borrow" | "claim" | "deref" => Ok(args.first().cloned().unwrap_or(Value::Unit)),
        other => Err(InterpretError::Runtime(format!(
            "unknown function `{other}` at runtime"
        ))),
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Unit, Value::Unit) => true,
        _ => false,
    }
}

fn int_bin(args: &[Value], f: fn(i64, i64) -> i64) -> Result<Value, InterpretError> {
    if args.len() != 2 {
        return Err(InterpretError::Runtime(
            "expected 2 Int arguments".to_string(),
        ));
    }
    let a = args[0]
        .as_int()
        .ok_or_else(|| InterpretError::Runtime("expected Int".to_string()))?;
    let b = args[1]
        .as_int()
        .ok_or_else(|| InterpretError::Runtime("expected Int".to_string()))?;
    Ok(Value::Int(f(a, b)))
}

fn cmp_bin(args: &[Value], f: fn(i64, i64) -> bool) -> Result<Value, InterpretError> {
    if args.len() != 2 {
        return Err(InterpretError::Runtime(
            "expected 2 Int arguments".to_string(),
        ));
    }
    let a = args[0]
        .as_int()
        .ok_or_else(|| InterpretError::Runtime("expected Int".to_string()))?;
    let b = args[1]
        .as_int()
        .ok_or_else(|| InterpretError::Runtime("expected Int".to_string()))?;
    Ok(Value::Bool(f(a, b)))
}

fn bool_bin(args: &[Value], f: fn(bool, bool) -> bool) -> Result<Value, InterpretError> {
    if args.len() != 2 {
        return Err(InterpretError::Runtime(
            "expected 2 Bool arguments".to_string(),
        ));
    }
    let a = args[0]
        .as_bool()
        .ok_or_else(|| InterpretError::Runtime("expected Bool".to_string()))?;
    let b = args[1]
        .as_bool()
        .ok_or_else(|| InterpretError::Runtime("expected Bool".to_string()))?;
    Ok(Value::Bool(f(a, b)))
}
