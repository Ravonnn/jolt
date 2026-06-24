# Test should_fail <span class="edition-badge tiny">tiny</span>

```bash
./target/debug/jolt test tests/test/should_fail.jolt
```

Expect `ok (expected fail)` for tests that must fail at runtime:

```jolt
[test, should_fail]
@asserts_wrong() None ->
    assert_eq(1, 2);
;;
```

[All ways to test](../guides/ways-to-test.md)
