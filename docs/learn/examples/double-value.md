# Double a value <span class="edition-badge tiny">tiny</span>

```jolt runnable
@double(x: Int) Int -> x * 2 ;;

@main() None ->
    $n = double(21);
    if n == 42 ->
        println("42");
    ;;
;;
```

[Tour §3](../tour/03-functions.md)
