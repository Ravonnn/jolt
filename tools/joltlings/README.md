# Joltlings

Hands-on Jolt exercises (Rustlings-style). Uses the real `jolt` CLI.

## Setup

```bash
cargo build -p jolt-cli
cargo build -p joltlings
```

## Commands

```bash
./target/debug/joltlings list
./target/debug/joltlings run 1
./target/debug/joltlings watch intro_3
./target/debug/joltlings verify
./target/debug/joltlings init    # copy to ./joltlings/
```

Exercises live in `tools/joltlings/exercises/`. Modes: `run`, `check`, `test`.

[Learn hub](../../learn/hub.md)
