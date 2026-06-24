# Property tests

Property-based and corpus-wide checks for the compiler front end.

## Phase 1f

| Test | Command |
| ---- | ------- |
| `fmt(fmt(x)) == fmt(x)` on `tests/ui/tiny/` | `cargo test -p jolt-harness fmt_idempotent_on_tiny_corpus` |
| Full pipeline accept on `tests/ui/tiny/` | `cargo test -p jolt-harness ui_tiny_corpus_check_clean` |
| Type-reject `.stderr` snapshots | `cargo test -p jolt-harness ui_type_reject_corpus_snapshots` |

Or run all harness property tests:

```bash
cargo test -p jolt-harness property_tests
```

Future: dedicated `tests/property/` Rust crate with proptest generators (Phase 11+).
