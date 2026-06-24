use jolt_ast::{BinaryOp, Expr, FnBody, Item};
use jolt_lexer::lex;
use jolt_parser::{parse_tokens, ParseResult};
use jolt_query::{hash_bytes, QueryEngine};
use jolt_source::FileId;

use crate::error::ResolveErrorKind;
use crate::query::{resolve_file, resolve_parsed, RESOLVE_FILE};
use crate::resolve_program;

fn fid() -> FileId {
    FileId(1)
}

fn parse(src: &str) -> ParseResult {
    parse_tokens(&lex(src, fid()))
}

fn resolve(src: &str) -> crate::ResolveResult {
    resolve_parsed(&parse(src))
}

fn has_kind(result: &crate::ResolveResult, kind: ResolveErrorKind) -> bool {
    result.errors.iter().any(|e| e.kind == kind)
}

#[test]
fn param_used_in_body_ok() {
    let r = resolve("@double(x: Int) Int -> x * 2 ;;");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn immutable_use_ok() {
    let r = resolve("@main() None -> $x = 1; x ;;");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn mutable_reassign_ok() {
    let r = resolve("@m() None -> $$y = 1; y = 2; ;;");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn immutable_reassign_error() {
    let r = resolve("@m() None -> $x = 1; x = 2; ;;");
    assert!(!r.is_ok());
    assert!(has_kind(&r, ResolveErrorKind::ImmutableAssign));
}

#[test]
fn undefined_name_error() {
    let r = resolve("@f() None -> unknown ;;");
    assert!(!r.is_ok());
    assert!(has_kind(&r, ResolveErrorKind::UndefinedName));
}

#[test]
fn duplicate_binding_error() {
    let r = resolve("@f() None -> $x = 1; $x = 2; ;;");
    assert!(!r.is_ok());
    assert!(has_kind(&r, ResolveErrorKind::DuplicateBinding));
}

#[test]
fn inner_shadow_ok() {
    let r = resolve("@f() None -> $$x = 1; -> $$x = 2; x ;; ;;");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn for_loop_binding_scope() {
    let r = resolve("@h(xs: Int) None -> for i in xs -> i ;; ;;");
    assert!(r.is_ok(), "{:?}", r.errors);
    let r2 = resolve("@bad() None -> i ;;");
    assert!(!r2.is_ok());
    assert!(has_kind(&r2, ResolveErrorKind::UndefinedName));
}

#[test]
fn parse_errors_still_resolve() {
    let parse = parse("@bad() None -> if 1 ;;");
    assert!(!parse.is_ok());
    let r = resolve_parsed(&parse);
    // Resolver runs on partial AST; may or may not produce resolve errors.
    assert_eq!(r.program.items.len(), parse.program.items.len());
}

#[test]
fn resolve_file_query_cache() {
    let mut engine = QueryEngine::new();
    let f = fid();
    let src = "@main() None -> 0 ;;";
    resolve_file(&mut engine, f, src);
    let c1 = engine.compute_count(RESOLVE_FILE);
    resolve_file(&mut engine, f, src);
    assert_eq!(engine.compute_count(RESOLVE_FILE), c1);
}

#[test]
fn resolve_invalidates_on_input_change() {
    let mut engine = QueryEngine::new();
    let f = fid();
    let src = "@main() None -> 0 ;;";
    let h = hash_bytes(src.as_bytes());
    resolve_file(&mut engine, f, src);
    let before = engine.compute_count(RESOLVE_FILE);
    engine.invalidate_input(h);
    resolve_file(&mut engine, f, src);
    assert!(engine.compute_count(RESOLVE_FILE) > before);
}

#[test]
fn resolve_depends_on_parse() {
    let mut engine = QueryEngine::new();
    let f = fid();
    resolve_file(&mut engine, f, "@main() None -> 1 ;;");
    let parse_before = engine.compute_count(jolt_parser::PARSE_FILE);
    let resolve_before = engine.compute_count(RESOLVE_FILE);
    resolve_file(&mut engine, f, "@main() None -> 2 ;;");
    assert!(engine.compute_count(jolt_parser::PARSE_FILE) > parse_before);
    assert!(engine.compute_count(RESOLVE_FILE) > resolve_before);
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
            let parse = parse(&src);
            assert!(parse.is_ok(), "{name}: parse {:?}", parse.errors);
            let r = resolve_parsed(&parse);
            assert!(r.is_ok(), "{name}: resolve {:?}", r.errors);
            count += 1;
        }
    }
    assert!(count >= 5, "expected at least 5 tiny ui files");
}

#[test]
fn ui_tiny_reject_corpus() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/ui/tiny-reject");
    let cases = [
        ("immutable_reassign.jolt", ResolveErrorKind::ImmutableAssign),
        ("undefined_name.jolt", ResolveErrorKind::UndefinedName),
        ("duplicate_binding.jolt", ResolveErrorKind::DuplicateBinding),
    ];
    for (file, expected) in cases {
        let path = root.join(file);
        let src = std::fs::read_to_string(&path).expect("read reject case");
        let parse = parse(&src);
        let r = resolve_parsed(&parse);
        assert!(
            has_kind(&r, expected),
            "{file}: expected {expected:?}, got {:?}",
            r.errors
        );
    }
}

#[test]
fn resolve_program_direct() {
    let parse = parse("@double(x: Int) Int -> x * 2 ;;");
    let r = resolve_program(&parse.program);
    assert!(r.is_ok());
    assert!(!r.symbols.is_empty());
    match &parse.program.items[0] {
        Item::Fn(f) => match &f.body {
            FnBody::Expr(Expr::Binary(op, _, _)) => assert_eq!(op.value, BinaryOp::Mul),
            b => panic!("unexpected body {b:?}"),
        },
    }
}
