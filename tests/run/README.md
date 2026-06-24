# Run tests

End-to-end program execution: each case is a `.jolt` program and a `.stdout` file with expected
output.

## Layout

```
tests/run/
  hello.jolt / hello.stdout
  arithmetic.jolt / arithmetic.stdout
  if_else.jolt / if_else.stdout
  loop_break.jolt / loop_break.stdout
  for_sum.jolt / for_sum.stdout
```

Sample smoke files live under `tests/run/sample/`.

## Commands

```bash
cargo build -p jolt-cli
./target/debug/jolt run --interpret tests/run/if_else.jolt
cargo test -p jolt-harness run_corpus
cargo test -p libjolt run_hello_world
```

Refresh stdout snapshots:

```bash
UPDATE_UI=1 cargo test -p jolt-harness run_corpus
```

**Known limits (3b):** `for x in n` treats `n: Int` as `0..n-1`; `break`/`next` require `;` before
`;;`; no native codegen; tutorial §1–§3 not yet runnable as-written.
