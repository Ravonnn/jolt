# Shared borrow <span class="edition-badge tiny">tiny</span>

```jolt runnable
@main() None ->
    $data = "hello";
    $v1 = borrow(data);
    $v2 = borrow(data);
    println(deref(v1));
    println(deref(v2));
;;
```

```bash
./target/debug/jolt check tests/tutorial/borrow_shared.jolt
```

[All ways to borrow](../guides/ways-to-borrow.md)
