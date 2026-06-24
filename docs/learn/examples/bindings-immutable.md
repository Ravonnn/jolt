# Immutable bindings <span class="edition-badge tiny">tiny</span>

Use `$name` for bindings that are set once.

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

[All ways to bind](../guides/ways-to-bind.md)
