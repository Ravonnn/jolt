# All ways to return values

In Jolt, function bodies are expressions. Tiny uses tail expressions; explicit `return` is planned
for Core.

## Comparison

| Style | Tiny? | Example |
| ----- | ----- | ------- |
| Tail expression | Yes | `@f() Int -> 42 ;;` |
| Tail call | Yes | `@f() Int -> g() ;;` |
| Block tail | Yes | `@f() Int -> { $x = 1; x + 1 } ;;` |
| `if` as value | Yes | `@max(a,b) Int -> if a > b -> a ;; else -> b ;;` |
| Explicit `return` | Future | `return 42;` in statement blocks |

## Tail expression

{{#snippet tutorial/double}}

## If as value

{{#snippet tutorial/if_else}}

## When to use which

- **Single expression:** tail form (idiomatic Jolt)
- **Multiple steps:** block with tail expression on last line
- **Early exit (future):** explicit `return` when Core adds statement-level control

## Anti-patterns

- Adding unnecessary blocks when a single tail expression suffices

## See also

- [Tour §3](../tour/03-functions.md)
- [function-tail-expr example](../examples/function-tail-expr.md)
