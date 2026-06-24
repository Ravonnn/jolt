use jolt_ast::{BinaryOp, Expr, FnBody, Item};
use jolt_lexer::lex;
use jolt_query::{hash_bytes, QueryEngine};
use jolt_source::FileId;

use crate::parse_file;
use crate::parse_tokens;
use crate::PARSE_FILE;

fn fid() -> FileId {
    FileId(1)
}

fn parse(src: &str) -> crate::ParseResult {
    parse_tokens(&lex(src, fid()))
}

#[test]
fn parse_single_expr_fn() {
    let r = parse("@double(x: Int) Int -> x * 2 ;;");
    assert!(r.is_ok(), "{:?}", r.errors);
    assert_eq!(r.program.items.len(), 1);
    match &r.program.items[0] {
        Item::Fn(f) => {
            assert_eq!(f.name.name, "double");
            match &f.body {
                FnBody::Expr(Expr::Binary(op, _, _)) => assert_eq!(op.value, BinaryOp::Mul),
                other => panic!("expected mul expr body, got {other:?}"),
            }
        }
    }
}

#[test]
fn parse_block_with_tail() {
    let src = "@main() None -> $x = 1; x ;;";
    let r = parse(src);
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn parse_if_else_fn() {
    let src = "@f(x: Int) Int -> if x > 0 -> 1 ;; else -> 0 ;; ;;";
    let r = parse(src);
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn parse_loop_and_for() {
    let r = parse("@g() None -> loop -> break; ;; ;;");
    assert!(r.is_ok(), "{:?}", r.errors);
    let r2 = parse("@h() None -> for i in xs -> i ;; ;;");
    assert!(r2.is_ok(), "{:?}", r2.errors);
}

#[test]
fn parse_assign_after_binding() {
    let r = parse("@m() None -> $x = 1; x = 2; ;;");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn parse_precedence() {
    let r = parse("@p() Int -> 1 + 2 * 3 ;;");
    assert!(r.is_ok(), "{:?}", r.errors);
    match &r.program.items[0] {
        Item::Fn(f) => match &f.body {
            FnBody::Expr(Expr::Binary(add, _, right)) => {
                assert_eq!(add.value, BinaryOp::Add);
                match &**right {
                    Expr::Binary(mul, _, _) => assert_eq!(mul.value, BinaryOp::Mul),
                    e => panic!("expected mul on right, got {e:?}"),
                }
            }
            b => panic!("expected expr body {b:?}"),
        },
    }
}

#[test]
fn parse_error_missing_semi_semi() {
    let r = parse("@bad() None -> 1 ");
    assert!(!r.is_ok());
    assert!(!r.errors.is_empty());
}

#[test]
fn parse_error_malformed_if_partial() {
    let r = parse("@bad() None -> if 1 ;;");
    assert!(!r.errors.is_empty());
}

#[test]
fn parse_file_query_cache() {
    let mut engine = QueryEngine::new();
    let f = fid();
    let src = "@main() None -> 0 ;;";
    parse_file(&mut engine, f, src);
    let c1 = engine.compute_count(PARSE_FILE);
    parse_file(&mut engine, f, src);
    assert_eq!(engine.compute_count(PARSE_FILE), c1);
}

#[test]
fn parse_invalidates_on_input_change() {
    let mut engine = QueryEngine::new();
    let f = fid();
    let h = hash_bytes(b"@main() None -> 0 ;;");
    parse_file(&mut engine, f, "@main() None -> 0 ;;");
    let before = engine.compute_count(PARSE_FILE);
    engine.invalidate_input(h);
    parse_file(&mut engine, f, "@main() None -> 0 ;;");
    assert!(engine.compute_count(PARSE_FILE) > before);
}

#[test]
fn parse_depends_on_lex() {
    let mut engine = QueryEngine::new();
    let f = fid();
    parse_file(&mut engine, f, "@main() None -> 1 ;;");
    let lex_before = engine.compute_count(jolt_lexer::LEX_FILE);
    let parse_before = engine.compute_count(PARSE_FILE);
    parse_file(&mut engine, f, "@main() None -> 2 ;;");
    assert!(engine.compute_count(jolt_lexer::LEX_FILE) > lex_before);
    assert!(engine.compute_count(PARSE_FILE) > parse_before);
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
            let r = parse(&src);
            assert!(r.is_ok(), "{name}: {:?}", r.errors);
            count += 1;
        }
    }
    assert!(count >= 5, "expected at least 5 tiny ui files");
}
