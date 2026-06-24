# Test corpus

`[test]` functions discovered and run via `jolt test`.

## Layout

```
tests/test/
  passing.jolt
  should_fail.jolt
```

## Commands

```bash
cargo build -p jolt-cli
./target/debug/jolt test tests/test/
cargo test -p jolt-harness test_corpus
```

**`should_fail`:** `[test, should_fail]` expects a runtime failure (e.g. failing `assert_eq`).

**Known limits (3c):** no `should_fail: "msg"` substring match; no `skip`/`only`; no doctests.
