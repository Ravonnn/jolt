# Struct point <span class="edition-badge tiny">tiny</span>

A minimal `Point` struct with field reads and a printed sum.

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

[Tour §5 — Structs](../tour/05-structs.md)
