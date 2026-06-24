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

```jolt runnable
@double(x: Int) Int -> x * 2 ;;

@main() None ->
    $n = double(21);
    if n == 42 ->
        println("42");
    ;;
;;
```

## If as value

```jolt runnable
@max(a: Int, b: Int) Int ->
    if a > b ->
        a
    ;; else ->
        b
    ;;
;;

@main() None ->
    $m = max(3, 7);
    if m == 7 ->
        println("7");
    ;;
;;
```

## When to use which

- **Single expression:** tail form (idiomatic Jolt)
- **Multiple steps:** block with tail expression on last line
- **Early exit (future):** explicit `return` when Core adds statement-level control

## Anti-patterns

- Adding unnecessary blocks when a single tail expression suffices

## See also

- [Tour §3](../tour/03-functions.md)
- [function-tail-expr example](../examples/function-tail-expr.md)
