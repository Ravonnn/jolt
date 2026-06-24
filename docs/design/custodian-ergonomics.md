# Custodian ergonomics report (Tiny, Phase 2c)

> Initial evaluation of the v0.4 Custodian model on the Tiny subset. Full gate sign-off still
> requires independent review per [`year-1.md`](year-1.md) Phase 2.

Last updated: **Phase 2c**.

---

## Programs evaluated

| Program | Source | Pattern |
| ------- | ------ | ------- |
| String handoff | `tests/custody/should_accept/string_move_ok.jolt` | Move `$a` → `$b`, use `$b` only |
| NLL re-borrow | `tests/custody/should_accept/borrow_nll.jolt` | `borrow` → `deref` → use owner again |
| Shared borrows | `tests/custody/should_accept/shared_borrows.jolt` | Multiple `borrow(data)` concurrently |
| Claim after release | `tests/custody/should_accept/claim_after_borrow_release.jolt` | Shared borrow ends, then `claim` |
| Tour-style read | `borrow` + `deref` + `println` (corpus) | Read through shared borrow |

---

## Friction vs Rust

**Easier or comparable**

- No lifetime annotations on Tiny programs; inference + last-use release is enough for the corpus.
- Vocabulary (`borrow` / `claim` / `deref`) reads clearly in error messages and tour docs.
- `$` / `$$` immutability vs mutability is visible at the binding site (Rust uses `let` / `let mut`).

**More friction (acceptable on Tiny)**

- `borrow`/`claim`/`deref` are call expressions today, not keywords — fine for Tiny, may want
  first-class syntax later.
- Only `String` is move-only in Tiny; real structs will stress the model more.
- Diagnostic hints are text-only; no `jolt fix` yet.

**Rust patterns that translate directly**

- Use-after-move on assign (`$b = a; use(a)`).
- Shared XOR mutable (`borrow` while `claim` live).
- Non-lexical release: `let v = borrow(x); use(v); use(x)` after `v` expires.

---

## What NLL solved

Without last-use release, `borrow_nll.jolt` and `claim_after_borrow_release.jolt` would force
artificial binding reordering or nested scopes. Statement-granular last-use tracking accepts the
natural tour-style sequencing:

```jolt
$v = borrow(data);
println(deref(v));
$w = borrow(data);   // OK: prior borrow handle expired
```

---

## Open pain points (no model change yet)

1. **Block-expression nesting** — deep `-> ... ;;` borrow scopes are easy to get wrong syntactically;
   prefer `if`/`for` bodies for now.
2. **Hint-only fixes** — suggestions are not machine-applicable until `jolt fix` exists.
3. **Corpus scale** — 10 accept / 8 reject cases prove the implementation, not full language safety.

---

## Conclusion

**The Custodian model is viable on Tiny.** Moves, shared borrows, exclusive claims, and NLL behave as
documented in `jolt-spec-v0.4.md` §9 for the programs above. No ADR or semantic redesign is required
before Phase 3 (interpreter). Scale-up risks (structs, fields, closures) remain explicitly deferred
to Core.

**Recommended next step:** independent custody review of `tests/custody`, then Phase 3a (MIR +
interpreter).
