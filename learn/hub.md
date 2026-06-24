# Learn Jolt

Welcome to **Jolt Learn** — a beginner-friendly, locally-runnable tutorial for the Jolt
programming language. Every runnable example executes on **your machine** using the real `jolt`
compiler (not a cloud sandbox).

## Who is this for?

- **Complete beginners** with some programming experience — start the [Guided Tour](tour/01-hello.md)
- **Hands-on learners** — browse [Jolt by Example](examples/index.md)
- **Coming from another language** — read a [migration guide](migrations/from-rust.md)
- **Practice-oriented learners** — try [Joltlings](../../tools/joltlings/README.md) exercises

## Quick start

```bash
git clone https://github.com/jolt-lang/jolt.git
cd jolt
cargo build -p jolt-cli
./target/debug/jolt run --interpret tests/tutorial/hello.jolt
```

### Interactive mode (Run button in the browser)

```bash
cargo xtask learn serve --open
```

This starts the documentation site **and** a local runner on `http://127.0.0.1:3847` so you can
edit and run Jolt code directly in lesson pages.

## Choose your path

<div class="path-card">

### Complete beginner

Linear narrative from Hello World through the Custodian, control flow, and testing.

**Start:** [Tour §1 — Hello, Jolt](tour/01-hello.md)

</div>

<div class="path-card">

### Learn by example

Short, focused programs — one concept per page. Great for lookup and experimentation.

**Start:** [Jolt by Example](examples/index.md)

</div>

<div class="path-card">

### All ways to do X

Comparison guides when multiple idioms are valid — tables, when-to-use, and anti-patterns.

**Start:** [Ways to bind values](guides/ways-to-bind.md)

</div>

<div class="path-card">

### Exercises

Fix broken programs, pass custody checks, and build muscle memory with `joltlings watch`.

**Start:** [Joltlings](../../tools/joltlings/README.md)

</div>

## Edition badges

Lessons are tagged by what the compiler supports **today**:

| Badge | Meaning |
| ----- | ------- |
| <span class="edition-badge tiny">tiny</span> | Runs with `jolt run --interpret` now |
| <span class="edition-badge core">core</span> | Structs/enums (Year 2+) |
| <span class="edition-badge full">full</span> | Spec narrative; not runnable yet |

The full [language spec](../spec/jolt-spec-v0.4.md) describes the target language; Tiny is the
current interpreter subset. See [Jolt 0.1 preview](../design/jolt-0.1-preview.md).

## Reference ladder

1. **Learn** (you are here) — tutorials and examples
2. [Cheatsheet](../spec/jolt-cheatsheet.md) — quick syntax lookup
3. [Language spec](../spec/jolt-spec-v0.4.md) — authoritative reference
4. [Compiler status](../design/compiler-status.md) — what is implemented
