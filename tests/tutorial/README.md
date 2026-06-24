# Tutorial corpus (Tiny adaptations)

Runnable Tiny programs aligned with [`docs/tutorial/jolt-tour.md`](../../docs/tutorial/jolt-tour.md) and
[`learn/tour/`](../../learn/tour/).

| File | Tour section | Notes |
| ---- | -------------- | ----- |
| `hello.jolt` | §1 Hello | No `using Std`, `[public]`, or `!None` |
| `bindings.jolt` | §2 Bindings | `$` / `$$` / reassignment |
| `double.jolt` | §3 Functions | No interpolation or default args |
| `borrow_shared.jolt` | §4 Custodian | `borrow` / `deref` |
| `if_else.jolt` | §6 Control flow | `if` / user fn |
| `loop_break.jolt` | §6 | `loop` / `break` |
| `for_sum.jolt` | §6 | `for x in n` |
| `testing.jolt` | §11 Testing | `assert_eq` in `@main` |

```bash
cargo build -p jolt-cli
./target/debug/jolt run --interpret tests/tutorial/hello.jolt
cargo test -p jolt-harness tutorial_corpus
cargo xtask learn serve --open
UPDATE_UI=1 cargo test -p jolt-harness tutorial_corpus
```

Manifest: [`learn/curriculum.yaml`](../../learn/curriculum.yaml)
