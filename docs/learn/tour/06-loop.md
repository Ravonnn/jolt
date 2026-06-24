# Tour §6b — Loops and break <span class="edition-badge tiny">tiny</span>

`loop` runs until `break;`. In Tiny, `break` requires a semicolon before the closing `;;`.

```jolt runnable
@main() None ->
    $$n = 0;
    loop ->
        n = n + 1;
        if n >= 10 ->
            break;
        ;;
    ;;
    if n == 10 ->
        println("10");
    ;;
;;
```

See: [All ways to iterate](../guides/ways-to-iterate.md)

**Next:** [For loops](06-for.md)
