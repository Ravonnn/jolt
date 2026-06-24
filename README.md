# Jolt

Jolt is a general-purpose, statically-typed systems language with memory safety (the Custodian),
structured concurrency, and a unified toolchain. This repository contains the **stage-0 compiler**
(Rust) and the design corpus.

**License:** [MIT](LICENSE)

## Build

Requires Rust 1.85+ (see `rust-toolchain.toml`).

```bash
cargo build --workspace
cargo test --workspace
```

## CLI

```bash
cargo build -p jolt-cli
./target/debug/jolt --version
```

Subcommands (`run`, `fmt`, `test`) are stubs until later phases. Type-check a file:

```bash
./target/debug/jolt check tests/ui/tiny/double.jolt
./target/debug/jolt fmt tests/ui/tiny/double.jolt
./target/debug/jolt fmt --write tests/ui/tiny/double.jolt
```

## Documentation

Design and roadmap: [docs/index.md](docs/index.md)

Preview the full corpus locally with [mdBook](https://rust-lang.github.io/mdBook/):

```bash
cargo install mdbook --locked   # once
mdbook serve docs --open        # or: cargo xtask docs serve --open
```

See [docs/README.md](docs/README.md) for layout and editing notes.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
