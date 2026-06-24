//! Umbrella API for the Jolt stage-0 compiler.
//!
//! Tools and the CLI must depend on this crate only, not individual stage crates.

mod check;
mod run;
mod test;

pub use check::CheckReport;
pub use run::RunReport;
pub use test::TestSourceReport;

pub use jolt_ast;
pub use jolt_backend_fast;
pub use jolt_backend_interp;
pub use jolt_backend_llvm;
pub use jolt_cache;
pub use jolt_capability;
pub use jolt_comptime;
pub use jolt_custody;
pub use jolt_diagnostics;
pub use jolt_fmt;
pub use jolt_lexer;
pub use jolt_mir;
pub use jolt_mono;
pub use jolt_parser;
pub use jolt_query;
pub use jolt_resolve;
pub use jolt_source;
pub use jolt_types;

pub use jolt_backend_interp::{
    interpret, run_file, run_mir, InterpretError, InterpretResult, RunResult, Value, RUN_FILE,
};
pub use jolt_custody::{
    check_program as custody_program, custody_checked, custody_file, CustodyErrorKind,
    CustodyResult, CUSTODY_FILE,
};
pub use jolt_diagnostics::{
    line_col, render_at, render_diagnostic, Diagnostic, Diagnostics, Severity,
};
pub use jolt_fmt::{
    format_file, format_parsed, format_program, print_program, FormatResult, FMT_FILE,
};
pub use jolt_lexer::{lex, lex_file, LexErrorKind, Lexer, Token, TokenKind, LEX_FILE};
pub use jolt_mir::{
    lower_program, mir_custodied, mir_file, MirFn, MirInstr, MirModule, MirResult, MIR_FILE,
};
pub use jolt_parser::{
    parse_file, parse_source, ParseError, ParseErrorKind, ParseResult, PARSE_FILE,
};
pub use jolt_query::{InputHash, QueryEngine, QueryName};
pub use jolt_resolve::{
    resolve_file, resolve_parsed, resolve_program, BindingInfo, BindingOrigin, ResolveError,
    ResolveErrorKind, ResolveResult, SymbolId, RESOLVE_FILE,
};
pub use jolt_test_runner::{
    discover_tests, run_tests, test_file, TestCaseResult, TestFileResult, TestFn, TestReport,
    TEST_FILE,
};
pub use jolt_types::{
    check_file, check_program, check_resolved, CheckResult, Ty, TypeErrorKind, CHECK_FILE,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compiler session owning the query engine and (eventually) full pipeline state.
pub struct Session {
    engine: QueryEngine,
}

impl Session {
    pub fn new() -> Self {
        Self {
            engine: QueryEngine::new(),
        }
    }

    pub fn engine_mut(&mut self) -> &mut QueryEngine {
        &mut self.engine
    }

    /// Lex a source file (memoized via [`LEX_FILE`]).
    pub fn lex_file(&mut self, file: jolt_source::FileId, source: &str) -> Vec<Token> {
        jolt_lexer::lex_file(&mut self.engine, file, source)
    }

    /// Parse a source file (memoized via [`PARSE_FILE`]; lexes via [`LEX_FILE`]).
    pub fn parse_file(&mut self, file: jolt_source::FileId, source: &str) -> ParseResult {
        jolt_parser::parse_file(&mut self.engine, file, source)
    }

    /// Resolve names in a source file (memoized via [`RESOLVE_FILE`]; parses via [`PARSE_FILE`]).
    pub fn resolve_file(&mut self, file: jolt_source::FileId, source: &str) -> ResolveResult {
        jolt_resolve::resolve_file(&mut self.engine, file, source)
    }

    /// Type-check a source file (memoized via [`CHECK_FILE`]; resolves via [`RESOLVE_FILE`]).
    pub fn check_file(&mut self, file: jolt_source::FileId, source: &str) -> CheckResult {
        jolt_types::check_file(&mut self.engine, file, source)
    }

    /// Custody-check a source file (memoized via [`CUSTODY_FILE`]; type-checks via [`CHECK_FILE`]).
    pub fn custody_file(&mut self, file: jolt_source::FileId, source: &str) -> CustodyResult {
        jolt_custody::custody_file(&mut self.engine, file, source)
    }

    /// Lower a source file to MIR (memoized via [`MIR_FILE`]; custody via [`CUSTODY_FILE`]).
    pub fn mir_file(&mut self, file: jolt_source::FileId, source: &str) -> MirResult {
        jolt_mir::mir_file(&mut self.engine, file, source)
    }

    /// Interpret a source file (memoized via [`RUN_FILE`]; MIR via [`MIR_FILE`]).
    pub fn run_file(&mut self, file: jolt_source::FileId, source: &str) -> RunResult {
        jolt_backend_interp::run_file(&mut self.engine, file, source)
    }

    /// Run `[test]` functions in a source file (memoized via [`TEST_FILE`]).
    pub fn test_file(&mut self, file: jolt_source::FileId, source: &str) -> TestFileResult {
        jolt_test_runner::test_file(&mut self.engine, file, source)
    }

    /// Full pipeline test report with stage snapshots.
    pub fn test_source(&mut self, file: jolt_source::FileId, source: &str) -> TestSourceReport {
        test::test_source(self, file, source)
    }

    /// Format a source file (memoized via [`FMT_FILE`]; parses via [`PARSE_FILE`]).
    pub fn format_file(&mut self, file: jolt_source::FileId, source: &str) -> FormatResult {
        jolt_fmt::format_file(&mut self.engine, file, source)
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Names of all wired compiler stages (proves workspace wiring).
pub fn stage_names() -> &'static [&'static str] {
    &[
        jolt_source::STAGE,
        jolt_lexer::STAGE,
        jolt_ast::STAGE,
        jolt_parser::STAGE,
        jolt_query::STAGE,
        jolt_resolve::STAGE,
        jolt_types::STAGE,
        jolt_custody::STAGE,
        jolt_capability::STAGE,
        jolt_comptime::STAGE,
        jolt_mono::STAGE,
        jolt_mir::STAGE,
        jolt_backend_interp::STAGE,
        jolt_backend_llvm::STAGE,
        jolt_backend_fast::STAGE,
        jolt_diagnostics::STAGE,
        jolt_cache::STAGE,
        jolt_fmt::STAGE,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use jolt_query::demo::*;
    use jolt_query::hash_bytes;
    use jolt_source::FileId;

    use jolt_ast::{BinaryOp, Expr, FnBody, Item};

    #[test]
    fn session_runs_toy_query() {
        let mut session = Session::new();
        let file = hash_bytes(b"fn main()");
        let v = session
            .engine_mut()
            .query(INPUT_BYTES, file, &[file], |_| 7u64);
        assert_eq!(v, 7);
    }

    #[test]
    fn all_stages_wired() {
        assert_eq!(stage_names().len(), 18);
    }

    #[test]
    fn lex_cache_hit() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "1 + 2;";
        session.lex_file(file, src);
        let count = session.engine_mut().compute_count(LEX_FILE);
        session.lex_file(file, src);
        assert_eq!(session.engine_mut().compute_count(LEX_FILE), count);
    }

    #[test]
    fn lex_invalidates_on_input_change() {
        let mut session = Session::new();
        let file = FileId(1);
        let h = hash_bytes(b"a");
        session.lex_file(file, "a");
        let before = session.engine_mut().compute_count(LEX_FILE);
        session.engine_mut().invalidate_input(h);
        session.lex_file(file, "a");
        assert!(session.engine_mut().compute_count(LEX_FILE) > before);
    }

    #[test]
    fn parse_double_tour_snippet() {
        let mut session = Session::new();
        let r = session.parse_file(FileId(1), "@double(x: Int) Int -> x * 2 ;;");
        assert!(r.is_ok(), "{:?}", r.errors);
        match &r.program.items[0] {
            Item::Fn(f) => match &f.body {
                FnBody::Expr(Expr::Binary(op, _, _)) => assert_eq!(op.value, BinaryOp::Mul),
                b => panic!("unexpected body {b:?}"),
            },
        }
    }

    #[test]
    fn parse_cache_hit() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "@main() None -> 0 ;;";
        session.parse_file(file, src);
        let count = session.engine_mut().compute_count(PARSE_FILE);
        session.parse_file(file, src);
        assert_eq!(session.engine_mut().compute_count(PARSE_FILE), count);
    }

    #[test]
    fn parse_invalidates_on_input_change() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "@main() None -> 0 ;;";
        let h = hash_bytes(src.as_bytes());
        session.parse_file(file, src);
        let before = session.engine_mut().compute_count(PARSE_FILE);
        session.engine_mut().invalidate_input(h);
        session.parse_file(file, src);
        assert!(session.engine_mut().compute_count(PARSE_FILE) > before);
    }

    #[test]
    fn resolve_accepts_bindings_jolt() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/ui/tiny");
        let path = root.join("bindings.jolt");
        let src = std::fs::read_to_string(&path).expect("read bindings.jolt");
        let mut session = Session::new();
        let r = session.resolve_file(FileId(1), &src);
        assert!(r.is_ok(), "{:?}", r.errors);
    }

    #[test]
    fn resolve_rejects_immutable_reassign() {
        let mut session = Session::new();
        let r = session.resolve_file(FileId(1), "@m() None -> $x = 1; x = 2; ;;");
        assert!(!r.is_ok());
        assert!(r
            .errors
            .iter()
            .any(|e| e.kind == ResolveErrorKind::ImmutableAssign));
    }

    #[test]
    fn resolve_cache_hit() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "@main() None -> 0 ;;";
        session.resolve_file(file, src);
        let count = session.engine_mut().compute_count(RESOLVE_FILE);
        session.resolve_file(file, src);
        assert_eq!(session.engine_mut().compute_count(RESOLVE_FILE), count);
    }

    #[test]
    fn resolve_invalidates_on_input_change() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "@main() None -> 0 ;;";
        let h = hash_bytes(src.as_bytes());
        session.resolve_file(file, src);
        let before = session.engine_mut().compute_count(RESOLVE_FILE);
        session.engine_mut().invalidate_input(h);
        session.resolve_file(file, src);
        assert!(session.engine_mut().compute_count(RESOLVE_FILE) > before);
    }

    #[test]
    fn resolve_depends_on_parse() {
        let mut session = Session::new();
        let file = FileId(1);
        session.resolve_file(file, "@main() None -> 1 ;;");
        let parse_before = session.engine_mut().compute_count(PARSE_FILE);
        let resolve_before = session.engine_mut().compute_count(RESOLVE_FILE);
        session.resolve_file(file, "@main() None -> 2 ;;");
        assert!(session.engine_mut().compute_count(PARSE_FILE) > parse_before);
        assert!(session.engine_mut().compute_count(RESOLVE_FILE) > resolve_before);
    }

    #[test]
    fn check_accepts_tiny_corpus() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/ui/tiny");
        let mut session = Session::new();
        for entry in std::fs::read_dir(&root).expect("tiny dir") {
            let path = entry.expect("entry").path();
            if path.extension().is_some_and(|e| e == "jolt") {
                let src = std::fs::read_to_string(&path).expect("read");
                let name = path.file_name().unwrap().to_string_lossy();
                let r = session.check_file(FileId(1), &src);
                assert!(r.is_ok(), "{name}: {:?}", r.diagnostics.items);
            }
        }
    }

    #[test]
    fn check_rejects_type_mismatch() {
        let mut session = Session::new();
        let r = session.check_file(FileId(1), "@f() Int -> true ;;");
        assert!(!r.is_ok());
        assert!(r
            .diagnostics
            .items
            .iter()
            .any(|d| { d.code.as_deref() == Some(TypeErrorKind::Mismatch.code()) }));
    }

    #[test]
    fn check_cache_hit() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "@main() None -> 0 ;;";
        session.check_file(file, src);
        let count = session.engine_mut().compute_count(CHECK_FILE);
        session.check_file(file, src);
        assert_eq!(session.engine_mut().compute_count(CHECK_FILE), count);
    }

    #[test]
    fn check_invalidates_on_input_change() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "@main() None -> 0 ;;";
        let h = hash_bytes(src.as_bytes());
        session.check_file(file, src);
        let before = session.engine_mut().compute_count(CHECK_FILE);
        session.engine_mut().invalidate_input(h);
        session.check_file(file, src);
        assert!(session.engine_mut().compute_count(CHECK_FILE) > before);
    }

    #[test]
    fn check_depends_on_resolve() {
        let mut session = Session::new();
        let file = FileId(1);
        session.check_file(file, "@main() None -> 1 ;;");
        let resolve_before = session.engine_mut().compute_count(RESOLVE_FILE);
        let check_before = session.engine_mut().compute_count(CHECK_FILE);
        session.check_file(file, "@main() None -> 2 ;;");
        assert!(session.engine_mut().compute_count(RESOLVE_FILE) > resolve_before);
        assert!(session.engine_mut().compute_count(CHECK_FILE) > check_before);
    }

    #[test]
    fn custody_cache_hit() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "@f() None -> $a = \"x\"; println(a); ;;";
        session.custody_file(file, src);
        let count = session.engine_mut().compute_count(CUSTODY_FILE);
        session.custody_file(file, src);
        assert_eq!(session.engine_mut().compute_count(CUSTODY_FILE), count);
    }

    #[test]
    fn custody_depends_on_check() {
        let mut session = Session::new();
        let file = FileId(1);
        session.custody_file(file, "@f() None -> 0 ;;");
        let check_before = session.engine_mut().compute_count(CHECK_FILE);
        let custody_before = session.engine_mut().compute_count(CUSTODY_FILE);
        session.custody_file(file, "@f() None -> 1 ;;");
        assert!(session.engine_mut().compute_count(CHECK_FILE) > check_before);
        assert!(session.engine_mut().compute_count(CUSTODY_FILE) > custody_before);
    }

    #[test]
    fn format_accepts_tiny_corpus_idempotent() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/ui/tiny");
        let mut session = Session::new();
        for entry in std::fs::read_dir(&root).expect("tiny dir") {
            let path = entry.expect("entry").path();
            if path.extension().is_some_and(|e| e == "jolt") {
                let src = std::fs::read_to_string(&path).expect("read");
                let name = path.file_name().unwrap().to_string_lossy();
                let once = session.format_file(FileId(1), &src);
                assert!(once.is_ok(), "{name}: {:?}", once.errors);
                let twice = session.format_file(FileId(1), &once.source);
                assert_eq!(once.source, twice.source, "{name}");
            }
        }
    }

    #[test]
    fn format_cache_hit() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "@main() None -> 0 ;;";
        session.format_file(file, src);
        let count = session.engine_mut().compute_count(FMT_FILE);
        session.format_file(file, src);
        assert_eq!(session.engine_mut().compute_count(FMT_FILE), count);
    }

    #[test]
    fn format_invalidates_on_input_change() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "@main() None -> 0 ;;";
        let h = hash_bytes(src.as_bytes());
        session.format_file(file, src);
        let before = session.engine_mut().compute_count(FMT_FILE);
        session.engine_mut().invalidate_input(h);
        session.format_file(file, src);
        assert!(session.engine_mut().compute_count(FMT_FILE) > before);
    }

    #[test]
    fn incremental_cache_hit_without_invalidation() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "@a() Int -> 1 ;; @b() Int -> 2 ;;";
        session.check_source(file, src);
        let parse_before = session.engine_mut().compute_count(PARSE_FILE);
        let check_before = session.engine_mut().compute_count(CHECK_FILE);
        let report = session.check_source(file, src);
        assert!(report.is_ok());
        assert_eq!(session.engine_mut().compute_count(PARSE_FILE), parse_before);
        assert_eq!(session.engine_mut().compute_count(CHECK_FILE), check_before);
    }

    #[test]
    fn incremental_comment_edit_reprunes_at_file_level() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "// comment only\n@a() Int -> 1 ;; @b() Int -> 2 ;;";
        session.check_source(file, src);
        let parse_before = session.engine_mut().compute_count(PARSE_FILE);
        let check_before = session.engine_mut().compute_count(CHECK_FILE);
        session
            .engine_mut()
            .invalidate_input(hash_bytes(src.as_bytes()));
        let report = session.check_source(file, src);
        assert!(report.is_ok(), "{:?}", report.render_diagnostics(src));
        assert!(session.engine_mut().compute_count(PARSE_FILE) > parse_before);
        assert!(
            session.engine_mut().compute_count(CHECK_FILE) > check_before,
            "after invalidation, file-level queries re-run (per-fn queries deferred)"
        );
    }

    #[test]
    fn incremental_body_edit_rechecks() {
        let mut session = Session::new();
        let file = FileId(1);
        let before = "@a() Int -> 1 ;; @b() Int -> 2 ;;";
        let after = "@a() Int -> 1 ;; @b() Int -> 3 ;;";
        session.check_source(file, before);
        let check_before = session.engine_mut().compute_count(CHECK_FILE);
        session
            .engine_mut()
            .invalidate_input(hash_bytes(after.as_bytes()));
        let report = session.check_source(file, after);
        assert!(report.is_ok());
        assert!(
            session.engine_mut().compute_count(CHECK_FILE) > check_before,
            "semantic edit must re-run type check at file level"
        );
    }

    #[test]
    fn run_hello_world() {
        let mut session = Session::new();
        let src = "@main() None -> println(\"Hello, Jolt!\"); ;;";
        let report = session.run_source(FileId(1), src);
        assert!(report.is_ok(), "{:?}", report.render_all_errors(src));
        assert_eq!(report.stdout(), Some("Hello, Jolt!\n"));
    }

    #[test]
    fn mir_cache_hit() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "@main() None -> println(\"x\"); ;;";
        session.mir_file(file, src);
        let count = session.engine_mut().compute_count(MIR_FILE);
        session.mir_file(file, src);
        assert_eq!(session.engine_mut().compute_count(MIR_FILE), count);
    }

    #[test]
    fn mir_depends_on_custody() {
        let mut session = Session::new();
        let file = FileId(1);
        session.mir_file(file, "@main() None -> println(\"a\"); ;;");
        let custody_before = session.engine_mut().compute_count(CUSTODY_FILE);
        let mir_before = session.engine_mut().compute_count(MIR_FILE);
        session.mir_file(file, "@main() None -> println(\"b\"); ;;");
        assert!(session.engine_mut().compute_count(CUSTODY_FILE) > custody_before);
        assert!(session.engine_mut().compute_count(MIR_FILE) > mir_before);
    }

    #[test]
    fn run_cache_hit() {
        let mut session = Session::new();
        let file = FileId(1);
        let src = "@main() None -> println(\"x\"); ;;";
        session.run_file(file, src);
        let count = session.engine_mut().compute_count(RUN_FILE);
        session.run_file(file, src);
        assert_eq!(session.engine_mut().compute_count(RUN_FILE), count);
    }

    #[test]
    fn run_skipped_when_custody_fails() {
        let mut session = Session::new();
        let src = "@main() None -> $a = \"x\"; $b = a; println(a); ;;";
        let report = session.run_source(FileId(1), src);
        assert!(!report.custody.is_ok());
        assert!(!report.run.is_ok());
        assert!(!report.is_ok());
    }

    #[test]
    fn test_passing_corpus() {
        let src = r#"
[test]
@adds_two_and_three() None ->
    assert_eq(2 + 3, 5);
;;

[test]
@bool_ok() None ->
    assert_eq(true, true);
;;
"#;
        let mut session = Session::new();
        let report = session.test_source(FileId(1), src);
        assert!(report.parse.is_ok(), "{:?}", report.parse.errors);
        assert!(
            report.check.diagnostics.is_empty(),
            "{:?}",
            report.check.diagnostics
        );
        assert!(
            !report.report().cases.is_empty(),
            "expected tests, mir={:?} test={:?}",
            report.mir.diagnostics,
            report.test
        );
        assert!(report.is_ok(), "{:?}", report.report());
    }
}
