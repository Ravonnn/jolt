## Learned User Preferences

- When implementing an attached Cursor plan, do not edit the plan file itself.
- Plan todos are pre-created; update their status with TodoWrite instead of recreating todos.
- Chose MIT-only license for the project (not dual MIT/Apache).
- Driver CLI and tools crates must depend on `libjolt` only, not individual compiler stage crates.
- Verify compiler work with `cargo test --workspace` (and `cargo clippy` / `cargo fmt` per phase gates) before claiming completion.
- When completing a compiler phase, update `CHANGELOG.md` and `docs/design/compiler-status.md`.

## Learned Workspace Facts

- Jolt stage-0 compiler is Rust; language rollout follows Tiny → Core → Full per design docs.
- Original design corpus is under `wiki/`; implementation and agents should read `docs/` (populated from wiki in Phase 0). Preview with `mdbook serve docs --open` or `cargo xtask docs serve --open`.
- Monorepo layout follows `docs/design/00-repo-structure.md`: `compiler/*` stage crates, `libjolt` umbrella, `driver/jolt-cli`, `xtask`, `tests/ui|run|custody`.
- Workspace Rust edition is `2021` (set in root `Cargo.toml`); distinct from Jolt language edition `2026` in future tooling.
- Living status: `docs/design/compiler-status.md`; phase completions recorded in `CHANGELOG.md`.
- mdBook configured for `docs/` (`book.toml`, `SUMMARY.md`); output `docs/book/` is gitignored; CI runs `mdbook build docs`.
- Phase 0 complete: workspace scaffold, `QueryEngine` (memoization, invalidation, early cutoff), MIT `LICENSE`, pinned toolchain in `rust-toolchain.toml`.
- Phase 1a–1f complete (Tiny front end): `LEX_FILE` → `PARSE_FILE` → `RESOLVE_FILE` → `CHECK_FILE` → `FMT_FILE`; `Session` APIs in `libjolt`; `jolt check` (files/dirs), `jolt fmt`; UI corpora under `tests/ui/tiny`, `tiny-reject`, `type-reject`; `jolt-harness` property runners.
- Phase 2a–2b complete (Custodian Tiny): `jolt-custody`, `CUSTODY_FILE` query; move analysis plus `borrow`/`claim`/`deref` with non-lexical borrow release; `tests/custody` corpus with stderr snapshots.
- `jolt-lexer` depends only on `jolt-source` and `jolt-query`.
