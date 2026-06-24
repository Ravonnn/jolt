# Tour §1 — Hello, Jolt <span class="edition-badge tiny">tiny</span>

Every Jolt program is a set of functions. Execution starts at `@main`.

The full language uses `using Std` and `!None` for the return type. In **Tiny** (what runs today),
we write:

{{#snippet tutorial/hello}}

Click **Run** above (requires `cargo xtask learn serve`) or:

```bash
./target/debug/jolt run --interpret tests/tutorial/hello.jolt
```

**Next:** [Values and bindings](02-bindings.md)
