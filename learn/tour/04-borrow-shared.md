# Tour §4b — Shared borrows <span class="edition-badge tiny">tiny</span>

Multiple shared `borrow` references can exist at once when nobody holds an exclusive `claim`.

{{#snippet tutorial/borrow_shared}}

**Custody check** (compile-only):

```bash
./target/debug/jolt check tests/tutorial/borrow_shared.jolt
```

See: [All ways to borrow](../guides/ways-to-borrow.md) · [Borrow shared example](../examples/borrow-shared.md)

**Next:** [If and else](06-if-else.md)
