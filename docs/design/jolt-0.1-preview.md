# Jolt 0.1 (preview)

> **Tiny interpreter MVP** — Year-1 Phase 3 complete. Safe-by-default front end + Custodian + MIR
> interpreter for a minimal language slice.

## What works

- **Pipeline:** `lex → parse → resolve → type-check → custody → MIR → interpret`
- **CLI:** `jolt check`, `jolt fmt`, `jolt run --interpret`, `jolt test`
- **Language (Tiny):** `@fn`, `$`/`$$` bindings, `Int`/`Bool`/`String`/`None`, operators, `if`/`loop`/`for`,
  `borrow`/`claim`/`deref`, `[test]` / `[test, should_fail]`
- **Corpora:** `tests/ui`, `tests/custody`, `tests/run` (5 programs), `tests/tutorial` (§1–§3),
  `tests/test`

```bash
cargo build -p jolt-cli
./target/debug/jolt run --interpret tests/run/hello.jolt
./target/debug/jolt test tests/test/
./target/debug/jolt check tests/ui/tiny/
cargo run -p xtask -- pipeline-smoke
```

## What does not work yet

- Structs, enums, unions, generics, `match`, `!T` / `Result` / `?`
- `using` imports, string `{interpolation}`, default/named arguments
- Native codegen (`jolt run` without `--interpret` is a stub)
- Package manager, capabilities, comptime, REPL, LSP depth
- `jolt test` modifiers beyond `should_fail` (`skip`, `only`, doctests, parallel runner)

See [`year-2.md`](year-2.md) for Core language growth (Phase 4+).

## Tag (process)

When ready to mark the preview release:

```bash
git tag -a v0.1.0-preview -m "Jolt 0.1 preview: Tiny interpreter MVP"
git push origin v0.1.0-preview
```

## Related

- [`compiler-status.md`](compiler-status.md) — living implementation status
- [`year-1.md`](year-1.md) — Phase 3 verification gate
- [`tests/tutorial/README.md`](../../tests/tutorial/README.md) — runnable tour §1–§3 (Tiny)
