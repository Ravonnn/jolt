# String move (compile check) <span class="edition-badge tiny">tiny</span>

Moving a `String` transfers ownership. This corpus example type-checks and passes Custodian:

```jolt
@move_demo() None ->
    $s = "hello";
    $t = s;
    println(t);
;;
```

Run **Check** on `tests/custody/should_accept/string_move_ok.jolt`:

```bash
./target/debug/jolt check tests/custody/should_accept/string_move_ok.jolt
```

[All ways to borrow](../guides/ways-to-borrow.md)
