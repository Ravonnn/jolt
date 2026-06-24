# Jolt v0.4 — Changes from v0.3 & New Open Questions

## What changed (by review item)

| Item | Decision applied in v0.4 |
| ---- | ------------------------ |
| A1 | `-> … ;;` bodies are **statement lists**; a final expression without `;` is the block's value. |
| A2 | `?` flex-type stays mutable-only. |
| A3 | Field mutability is per-field (`$`/`$$`); a mutable field is writable **only** through a mutable binding; an immutable binding freezes everything. |
| A4 | Guards now reference real declared parameters (`unit: String` added). |
| A5 | `match` is the pattern-matching construct (expanded); `switch` kept only for constant comparison. |
| A6 | `None` = void type. `Option` cases are now **`Some` / `Nothing`** (not `None`). |
| A7 | Unicode math aliases (`÷ √ ≠ ≈ ≡`) **removed**. |
| A8 | Union fields take `$`/`$$` sigils like struct fields. |
| B1 | **Concurrency**: models A (structured `scope`/`spawn`) + B (raw `Thread`); **I/O**: Tier 1 completion-based (`io`) + Tier 2 fibers (`fiber`), no async/await; `Channel`/`Mutex`/`Atomic`/`RwLock`/`Once`/`Barrier`; `Sendable`/`Shareable` contracts (Send/Sync analogues). |
| B2 | **`dyn Contract`** for dynamic dispatch + heterogeneous collections; `as` downcast → `Option`. |
| B3 | **`Dispose`** contract (`@dispose($$self)`) = RAII destructors. |
| B4 | Full pattern grammar: literals, enums, tuples/structs, `_`, `..` rest, or-patterns `|`, arm guards `if`, range patterns, whole-value capture `:=`. |
| B5 | **`Iterator`/`Iterable`** contracts; `for` desugars to them; lazy adapters (`map`/`filter`/`take`). |
| B6 | Strings: `+` concat, `{}` interpolation, `Display` contract, `.chars()`/`.bytes()`, slicing. |
| B7 | Visibility granularity: `[public]` and `[public: package]`. |
| B8 | Value generics: `|comptime N: Uint|`, fixed-size `Array<T, N>`. |
| B9 | Overflow policy (trap debug / wrap release + `wrapping_*`/`checked_*`/`saturating_*`); lossy conversions require `as`; `as` now defined. |
| B10 | FFI: `extern` declarations + `[extern: "C", link: …]`. |
| C1 | Named & default arguments. |
| C2 | Labeled loops via `[label: name]` + `break/next label`. |
| C3 | `match` is an expression. |
| C4 | Blocks are expressions (the §0 rule). |
| C5 | Operator overloading via contracts (`Plus`/`Minus`/`Times`/`Over`/`Equals`), method **is** the operator symbol (`@+`). Non-Rust naming. |
| C6 | `Slice<T>` views. |
| C7 | Comptime reflection API on top of `typeinfo`. |
| C8 | `[test]` + `assert`. |
| C9 | Doctests in `///` comments. |
| C10 | Full safety/capability attribute catalog (§21): resource (`noalloc`/`noio`/`nopanic`/`noblock`/`nostd`/`norecurse`/`bounded_stack`), purity (`pure`/`const`/`idempotent`/`total`), concurrency (`threadsafe`/`atomic`/`main_thread`/`no_capture`), security (`constanttime`/`zeroize`/`tainted`/`secret`), lint (`must_use`/`experimental`/`stable`). Transitively enforced. |

## Borrow checker renamed → the **Custodian**
- Borrow checker → **the Custodian**; a violation is a **custody violation**.
- `ref`/`ref_mut`/`unref` → **`borrow`/`claim`/`deref`**; types `Borrow<T>`/`Claim<T>`.
- Semantics unchanged (still Rust-style shared-XOR-mutable, move, non-lexical).

## New open questions this round surfaced

These are *new* forks created by the v0.4 features; none block the spec but each wants a call:

1. **`Some`/`Nothing` naming** — ✅ confirmed.
2. **Naming** — checker = **Custodian**, borrows = **`borrow`/`claim`/`deref`**, violation =
   **custody violation** (settled).
3. **`dyn` boxing & allocation** — ✅ resolved: a `dyn` object is **always heap-boxed**, owned by
   the allocator in effect at its creation site (default allocator, or the `[alloc: …]` in scope).
   `Array<dyn T>` stores boxes allocated by the array's allocator. No stack-boxing variant.
4. **Concurrency & I/O** — ✅ resolved. **Concurrency:** two models — A structured `scope`/`spawn`,
   B raw `Thread::spawn`. **I/O:** two tiers — Tier 1 completion-based (`io`, zero runtime), Tier 2
   green threads/fibers (`fiber`, ergonomic, sits on Tier 1). **No `async`/`await`** (green threads
   avoid function coloring). All enforced safe by the Custodian + `Sendable`/`Shareable`; non-atomic
   `[shared]` across threads is **rejected** (no silent upgrade).
5. **`Dispose` + move interaction** — ✅ resolved: when a value is moved, its `@dispose` runs at the
   **new owner's** scope end (move transfers the destruction obligation; the old binding is dead and
   runs nothing). A **partial move** out of a struct that conforms to `Dispose` is a **compile error**
   — a `Dispose` type must be disposed whole, so you cannot move a field out and leave a partially-
   live object. (Extract via a method that consumes `$$self` and returns the pieces instead.)
6. **Operator-as-method** — ✅ resolved: spelled **`@(+)`** (operator in parens after `@`).
7. **`switch`** — ✅ resolved: **removed**; `match` subsumes it.
8. **Value-generics vs guards** — ✅ resolved (Option B). The compiler classifies each `{guard}`:
   a guard mentioning **only comptime parameters** is evaluated at monomorphization — a failing
   instantiation is a **compile error**, a passing one emits **no runtime check** (so `{N > 0}`
   becomes a real compile-time constraint, like a `where` bound). Guards touching any **runtime
   value** stay runtime-checked exactly as before. Mixed guards are split: the comptime part gates
   instantiation, the runtime part is checked at the call.
9. **Capability attributes** — ✅ full catalog folded into §21 (resource/purity/concurrency/security/
   lint families) with transitive-enforcement rules. Remaining impl work: the transitive checker
   itself + annotating the whole stdlib (an implementation sub-project, not a design question).
10. **Interpolation grammar** — ✅ resolved in the EBNF (`jolt-grammar.md` §1/§8.3):
    `"{ expr [: format_spec] }"`, with `{{`/`}}` as literal-brace escapes and `\u{…}` for unicode.
    `format_spec` (e.g. `hex`, `.2`, `08`) is an opaque token handed to the `Display`/format machinery.
11. **`[const]` overload** — ✅ resolved: **split**. `[const]` stays the *binding* attribute (§4,
    compile-time constant value); the *function* "evaluable at compile time" attribute is renamed
    **`[constfn]`** (§21.2).

## Suggested next step
Resolved this round: naming (#1, #2, #6, #7), concurrency model (#4), capability attrs kept (#9).
Still open: `dyn` allocation (#3), `Dispose`+move (#5), value-generics×guards (#8), interpolation
grammar (#10), and the capability *propagation* design. A **grammar (EBNF)** pass would naturally
force #8 and #10 into final form and lock the `@(+)` and pattern syntax.
