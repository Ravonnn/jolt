# Tour §2 — Values and bindings <span class="edition-badge tiny">tiny</span>

Jolt has two binding kinds:

| Syntax | Meaning |
| ------ | ------- |
| `$name` | Immutable binding — assign once |
| `$$name` | Mutable binding — reassign with `name = …` |

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

See also: [All ways to bind values](../guides/ways-to-bind.md)

**Next:** [Functions](03-functions.md)
