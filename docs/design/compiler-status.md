# Compiler implementation status

> Living document for the **stage-0 Rust compiler**. Updated as phases land. Gates and objectives
> remain authoritative in [`year-1.md`](year-1.md).

Last updated: **Phase 3c** (Phase 3 gate + Jolt 0.1 preview).

---

## At a glance

| Phase | Scope | Status |
| ----- | ----- | ------ |
| **0** | Monorepo scaffold, query engine skeleton, CI, test harness | **Done** |
| **1a** | Tiny lexer (`jolt-lexer`, `LEX_FILE`) | **Done** |
| **1b** | Tiny parser + AST (`jolt-parser`, `PARSE_FILE`) | **Done** |
| **1c** | Tiny name resolver (`jolt-resolve`, `RESOLVE_FILE`) | **Done** |
| **1d** | Tiny type checker + diagnostics (`jolt-types`, `CHECK_FILE`) | **Done** |
| **1e** | `jolt fmt` (parse → print, idempotent) | **Done** |
| **1f** | Gate prep: incremental smoke, `check` dirs, property tests, corpus growth | **Done** |
| **2a** | Custodian move analysis (`jolt-custody`, `CUSTODY_FILE`, use-after-move) | **Done** |
| **2b** | Custodian borrows (`borrow`/`claim`/`deref`, shared-XOR-mutable, NLL) | **Done** |
| **2c** | Custodian gate: hints, corpus scale, ergonomics report | **Done** |
| **3a** | MIR lowering + interpreter + `jolt run --interpret` | **Done** |
| **3b** | Run corpus, control-flow MIR, `jolt test` | **Done** |
| **3c** | Tutorial corpus, `should_fail`, pipeline CI smoke, preview doc | **Done** |
| **4a** | Core structs (Year 2) | Not started |

---

## What works today

### Workspace & tooling

- Rust workspace with 17 stage crates + `libjolt` umbrella + `jolt-cli`
- `cargo build/test/clippy/fmt` green on CI
- Query engine (`jolt-query`): memoization, dependency tracking, input invalidation (unit tested)
- Test corpora: `tests/ui`, `tests/run`, `tests/custody` with harness smoke tests

### CLI

```bash
cargo build -p jolt-cli
./target/debug/jolt --version   # works
./target/debug/jolt check path/to/file.jolt   # or: jolt check tests/ui/tiny/
./target/debug/jolt fmt path/to/file.jolt     # print formatted source (use --write to overwrite)
./target/debug/jolt run --interpret tests/run/hello.jolt
./target/debug/jolt test tests/test/passing.jolt
./target/debug/jolt run         # stub without --interpret (exit 2)
```

Tools depend on **`libjolt` only**, not individual stage crates.

### Front end (Tiny)

| Crate | Query | `Session` API | Notes |
| ----- | ----- | ------------- | ----- |
| `jolt-lexer` | `LEX_FILE` | `lex_file` | Hand-rolled max-munch lexer |
| `jolt-parser` | `PARSE_FILE` | `parse_file` | Recursive descent, error recovery |
| `jolt-resolve` | `RESOLVE_FILE` | `resolve_file` | Scopes, `$` / `$$`, reassign rules |
| `jolt-types` | `CHECK_FILE` | `check_file` | `Int`/`Bool`/`None`/`String`, `borrow`/`claim`/`deref` |
| `jolt-custody` | `CUSTODY_FILE` | `custody_file` | Moves, shared-XOR-mutable, NLL; hints on `E0401`–`E0404` |
| `jolt-mir` | `MIR_FILE` | `mir_file` | Lower Tiny AST to MIR; control flow |
| `jolt-backend-interp` | `RUN_FILE` | `run_file` | Interpret MIR; builtins + user calls |
| `jolt-test-runner` | `TEST_FILE` | `test_file` | Discover/run `[test]` functions |
| `jolt-fmt` | `FMT_FILE` | `format_file` | Canonical parse → print (4-space indent) |
| `jolt-diagnostics` | — | (via check) | Stable `line:col: error:` + optional `; hint: …` |

**Tiny surface:** `@fn` declarations, params, `Int`/`Bool`/`None` type names (unresolved), `$`/`$$`
bindings, bare reassignment, `if`/`else`, `loop`, `for x in xs`, blocks with tail expressions,
operators, calls, `println`, `borrow`/`claim`/`deref`.

### Test corpora

| Corpus | Path | Purpose |
| ------ | ---- | ------- |
| Accept | `tests/ui/tiny/*.jolt` (12 files) | Parse + resolve + type-check with zero errors |
| Resolve reject | `tests/ui/tiny-reject/*.jolt` (3 files) | Expected `ResolveErrorKind` |
| Type reject | `tests/ui/type-reject/*.jolt` (9 files) | Expected `.stderr` snapshots |
| Custody accept | `tests/custody/should_accept/*.jolt` (10 files) | Full pipeline including custody |
| Custody reject | `tests/custody/should_reject/*.jolt` (8 files) | Custody `.stderr` snapshots with hints |
| Run | `tests/run/*.jolt` + `.stdout` (5 files) | Interpreter stdout snapshots |
| Tutorial | `tests/tutorial/*.jolt` + `.stdout` (3 files) | Tour §1–§3 Tiny adaptations |
| Test | `tests/test/*.jolt` (2 files) | `[test]` + `should_fail` harness |

Run:

```bash
cargo test -p jolt-parser ui_tiny_corpus_accepts
cargo test -p jolt-resolve ui_tiny_corpus_accepts ui_tiny_reject_corpus
cargo test -p jolt-types ui_tiny_corpus_accepts ui_type_reject_corpus
cargo test -p jolt-fmt ui_tiny_corpus_idempotent
cargo test -p jolt-harness property_tests
cargo test -p jolt-custody
cargo test -p jolt-mir
cargo test -p jolt-backend-interp
cargo test -p jolt-harness run_corpus
cargo test -p jolt-harness test_corpus
cargo test -p jolt-harness tutorial_corpus
cargo run -p xtask -- pipeline-smoke
cargo test -p libjolt
```

---

## Query pipeline (file-level)

Each stage memoizes on source hash and records dependencies:

```
source bytes
    └─► LEX_FILE      (jolt-lexer)
            └─► PARSE_FILE   (jolt-parser)
                    └─► RESOLVE_FILE  (jolt-resolve)
                            └─► CHECK_FILE  (jolt-types)
                                    └─► CUSTODY_FILE (jolt-custody)
                                            └─► MIR_FILE     (jolt-mir)
                                                    └─► RUN_FILE (jolt-backend-interp)
                                                    └─► TEST_FILE (jolt-test-runner)
                                            └─► FMT_FILE   (jolt-fmt — parse-only dependency)
```

Changing source invalidates downstream queries. Integration tests in `libjolt` assert cache hits
and invalidation for lex, parse, and resolve.

Per-function incremental queries (`RESOLVE_FN`, etc.) are **deferred**; file-level memoization is
in place. Phase 1f added smoke tests: cache hit on unchanged source, re-run after invalidation
(comment-only or semantic edits re-check at file level today).

---

## Next milestones

See [`year-1.md`](year-1.md) Phase 1 Definition of Done for the full gate.

1. **Year 2 / Phase 4a — Core structs:** first struct decl + field access slice.
2. **Phase 2 verification gate:** independent custody review (process).

**Year-1 Phase 3 gate:** complete — see [`jolt-0.1-preview.md`](jolt-0.1-preview.md).

---

## Related docs

- [`jolt-implementation-plan.md`](jolt-implementation-plan.md) — milestone exit tests (Phase A–D)
- [`00-repo-structure.md`](00-repo-structure.md) — crate layout
- [`jolt-caching-system.md`](jolt-caching-system.md) — query engine design
- [`custodian-ergonomics.md`](custodian-ergonomics.md) — Phase 2c ergonomics evaluation
- [`../spec/jolt-grammar.md`](../spec/jolt-grammar.md) — Tiny grammar reference
