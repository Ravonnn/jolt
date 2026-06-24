# If / else <span class="edition-badge tiny">tiny</span>

```jolt runnable
@max(a: Int, b: Int) Int ->
    if a > b ->
        a
    ;; else ->
        b
    ;;
;;

@main() None ->
    $m = max(3, 7);
    if m == 7 ->
        println("7");
    ;;
;;
```

[Tour §6](../tour/06-if-else.md)
