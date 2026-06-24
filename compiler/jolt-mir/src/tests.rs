use jolt_lexer::lex;
use jolt_parser::parse_tokens;
use jolt_resolve::resolve_parsed;
use jolt_source::FileId;
use jolt_types::check_program as type_check;

use super::*;

fn fid() -> FileId {
    FileId(1)
}

fn parse_and_resolve(src: &str) -> jolt_resolve::ResolveResult {
    resolve_parsed(&parse_tokens(&lex(src, fid())))
}

#[test]
fn lowers_hello_main() {
    let src = "@main() None -> println(\"Hello, Jolt!\"); ;;";
    let resolved = parse_and_resolve(src);
    assert!(resolved.is_ok());
    assert!(type_check(&resolved.program).is_empty());
    let (module, diags) = lower_program(&resolved.program);
    assert!(diags.is_empty(), "{diags:?}");
    let main_fn = module.entry_fn().expect("main");
    assert_eq!(main_fn.name, "main");
    assert!(main_fn.body.iter().any(|i| matches!(
        i,
        MirInstr::Call { callee, .. } if callee == "println"
    )));
}

#[test]
fn missing_main_still_lowers_other_fns() {
    let src = "@other() None -> 0 ;;";
    let resolved = parse_and_resolve(src);
    let (module, diags) = lower_program(&resolved.program);
    assert!(diags.is_empty(), "{diags:?}");
    assert!(module.entry_fn().is_none());
    assert_eq!(module.functions.len(), 1);
}

#[test]
fn lowers_if_else_fn() {
    let src = r#"
@max(a: Int, b: Int) Int ->
    if a > b -> a ;; else -> b ;;
;;
"#;
    let resolved = parse_and_resolve(src);
    assert!(resolved.is_ok());
    let (module, diags) = lower_program(&resolved.program);
    assert!(diags.is_empty(), "{diags:?}");
    let max_fn = module.functions.iter().find(|f| f.name == "max").unwrap();
    assert!(max_fn
        .body
        .iter()
        .any(|i| matches!(i, MirInstr::BranchIf { .. })));
}

#[test]
fn lowers_loop_with_break() {
    let src = r#"
@count() Int ->
    $$n = 0;
    loop ->
        n = n + 1;
        if n >= 10 -> break ;;
    ;;
    n
;;
"#;
    let resolved = parse_and_resolve(src);
    let (module, diags) = lower_program(&resolved.program);
    assert!(diags.is_empty(), "{diags:?}");
    let count_fn = module.functions.iter().find(|f| f.name == "count").unwrap();
    assert!(count_fn
        .body
        .iter()
        .any(|i| matches!(i, MirInstr::Jump { .. })));
}
