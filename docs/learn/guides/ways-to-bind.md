# All ways to bind values

Jolt offers several ways to introduce names. This guide compares them.

## Comparison

| Approach | Syntax | Reassign? | When to use |
| -------- | ------ | --------- | ----------- |
| Immutable binding | `$x = expr` | No | Default — most values should not change |
| Mutable binding | `$$y = expr` then `y = …` | Yes | Counters, accumulators, builder state |
| Typed binding | `$x: Int = 1` | No (Tiny: type names limited) | Document intent when inference is unclear |

## Immutable `$`

```jolt
$x = 10;
// x = 11;  // compile error in Tiny
```

**When to use:** Default choice. Prefer immutability unless you need to mutate.

## Mutable `$$`

```jolt
$$n = 0;
n = n + 1;
```

**When to use:** Loops, running totals, state machines.

## Anti-patterns

- Using `$$` everywhere "just in case" — makes Custodian tracking harder to reason about
- Reassigning a `$` binding — use `$$` from the start if mutation is required

## Runnable example

```jolt runnable
@main() None ->
    $x = 10;
    $$y = 10;
    y = 11;
    if y == 11 ->
        println("ok");
    ;;
;;
```

## See also

- [Tour §2](../tour/02-bindings.md)
- [bindings-mutable example](../examples/bindings-mutable.md)
- [Spec: bindings](../../spec/jolt-spec-v0.4.md)
