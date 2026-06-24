# Arithmetic <span class="edition-badge tiny">tiny</span>

```jolt runnable
@main() None ->
    $x = 1 + 2;
    $y = x * 3;
    if y == 9 ->
        println("9");
    ;;
;;
```

Expected output: `9`
