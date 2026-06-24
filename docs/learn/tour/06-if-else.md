# Tour §6 — If and else <span class="edition-badge tiny">tiny</span>

`if` / `else` are expressions. Each branch ends with `;;` before the outer closing `;;`.

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

**Next:** [Loops and break](06-loop.md)
