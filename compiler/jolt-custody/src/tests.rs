use jolt_lexer::lex;
use jolt_parser::parse_tokens;
use jolt_resolve::resolve_parsed;
use jolt_source::FileId;
use jolt_types::check_program as type_check;

use super::*;
use crate::query::custody_checked;

fn fid() -> FileId {
    FileId(1)
}

fn parse_and_resolve(src: &str) -> jolt_resolve::ResolveResult {
    resolve_parsed(&parse_tokens(&lex(src, fid())))
}

fn check(src: &str) -> CustodyResult {
    let resolved = parse_and_resolve(src);
    assert!(resolved.is_ok(), "{:?}", resolved.errors);
    let types = type_check(&resolved.program);
    assert!(types.is_empty(), "{types:?}");
    let diagnostics = check_program(&resolved.program);
    CustodyResult {
        program: resolved.program,
        diagnostics,
    }
}

fn custody_errors(src: &str) -> CustodyResult {
    check(src)
}

fn has_code(r: &CustodyResult, code: &str) -> bool {
    r.diagnostics
        .items
        .iter()
        .any(|d| d.code.as_deref() == Some(code))
}

#[test]
fn int_copy_allows_reuse() {
    let r = check("@f() Int -> $x = 1; $y = x; x + y ;;");
    assert!(r.is_ok(), "{:?}", r.diagnostics.items);
}

#[test]
fn string_move_ok() {
    let r = check("@f() None -> $a = \"hi\"; $b = a; println(b); ;;");
    assert!(r.is_ok(), "{:?}", r.diagnostics.items);
}

#[test]
fn use_after_move_on_assign() {
    let r = custody_errors("@f() None -> $a = \"hi\"; $b = a; println(a); ;;");
    assert!(!r.is_ok());
    assert!(has_code(&r, CustodyErrorKind::UseAfterMove.code()));
}

#[test]
fn use_after_move_on_call() {
    let r = custody_errors("@f() None -> $s = \"x\"; println(s); println(s); ;;");
    assert!(!r.is_ok());
    assert!(has_code(&r, CustodyErrorKind::UseAfterMove.code()));
}

#[test]
fn shared_borrows_ok() {
    let r = check(
        "@f() None -> $$data = \"hi\"; $v1 = borrow(data); $v2 = borrow(data); println(deref(v1)); ;;",
    );
    assert!(r.is_ok(), "{:?}", r.diagnostics.items);
}

#[test]
fn borrow_nll_reborrow_after_last_use() {
    let r = check(
        "@f() None -> $data = \"hi\"; $v = borrow(data); println(deref(v)); $w = borrow(data); println(deref(w)); ;;",
    );
    assert!(r.is_ok(), "{:?}", r.diagnostics.items);
}

#[test]
fn claim_while_borrowed_rejected() {
    let r = custody_errors("@f() None -> $$data = \"hi\"; $v = borrow(data); $e = claim(data); ;;");
    assert!(!r.is_ok());
    assert!(has_code(&r, CustodyErrorKind::ClaimWhileBorrowed.code()));
}

#[test]
fn shared_while_claimed_rejected() {
    let r = custody_errors("@f() None -> $$data = \"hi\"; $e = claim(data); $v = borrow(data); ;;");
    assert!(!r.is_ok());
    assert!(has_code(&r, CustodyErrorKind::SharedWhileClaimed.code()));
}

#[test]
fn use_while_borrowed_rejected() {
    let r = custody_errors("@f() None -> $data = \"hi\"; $v = borrow(data); println(data); ;;");
    assert!(!r.is_ok());
    assert!(has_code(&r, CustodyErrorKind::UseWhileBorrowed.code()));
}

#[test]
fn double_claim_rejected() {
    let r =
        custody_errors("@f() None -> $$data = \"hi\"; $e1 = claim(data); $e2 = claim(data); ;;");
    assert!(!r.is_ok());
    assert!(has_code(&r, CustodyErrorKind::ClaimWhileBorrowed.code()));
}

#[test]
fn assign_while_borrowed_rejected() {
    let r = custody_errors("@f() None -> $data = \"hi\"; $v = borrow(data); $b = data; ;;");
    assert!(!r.is_ok());
    assert!(has_code(&r, CustodyErrorKind::UseWhileBorrowed.code()));
}

#[test]
fn custody_hints_rendered() {
    let r = custody_errors("@f() None -> $a = \"hi\"; $b = a; println(a); ;;");
    let line = jolt_diagnostics::render_diagnostic(&r.diagnostics.items[0], "");
    assert!(line.contains("; hint:"), "{line}");
}

#[test]
fn custody_skipped_when_types_fail() {
    let resolved = parse_and_resolve("@f() Int -> true ;;");
    let checked = jolt_types::CheckResult {
        program: resolved.program.clone(),
        diagnostics: type_check(&resolved.program),
    };
    assert!(!checked.is_ok());
    let r = custody_checked(&checked);
    assert!(r.is_ok(), "custody skipped when type errors present");
}
