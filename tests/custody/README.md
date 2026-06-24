# Custodian conformance tests

Ownership and borrow checking: programs that must be accepted or rejected by the Custodian.

## Layout

```
tests/custody/
  should_accept/
    case.jolt
  should_reject/
    case.jolt
    case.stderr    # expected custody diagnostic lines (with optional ; hint: …)
```

Sample cases under `tests/custody/sample/` (Phase 0 smoke).

## Phase 2a–2b

Move analysis and `borrow`/`claim`/`deref` with shared-XOR-mutable rules and non-lexical borrow
release.

## Phase 2c (gate scale)

| Code | Meaning |
| ---- | ------- |
| E0401 | use of moved value |
| E0402 | cannot use while borrowed |
| E0403 | cannot claim while borrowed |
| E0404 | cannot borrow while claimed |

Corpus: **10** accept, **8** reject (with `.stderr` snapshots). Diagnostics include suggested-fix
hints after the error code.

```bash
cargo test -p jolt-custody
cargo test -p jolt-harness custody_corpus_snapshots
cargo build -p jolt-cli
./target/debug/jolt check tests/custody/should_accept/
./target/debug/jolt check tests/custody/should_reject/double_claim.jolt  # exit 1
```

Refresh reject snapshots:

```bash
UPDATE_UI=1 cargo test -p jolt-harness custody_corpus_snapshots
```

See [`docs/design/custodian-ergonomics.md`](../docs/design/custodian-ergonomics.md) for the initial
ergonomics evaluation.
