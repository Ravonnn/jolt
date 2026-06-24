# Tour §4 — The Custodian <span class="edition-badge tiny">tiny</span>

Jolt tracks **who owns each value** at compile time via the Custodian. For `String`, ownership
transfers on move; `Int` and `Bool` are copyable.

This lesson uses `borrow` for shared read access and `claim` for exclusive access (see the
[spec](../../spec/jolt-spec-v0.4.md) for the full model).

```jolt runnable
@main() None ->
    $data = "hello";
    $v1 = borrow(data);
    $v2 = borrow(data);
    println(deref(v1));
    println(deref(v2));
;;
```

Use **Check** (not Run) for programs that only define helper functions without `@main`, or use the
runnable example above.

**Quiz:** [Custodian basics](../quizzes/custodian-01.toml) (concept check)

**Next:** [Shared borrows in practice](04-borrow-shared.md)
