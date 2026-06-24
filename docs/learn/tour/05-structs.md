# Tour §5 — Structs <span class="edition-badge tiny">tiny</span>

Structs group named fields. Declare a type, build values with struct literals, and read or update fields.

## Declare a struct

```jolt
struct Point ->
    $x: Int
    $y: Int
;;
```

Fields use `$` (immutable) or `$$` (mutable), like bindings.

## Struct literals

```jolt
$p = Point { x: 1, y: 2 };
```

Every field must be supplied (no defaults in this slice).

## Field access

```jolt
if p.x + p.y == 3 ->
    println("ok");
;;
```

## Mutable fields

When a field is `$$`, or the struct binding is `$$`, you can assign through the field:

```jolt
struct Counter ->
    $$n: Int
;;

@main() None ->
    $$c = Counter { n: 0 };
    c.n = 1;
    if c.n == 1 ->
        println("1");
    ;;
;;
```

```jolt runnable
struct Point ->
    $x: Int
    $y: Int
;;

@main() None ->
    $p = Point { x: 1, y: 2 };
    if p.x + p.y == 3 ->
        println("3");
    ;;
;;
```

See also: [Struct point example](../examples/struct-point.md)

**Next:** Control flow — [if / else](06-if-else.md)
