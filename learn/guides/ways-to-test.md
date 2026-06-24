# All ways to test

Jolt testing spans runtime assertions, test functions, and (future) doctests.

## Comparison

| Mechanism | Command | When |
| --------- | ------- | ---- |
| `assert_eq` in `@main` | `jolt run --interpret` | Quick sanity in scripts |
| `[test]` function | `jolt test file.jolt` | Unit tests |
| `[test, should_fail]` | `jolt test` | Expected failure |
| Doctests in `///` | Not yet | Doc examples as tests |
| Directory of tests | `jolt test dir/` | Project test suites |

## assert_eq in main

{{#snippet tutorial/testing}}

## [test] functions

```bash
./target/debug/jolt test tests/test/passing.jolt
```

## should_fail

```bash
./target/debug/jolt test tests/test/should_fail.jolt
```

## When to use which

- **Exploring in the tour:** `assert_eq` in `@main`
- **Real tests:** `[test]` + `jolt test`
- **Negative tests:** `[test, should_fail]`

## See also

- [Tour §11](../tour/11-testing.md)
- [test-assert-eq example](../examples/test-assert-eq.md)
