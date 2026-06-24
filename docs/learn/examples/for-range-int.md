# For loop (Int upper bound) <span class="edition-badge tiny">tiny</span>

```jolt runnable
@sum(n: Int) Int ->
    $$total = 0;
    for x in n ->
        total = total + x;
    ;;
    total
;;

@main() None ->
    $s = sum(4);
    if s == 6 ->
        println("6");
    ;;
;;
```

Tiny: `for x in n` iterates `0 .. n-1`.
