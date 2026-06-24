# Mutable claim <span class="edition-badge tiny">tiny</span>

Exclusive `claim` is used when you need mutable access through the Custodian (see spec for full
rules). Shared borrows are covered in [borrow-shared](borrow-shared.md).

```jolt runnable
@main() None ->
    $data = "hello";
    $v1 = borrow(data);
    $v2 = borrow(data);
    println(deref(v1));
    println(deref(v2));
;;
```
