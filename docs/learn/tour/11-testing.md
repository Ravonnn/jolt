# Tour §11 — Testing <span class="edition-badge tiny">tiny</span>

Jolt has built-in testing via `[test]` functions and `jolt test`. At runtime, `assert_eq` checks
equality and fails the test on mismatch.

This runnable program uses `assert_eq` inside `@main`:

```jolt runnable
@main() None ->
    assert_eq(2 + 2, 4);
    println("tests ok");
;;
```

For real test functions, see [test-assert-eq](../examples/test-assert-eq.md) and
[All ways to test](../guides/ways-to-test.md).

```bash
./target/debug/jolt test tests/test/
```

**Next:** [Jolt by Example](../examples/index.md) · [Learn hub](../hub.md)
