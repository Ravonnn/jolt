use jolt_lexer::lex;
use jolt_parser::{parse_tokens, ParseResult};
use jolt_query::{hash_bytes, QueryEngine};
use jolt_resolve::resolve_parsed;
use jolt_source::FileId;
use jolt_types::check_resolved;

use crate::query::{format_file, format_parsed, FMT_FILE};
use crate::{format_program, print_program};

fn fid() -> FileId {
    FileId(1)
}

fn parse(src: &str) -> ParseResult {
    parse_tokens(&lex(src, fid()))
}

fn format_src(src: &str) -> String {
    format_parsed(&parse(src), src).source
}

fn format_twice(src: &str) -> (String, String) {
    let once = format_src(src);
    let twice = format_src(&once);
    (once, twice)
}

#[test]
fn single_expr_fn_roundtrip() {
    let src = "@double(x: Int) Int -> x * 2 ;;";
    let once = format_src(src);
    assert_eq!(once.trim(), src.trim());
    assert!(parse(&once).is_ok());
}

#[test]
fn idempotent_on_double() {
    let src = "@double(x: Int) Int -> x * 2 ;;";
    let (once, twice) = format_twice(src);
    assert_eq!(once, twice);
}

#[test]
fn idempotent_on_negative_literal() {
    let src = "@f() Int -> -3 ;;";
    let (once, twice) = format_twice(src);
    assert_eq!(once, twice);
}

#[test]
fn parse_errors_return_original() {
    let src = "@bad() None -> 1 ";
    let r = format_parsed(&parse(src), src);
    assert!(!r.is_ok());
    assert_eq!(r.source, src);
}

#[test]
fn format_file_query_cache() {
    let mut engine = QueryEngine::new();
    let f = fid();
    let src = "@main() None -> 0 ;;";
    format_file(&mut engine, f, src);
    let c1 = engine.compute_count(FMT_FILE);
    format_file(&mut engine, f, src);
    assert_eq!(engine.compute_count(FMT_FILE), c1);
}

#[test]
fn format_invalidates_on_input_change() {
    let mut engine = QueryEngine::new();
    let f = fid();
    let src = "@main() None -> 0 ;;";
    let h = hash_bytes(src.as_bytes());
    format_file(&mut engine, f, src);
    let before = engine.compute_count(FMT_FILE);
    engine.invalidate_input(h);
    format_file(&mut engine, f, src);
    assert!(engine.compute_count(FMT_FILE) > before);
}

#[test]
fn format_depends_on_parse() {
    let mut engine = QueryEngine::new();
    let f = fid();
    format_file(&mut engine, f, "@main() None -> 1 ;;");
    let parse_before = engine.compute_count(jolt_parser::PARSE_FILE);
    let fmt_before = engine.compute_count(FMT_FILE);
    format_file(&mut engine, f, "@main() None -> 2 ;;");
    assert!(engine.compute_count(jolt_parser::PARSE_FILE) > parse_before);
    assert!(engine.compute_count(FMT_FILE) > fmt_before);
}

#[test]
fn ui_tiny_corpus_idempotent() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/ui/tiny");
    for entry in std::fs::read_dir(&root).expect("tiny dir") {
        let path = entry.expect("entry").path();
        if path.extension().is_some_and(|e| e == "jolt") {
            let src = std::fs::read_to_string(&path).expect("read");
            let name = path.file_name().unwrap().to_string_lossy();
            let parse1 = parse(&src);
            assert!(parse1.is_ok(), "{name}: {:?}", parse1.errors);
            let once = format_parsed(&parse1, &src).source;
            let parse2 = parse(&once);
            assert!(parse2.is_ok(), "{name} after fmt: {:?}", parse2.errors);
            let twice = format_parsed(&parse2, &once).source;
            assert_eq!(once, twice, "{name}: fmt not idempotent");
            let resolved = resolve_parsed(&parse2);
            assert!(resolved.is_ok(), "{name} resolve after fmt");
            let checked = check_resolved(&resolved);
            assert!(
                checked.is_ok(),
                "{name} check after fmt: {:?}",
                checked.diagnostics.items
            );
        }
    }
}

#[test]
fn print_program_has_trailing_newline() {
    let src = "@main() None -> 0 ;;";
    let out = format_src(src);
    assert!(out.ends_with('\n'));
    assert_eq!(out, print_program(&parse(src).program));
}

#[test]
fn format_program_direct() {
    let p = parse("@a() None -> 1 ;;").program;
    assert_eq!(format_program(&p), "@a() None -> 1 ;;\n");
}
