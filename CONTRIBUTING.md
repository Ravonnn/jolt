# Contributing to Jolt

Thank you for contributing to the Jolt compiler and toolchain.

## Development setup

1. Install Rust via [rustup](https://rustup.rs/) (the repo pins `1.85.0` in `rust-toolchain.toml`).
2. Clone the repository and run:

```bash
cargo build --workspace
cargo test --workspace
```

## Coding standards

- **Edition:** Rust 2021 for all workspace crates.
- **Format:** `cargo fmt --all` before submitting changes.
- **Lint:** `cargo clippy --workspace -- -D warnings` must pass.
- **Dependencies:** Keep new dependencies minimal; justify additions in PR descriptions.
- **API surface:** Tools and the CLI must depend on `libjolt` only, not individual stage crates.

## Pull requests

1. Branch from `main` (or the default branch).
2. Keep changes focused; Phase-scoped work is easier to review.
3. Ensure CI passes: build, test, `fmt --check`, clippy.
4. Update `CHANGELOG.md` under **Unreleased** for user-visible changes.

## Tests

### Unit and integration tests

Run with `cargo test --workspace`. Compiler stage crates should include at least one test where
behavior matters.

### Test corpora

Corpora live under `tests/`:

| Directory | Purpose | Layout |
| --------- | ------- | ------ |
| `tests/ui/` | Parse/type/diagnostic checks | `case.jolt` + `case.stderr` (expected diagnostics) |
| `tests/run/` | Program output | `program.jolt` + `program.stdout` |
| `tests/tutorial/` | Tour §1–§3 (Tiny) | `*.jolt` + `*.stdout` |
| `tests/test/` | `[test]` runner | `*.jolt` with test functions |
| `tests/custody/` | Custodian accept/reject | `should_accept/*.jolt`, `should_reject/*.jolt` |

**Adding a UI test:** create `tests/ui/my_case.jolt` and `tests/ui/my_case.stderr` with the expected
error output (one line per diagnostic after Phase 1).

**Adding a run test:** create `tests/run/my_case.jolt` and `tests/run/my_case.stdout`.

**Adding a custody test:** place sources under `tests/custody/should_accept/` or `should_reject/`.

### Running corpus smoke tests

```bash
cargo test -p jolt-harness
# or
cargo xtask test-corpora
cargo run -p xtask -- pipeline-smoke   # jolt-cli end-to-end: run, test, check
cargo run -p xtask -- learn verify     # curriculum manifest + snippet paths
cargo xtask learn serve --open         # interactive docs + local runner
```

## Documentation

The design corpus lives under [`docs/`](docs/). It is built with [mdBook](https://rust-lang.github.io/mdBook/):

```bash
cargo install mdbook --locked   # once
mdbook build docs               # or: cargo xtask docs build
mdbook serve docs --open        # local preview
```

When adding a page, update [`docs/SUMMARY.md`](docs/SUMMARY.md) so it appears in the book sidebar.
CI verifies the book builds on every PR.

## Architecture Decision Records

New implementation decisions that extend or refine the design corpus should be recorded in
`docs/adr/` using the template in `docs/adr/0000-template.md`.

## Questions

Open a discussion issue or refer to [docs/index.md](docs/index.md) for the design corpus.
