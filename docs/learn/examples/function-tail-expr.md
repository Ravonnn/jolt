# Function tail expression <span class="edition-badge tiny">tiny</span>

```jolt runnable
@double(x: Int) Int -> x * 2 ;;

@main() None ->
    $n = double(21);
    if n == 42 ->
        println("42");
    ;;
;;
```

[All ways to return](../guides/ways-to-return.md)
