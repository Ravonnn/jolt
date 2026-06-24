use jolt_lexer::lex;
use jolt_parser::parse_tokens;
use jolt_resolve::resolve_parsed;
use jolt_source::FileId;

use super::*;

#[test]
fn interpret_hello() {
    let src = "@main() None -> println(\"Hello, Jolt!\"); ;;";
    let resolved = resolve_parsed(&parse_tokens(&lex(src, FileId(1))));
    assert!(resolved.is_ok());
    let (module, diags) = jolt_mir::lower_program(&resolved.program);
    assert!(diags.is_empty(), "{diags:?}");
    let r = interpret(&module).expect("interpret");
    assert_eq!(r.stdout, "Hello, Jolt!\n");
    assert_eq!(r.exit_code, 0);
}

fn lower_and_interpret(src: &str) -> InterpretResult {
    let resolved = resolve_parsed(&parse_tokens(&lex(src, FileId(1))));
    assert!(resolved.is_ok(), "{:?}", resolved.errors);
    let (module, diags) = jolt_mir::lower_program(&resolved.program);
    assert!(diags.is_empty(), "{diags:?}");
    interpret(&module).expect("interpret")
}

#[test]
fn interpret_if_else_user_fn() {
    let src = r#"
@max(a: Int, b: Int) Int ->
    if a > b -> a ;; else -> b ;;
;;
@main() None ->
    $m = max(3, 7);
    if m == 7 -> println("7"); ;;
;;
"#;
    let r = lower_and_interpret(src);
    assert_eq!(r.stdout, "7\n");
}

#[test]
fn interpret_loop_break_immediately() {
    let src = "@main() None -> loop -> break; ;; println(\"done\"); ;;";
    let r = lower_and_interpret(src);
    assert_eq!(r.stdout, "done\n");
}

#[test]
fn interpret_loop_break() {
    let src = r#"
@main() None ->
    $$n = 0;
    loop ->
        n = n + 1;
        if n >= 10 -> break; ;;
    ;;
    if n == 10 -> println("10"); ;;
;;
"#;
    let r = lower_and_interpret(src);
    assert_eq!(r.stdout, "10\n");
}

#[test]
fn assert_eq_passes_in_test_fn() {
    let src = r#"
[test]
@ok() None -> assert_eq(1 + 1, 2); ;;
"#;
    let resolved = resolve_parsed(&parse_tokens(&lex(src, FileId(1))));
    let (module, diags) = jolt_mir::lower_program(&resolved.program);
    assert!(diags.is_empty(), "{diags:?}");
    interpret_named_fn(&module, "ok", &[]).expect("assert_eq passes");
}
