# Jolt documentation

This directory is the **design corpus** for Jolt: language spec, tutorials, compiler design, and
roadmap. It is published as an [mdBook](https://rust-lang.github.io/mdBook/) site.

## Quick start

Install mdBook (once):

```bash
cargo install mdbook --locked
```

Build the book:

```bash
mdbook build docs
# or
cargo xtask docs build
```

Preview locally (opens http://localhost:3000):

```bash
mdbook serve docs --open
# or
cargo xtask docs serve
```

Output lands in `docs/book/` (gitignored).

## Layout

| Path | Contents |
| ---- | -------- |
| [`index.md`](index.md) | Front door and reading paths |
| [`SUMMARY.md`](SUMMARY.md) | mdBook table of contents |
| [`spec/`](spec/) | Language specification and grammar |
| [`tutorial/`](tutorial/) | Learning material |
| [`design/`](design/) | Compiler, toolchain, and roadmap |
| [`adr/`](adr/) | Architecture Decision Records |

## Editing

1. Edit markdown under `docs/`.
2. Add new pages to [`SUMMARY.md`](SUMMARY.md) so they appear in the sidebar.
3. Run `mdbook build docs` (or `cargo xtask docs build`) before opening a PR.

The legacy [`wiki/`](../wiki/) directory redirects here; prefer `docs/` for new links.
