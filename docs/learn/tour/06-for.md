# Tour §6c — For loops (Tiny range) <span class="edition-badge tiny">tiny</span>

In Tiny, `for x in n` where `n: Int` iterates `x = 0 .. n-1` (exclusive upper bound). Full Jolt
will use `0..n` range syntax.

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

**Next:** [Testing](11-testing.md)
