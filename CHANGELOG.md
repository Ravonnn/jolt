# Changelog

All notable changes to the Jolt compiler and repository are documented here.

## Unreleased

### Added

- Phase 0 monorepo scaffold: Rust workspace, stage-0 compiler crates, query engine skeleton,
  `jolt` CLI (`--version`), test corpora layout, CI, design docs under `docs/`.
- Phase 1a Tiny lexer: hand-rolled `jolt-lexer` with max-munch rules, `lex` / `lex_file` query,
  `Session::lex_file` in `libjolt`.
- Phase 1b Tiny parser + AST: `jolt-ast`, recursive-descent `jolt-parser`, `PARSE_FILE` query,
  `Session::parse_file`, `tests/ui/tiny/` accept corpus.
- Phase 1c Tiny resolver: `jolt-resolve` scopes and `$`/`$$`/reassign rules, `RESOLVE_FILE` query,
  `Session::resolve_file`, `tests/ui/tiny-reject/` corpus.
- Phase 1d Tiny type checker: `jolt-diagnostics`, `jolt-types`, `CHECK_FILE` query, `Session::check_file`,
  `jolt check <file>`, `tests/ui/type-reject/` stderr snapshots.
- Phase 1e Tiny formatter: `compiler/jolt-fmt`, `FMT_FILE` query, `Session::format_file`, `jolt fmt`,
  idempotence tests over `tests/ui/tiny/`.
- Phase 1f front-end gate prep: `CheckReport` / `Session::check_source`, `jolt check` on directories,
  incremental query smoke tests, `jolt-harness` property/corpus runners, expanded UI corpus (12 accept,
  9 type-reject snapshots).
- Phase 2a Custodian move analysis: `jolt-custody`, `CUSTODY_FILE` query, use-after-move for `String`,
  `tests/custody` accept/reject corpus with stderr snapshots.
- Phase 2b Custodian borrows: `borrow`/`claim`/`deref` typing, shared-XOR-mutable, non-lexical
  borrow release, expanded `tests/custody` corpus.
- Phase 2c Custodian gate completion: diagnostic hints (`; hint: …`), double-claim and
  move-while-borrowed coverage, custody corpus ≥10 accept / ≥8 reject, ergonomics report,
  harness minimum counts and `UPDATE_UI` for custody snapshots.
- Phase 3b run corpus + control flow: `BranchIf`/`Jump`, `if`/`loop`/`for`/`break`, user fn calls,
  `jolt test` + `[test]` discovery, `tests/run` ≥5 stdout cases, `tests/test` corpus.
- Phase 3c Phase 3 gate + preview: `tests/tutorial` §1–§3 Tiny programs, `[test, should_fail]`,
  `xtask pipeline-smoke`, [`jolt-0.1-preview.md`](docs/design/jolt-0.1-preview.md).
- **Jolt Learn platform:** `learn/` curriculum manifest, interactive hub, tour modules, 16 Jolt by
  Example pages, 5 “all ways” guides, migration guides, Custodian quizzes, `jolt-learn-runner`
  local sidecar, `cargo xtask learn serve|verify`, `joltlings` exercises (15), GitHub Pages docs
  workflow with Pagefind search.
- mdBook setup for `docs/` (`book.toml`, `SUMMARY.md`, `cargo xtask docs build|serve`), CI doc build,
  and [`docs/design/compiler-status.md`](docs/design/compiler-status.md) living status page.
