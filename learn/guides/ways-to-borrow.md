# All ways to borrow and move

The Custodian enforces memory safety. These are the main access patterns in Tiny.

## Comparison

| Pattern | Syntax | Exclusive? | Runtime |
| ------- | ------ | ---------- | ------- |
| Move | `$b = a` (String) | Transfers ownership | N/A (compile-time) |
| Shared borrow | `borrow(x)` | No — many readers | `deref` to read |
| Exclusive claim | `claim(x)` | Yes — one writer | `deref` to read/write |
| Copy types | `Int`, `Bool` | Copies allowed | Use value directly |

## Shared borrow

Use when multiple readers need access and nobody needs to mutate:

{{#snippet tutorial/borrow_shared}}

## Move (String)

```bash
./target/debug/jolt check tests/custody/should_accept/string_move_ok.jolt
```

After move, the source name must not be used again.

## When to use which

- **Copy types (`Int`, `Bool`):** use directly — no `borrow`/`claim` needed
- **Read-only sharing:** `borrow`
- **Exclusive mutation:** `claim` (see spec for full rules)
- **Transfer ownership:** move by assignment

## Anti-patterns

- Holding a `claim` while also `borrow`ing — rejected by Custodian
- Using after move — compile error with hint

## See also

- [Tour §4](../tour/04-custodian.md)
- [move-string example](../examples/move-string.md)
