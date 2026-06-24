# All ways to iterate

Tiny supports `loop`, `for x in n`, and `if`/`break`. Full Jolt will add range syntax `0..n`.

## Comparison

| Style | Tiny? | Example use |
| ----- | ----- | ----------- |
| `for x in n` | Yes | Known iteration count `0 .. n-1` |
| `loop` + `break` | Yes | Unknown iterations, early exit |
| `0..n` range | Future | Idiomatic in full Jolt |
| `while cond` | Future | Condition-driven loops |

## For loop (Tiny)

{{#snippet tutorial/for_sum}}

`for x in n` sets `x` to `0, 1, …, n-1`.

## Loop + break

{{#snippet tutorial/loop_break}}

## When to use which

- **Counted loop:** `for x in n` in Tiny; `0..n` when available
- **Until condition:** `loop` with `break` inside `if`
- **Infinite loop:** `loop` with internal `break` only (use carefully)

## See also

- [Tour §6](../tour/06-for.md)
- [loop-break example](../examples/loop-break.md)
