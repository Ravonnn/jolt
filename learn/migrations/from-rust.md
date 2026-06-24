# Coming from Rust

| Rust | Jolt (Tiny) |
| ---- | ----------- |
| `fn main()` | `@main() None -> … ;;` |
| `let x = 1` | `$x = 1` |
| `let mut y = 1` | `$$y = 1` |
| `&T` / `&mut T` | `borrow` / `claim` + `deref` |
| `println!` | `println` |
| `cargo test` | `jolt test` |
| `cargo run` | `jolt run --interpret` |

## Ownership

Rust's borrow checker maps to the **Custodian**. Shared reads use `borrow`; exclusive access uses
`claim`. Moves work similarly for `String`.

**Start:** [Custodian tour](../tour/04-custodian.md) · [ways to borrow](../guides/ways-to-borrow.md)

## Build tooling

```bash
cargo build -p jolt-cli          # like building rustc
./target/debug/jolt check file.jolt
```

[Learn hub](../hub.md)
