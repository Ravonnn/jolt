# UI tests

Compile-time and diagnostic tests: each case is a `.jolt` source file paired with expected
stderr output.

## Layout

```
tests/ui/
  my_case.jolt
  my_case.stderr    # expected diagnostic lines (Phase 1+)
```

Sample cases live under `tests/ui/sample/` for harness smoke checks.

**Tiny accept corpus (Phase 1b–1e):** `tests/ui/tiny/*.jolt` — valid Tiny programs that must parse,
resolve, type-check, and format idempotently (`fmt(fmt(x)) == fmt(x)`). Run:

```bash
cargo test -p jolt-parser ui_tiny_corpus_accepts
cargo test -p jolt-resolve ui_tiny_corpus_accepts
cargo test -p jolt-types ui_tiny_corpus_accepts
cargo test -p jolt-fmt ui_tiny_corpus_idempotent
cargo test -p jolt-harness property_tests   # tiny accept + fmt idempotence + type/custody snapshots
```

**Front-end gate prep (Phase 1f):** `jolt check` accepts a directory; `Session::check_source` runs
parse → resolve → type → custody. Harness property tests:

```bash
cargo test -p jolt-harness ui_tiny_corpus_check_clean
cargo test -p jolt-harness fmt_idempotent_on_tiny_corpus
```

**Resolve reject corpus (Phase 1c):** `tests/ui/tiny-reject/*.jolt` — programs that must fail name
resolution with an expected [`ResolveErrorKind`](../../compiler/jolt-resolve/src/error.rs). Run:

```bash
cargo test -p jolt-resolve ui_tiny_reject_corpus
```

**Type reject corpus (Phase 1d):** `tests/ui/type-reject/*.jolt` + `.stderr` — programs that parse
and resolve but fail type checking. Snapshot lines match `jolt-diagnostics` output (`line:col: error: …`).
Run:

```bash
cargo test -p jolt-types ui_type_reject_corpus
```

Refresh snapshots after intentional diagnostic changes:

```bash
UPDATE_UI=1 cargo test -p jolt-types ui_type_reject_corpus
```

CLI check (manual):

```bash
cargo build -p jolt-cli
./target/debug/jolt check tests/ui/tiny/double.jolt
./target/debug/jolt check tests/ui/type-reject/return_type_mismatch.jolt  # exit 1
./target/debug/jolt fmt tests/ui/tiny/double.jolt
./target/debug/jolt fmt --write tests/ui/tiny/double.jolt
```
