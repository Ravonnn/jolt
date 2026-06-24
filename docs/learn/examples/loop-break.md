# Loop and break <span class="edition-badge tiny">tiny</span>

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

[All ways to iterate](../guides/ways-to-iterate.md)
