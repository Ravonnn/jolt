use jolt_diagnostics::render_diagnostic;
use jolt_lexer::lex;
use jolt_parser::parse_tokens;
use jolt_query::{hash_bytes, QueryEngine};
use jolt_resolve::resolve_parsed;
use jolt_source::FileId;

use crate::error::TypeErrorKind;
use crate::query::{check_file, check_resolved, CHECK_FILE};
use crate::{check_program, Ty};

fn fid() -> FileId {
    FileId(1)
}

fn parse_and_resolve(src: &str) -> jolt_resolve::ResolveResult {
    resolve_parsed(&parse_tokens(&lex(src, fid())))
}

fn check(src: &str) -> crate::CheckResult {
    check_resolved(&parse_and_resolve(src))
}

fn has_code(result: &crate::CheckResult, code: &str) -> bool {
    result
        .diagnostics
        .items
        .iter()
        .any(|d| d.code.as_deref() == Some(code))
}

#[test]
fn double_fn_ok() {
    let r = check("@double(x: Int) Int -> x * 2 ;;");
    assert!(r.is_ok(), "{:?}", r.diagnostics.items);
}

#[test]
fn return_type_mismatch_error() {
    let r = check("@f() Int -> true ;;");
    assert!(!r.is_ok());
    assert!(has_code(&r, TypeErrorKind::Mismatch.code()));
}

#[test]
fn binop_type_error() {
    let r = check("@f() Int -> 1 + true ;;");
    assert!(!r.is_ok());
    assert!(has_code(&r, TypeErrorKind::BinOpMismatch.code()));
}

#[test]
fn println_ok() {
    let r = check("@f() None -> println(\"ok\"); ;;");
    assert!(r.is_ok(), "{:?}", r.diagnostics.items);
}

#[test]
fn println_wrong_type() {
    let r = check("@f() None -> println(1); ;;");
    assert!(!r.is_ok());
    assert!(has_code(&r, TypeErrorKind::Mismatch.code()));
}

#[test]
fn param_use_ok() {
    let r = check("@f(x: Int) Int -> x ;;");
    assert!(r.is_ok(), "{:?}", r.diagnostics.items);
}

#[test]
fn if_branches_ok() {
    let r = check("@g() Int -> if true -> 1 ;; else -> 2 ;; ;;");
    assert!(r.is_ok(), "{:?}", r.diagnostics.items);
}

#[test]
fn if_branch_mismatch() {
    let r = check("@g() Int -> if true -> 1 ;; else -> true ;; ;;");
    assert!(!r.is_ok());
    assert!(has_code(&r, TypeErrorKind::BranchMismatch.code()));
}

#[test]
fn negative_int_literal() {
    let r = check("@f() Int -> -3 ;;");
    assert!(r.is_ok(), "{:?}", r.diagnostics.items);
}

#[test]
fn resolve_errors_skip_type_check() {
    let resolved = parse_and_resolve("@f() None -> unknown ;;");
    assert!(!resolved.is_ok());
    let r = check_resolved(&resolved);
    assert!(r.is_ok(), "type pass skipped when resolve fails");
}

#[test]
fn check_file_query_cache() {
    let mut engine = QueryEngine::new();
    let f = fid();
    let src = "@main() None -> 0 ;;";
    check_file(&mut engine, f, src);
    let c1 = engine.compute_count(CHECK_FILE);
    check_file(&mut engine, f, src);
    assert_eq!(engine.compute_count(CHECK_FILE), c1);
}

#[test]
fn check_invalidates_on_input_change() {
    let mut engine = QueryEngine::new();
    let f = fid();
    let src = "@main() None -> 0 ;;";
    let h = hash_bytes(src.as_bytes());
    check_file(&mut engine, f, src);
    let before = engine.compute_count(CHECK_FILE);
    engine.invalidate_input(h);
    check_file(&mut engine, f, src);
    assert!(engine.compute_count(CHECK_FILE) > before);
}

#[test]
fn check_depends_on_resolve() {
    let mut engine = QueryEngine::new();
    let f = fid();
    check_file(&mut engine, f, "@main() None -> 1 ;;");
    let resolve_before = engine.compute_count(jolt_resolve::RESOLVE_FILE);
    let check_before = engine.compute_count(CHECK_FILE);
    check_file(&mut engine, f, "@main() None -> 2 ;;");
    assert!(engine.compute_count(jolt_resolve::RESOLVE_FILE) > resolve_before);
    assert!(engine.compute_count(CHECK_FILE) > check_before);
}

#[test]
fn ui_tiny_corpus_accepts() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/ui/tiny");
    let mut count = 0;
    for entry in std::fs::read_dir(&root).expect("tiny dir") {
        let path = entry.expect("entry").path();
        if path.extension().is_some_and(|e| e == "jolt") {
            let src = std::fs::read_to_string(&path).expect("read");
            let name = path.file_name().unwrap().to_string_lossy();
            let r = check(&src);
            assert!(r.is_ok(), "{name}: {:?}", r.diagnostics.items);
            count += 1;
        }
    }
    assert!(count >= 5, "expected at least 5 tiny ui files");
}

fn rendered_lines(src: &str, result: &crate::CheckResult) -> Vec<String> {
    result
        .diagnostics
        .items
        .iter()
        .map(|d| render_diagnostic(d, src))
        .collect()
}

#[test]
fn ui_type_reject_corpus() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/ui/type-reject");
    for entry in std::fs::read_dir(&root).expect("type-reject dir") {
        let path = entry.expect("entry").path();
        if path.extension().is_some_and(|e| e == "jolt") {
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            let src = std::fs::read_to_string(&path).expect("read jolt");
            let stderr_path = path.with_extension("stderr");
            let resolved = parse_and_resolve(&src);
            assert!(
                resolved.is_ok(),
                "{stem}: resolve errors {:?}",
                resolved.errors
            );
            let r = check_resolved(&resolved);
            assert!(!r.is_ok(), "{stem}: expected type errors");
            let actual: Vec<String> = rendered_lines(&src, &r)
                .into_iter()
                .map(|l| l.trim_end().to_string())
                .collect();
            if std::env::var("UPDATE_UI").is_ok() {
                let body = actual.join("\n");
                let body = if body.is_empty() {
                    body
                } else {
                    format!("{body}\n")
                };
                std::fs::write(&stderr_path, body).expect("write stderr snapshot");
                continue;
            }
            let expected = std::fs::read_to_string(&stderr_path).unwrap_or_else(|_| {
                panic!(
                    "missing {} — create stderr snapshot from rendered diagnostics",
                    stderr_path.display()
                )
            });
            let expected: Vec<String> = expected
                .lines()
                .map(str::trim_end)
                .filter(|l| !l.is_empty())
                .map(str::to_string)
                .collect();
            assert_eq!(actual, expected, "{stem}");
        }
    }
}

#[test]
fn check_program_direct() {
    let resolved = parse_and_resolve("@double(x: Int) Int -> x * 2 ;;");
    let diags = check_program(&resolved.program);
    assert!(diags.is_empty());
    let _ = Ty::Int;
}
