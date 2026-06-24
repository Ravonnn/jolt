# Test with assert_eq <span class="edition-badge tiny">tiny</span>

```bash
./target/debug/jolt test tests/test/passing.jolt
```

The source uses `[test]` functions:

```jolt
[test]
@adds_two_and_three() None ->
    assert_eq(2 + 3, 5);
;;
```

[All ways to test](../guides/ways-to-test.md)
