# Tour §3 — Functions <span class="edition-badge tiny">tiny</span>

Functions use `@name(params) ReturnType -> body ;;`. The body is an expression (often a tail call).

```jolt runnable
@double(x: Int) Int -> x * 2 ;;

@main() None ->
    $n = double(21);
    if n == 42 ->
        println("42");
    ;;
;;
```

The full tour shows string `{interpolation}` and default arguments — not in Tiny yet.

See also: [All ways to return values](../guides/ways-to-return.md)

**Next:** [The Custodian](04-custodian.md)
