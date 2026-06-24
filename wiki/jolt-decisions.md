# Jolt — Decisions Log

> Source of truth for locked design choices. References section numbers in `jolt-design-options.md`.
> Status tags: ✅ locked · ⏳ pending input · 🔓 deliberately open · ⬜ not yet decided.
>
> **All 28 core decisions are ✅ locked and all spec-level residuals are now resolved.** The only
> remaining work is compiler/toolchain *implementation* (notably the borrow checker) — no open
> language-design questions. Full consolidated spec: `jolt-spec-v0.3.md`; quick reference:
> `jolt-cheatsheet.md`.

| #  | Topic | Decision | Status |
| -- | ----- | -------- | ------ |
| 1  | Comments / macros / asm | `//` single-line, `///` multiline+doc. Macros use `#` prefix. `asm`/`c` are `asm -> … ;;` arrow blocks (drop `#asm`/`#c`). | ✅ |
| 2  | Block syntax | Arrow blocks `-> … ;;` **everywhere** (delete all `{ }` block examples). | ✅ |
| 3  | Type annotation | Colon: `$x: Int = 5`. | ✅ |
| 4  | Declaration vs assignment | `$x` = new **immutable** binding, `$$x` = new **mutable** binding, bare `x =` = reassign (mutable only), `?` = flex-typed (mutable only). Drop `:=`. | ✅ |
| 5  | Mutability | **Immutable by default.** `$x` = immutable, `$$x` = mutable. Compile-time constants use the **`[const]`** attribute on an immutable binding: `[const] $x = 5`. | ✅ |
| 6  | `Char` width | 32-bit Unicode scalar; `String` = UTF-8 bytes. | ✅ |
| 7  | `Complex` / `Rational` | Stdlib generics (`Complex<Float64>`, `Rational<Int64>`), not core primitives. | ✅ |
| 8  | Absence model | `T?` = `Option<T>` (`Some`/`None`), no raw null. `Empty` = empty *collection* only. `??` = coalesce. | ✅ |
| 9  | Power operator | `^` is power. No `**`. (XOR stays `%|`.) | ✅ |
| 10 | Bitwise notation | Symbols only (`&`, `|`, `%|`, `~&`, …). Drop word forms (`AND`/`XOR`/…). | ✅ |
| 11 | `//` / `**` | `//` = floor division. Drop `**`. | ✅ |
| 12 | Math symbols | `÷ √ ≠ ≈ ≡` are pure ASCII aliases (`÷`≡`/`, `√x`≡`sqrt(x)`, `≠`≡`!=`, etc.). | ✅ |
| 13 | Function arg guards | **Multiple-dispatch guards, evaluated at runtime.** Identical signatures = compile error. Two *different* guards both true for the same call = **runtime error** (ambiguous dispatch). | ✅ |
| 14 | Closures | `\|params\| Ret -> body ;;`. | ✅ |
| 15 | Multi-branch | Keep **both**: `switch`/`case` for value matching, `match` for pattern matching + destructuring. | ✅ |
| 16 | Loop forms | Two forms: `loop` (infinite) + `for` (where `for cond` doubles as `while`). `while` becomes sugar. | ✅ |
| 17 | struct vs class | Structs + **contracts** (`@@Name -> … ;;`). **Trait-style: no inheritance, no runtime polymorphism, no instance data** — only required methods + default methods. Usable as static generic bounds (monomorphized). Adoption: `Type::Contract -> … ;;`. | ✅ |
| 18 | Method site | Separate `Type::method() -> … ;;` blocks (not Rust `impl`). | ✅ |
| 19 | Memory model | **FULLY DECIDED.** Move + borrow-checked (Rust-style, Jolt-simplified), no implicit copy. `$b = a` **moves**; explicit `copy(a)` to duplicate. A type copies implicitly **iff it conforms to the `Copy` contract** (ints/floats/bools/etc.). `ref()` = borrow (shared XOR mutable, borrow-checked), `unref()` = **deref only** (not free). `ptr()`/`Pointer<T>` only in `[unsafe]`. **Lifetimes inferred**, with a rare-case annotation escape hatch. Shared ownership via a **`[shared]` attribute** (opt-in counted ownership). Allocators: implicit default + explicit, selected via an **allocator attribute**; containers remember their allocator. `defer`/`errdefer` cleanup. | ✅ |
| 20 | Traits/interfaces | **= the contract system (#17).** No separate trait construct; `@@`-contracts *are* Jolt's interfaces — static bounds, default methods, no data, no dynamic dispatch. | ✅ |
| 21 | Error handling | **FULLY DECIDED.** `!T` is sugar for `Result<T, E>` with `E` = inferred **open** error set (one underlying type, two spellings; `?` works on both). Explicit `Result<T, NamedEnum>` opts into a **closed** set that's exhaustively matchable. **`?`** = error propagation, **`??`** = Option unwrap/coalesce. `defer` (LIFO) + **`errdefer`** (error-path only). `error NotFound;` declares errors; `Error` = root contract all errors conform to. **No exceptions.** | ✅ |
| 22 | Generics syntax | `\|T\|` brackets, consistent with closures (§14). `@max\|T: Comparable\|(a: T, b: T) T -> …`. ("for now" — revisit if `\|\|` clashes become annoying.) | ✅ |
| 23 | Modules/imports | **Path-based, reads left→right** (Jolt syntax kept): `import Utils.Print;`, `from Utils import Print;`, `using Math;`, `export { A as a, B as b }`. Keep `package`/`library` only if needed for distribution; drop Carbon's api-file split (visibility via `[public]`, #24). | ✅ |
| 24 | Visibility | **Private by default; `[public]` attribute to expose.** `[public] @foo() -> … ;;`. Pairs naturally with #23's qualified cross-package access. | ✅ |
| 25 | Macros (power) | **Both** (C): hygienic declarative pattern macros for common cases + `comptime` procedural macros for advanced codegen. All `#`-prefixed per #1. | ✅ |
| 26 | `comptime` | **Zig-style** (A): `comptime` values/args evaluated at compile time; also backs the procedural macros in #25. Note: with a full contract/generics system (#17/#22), this overlaps Rust's `const fn` role — see note. | ✅ |
| 27 | Builtins | **Stdlib functions, not keywords** (A): `len`, `log`, `typeinfo`, `help`, `print`/`println` live in the prelude. Remove them from the reserved-word list; standardize on `print`/`println` (drop `printd`). | ✅ |
| 28 | Naming conventions | **Locked:** types/contracts = `PascalCase`; functions/methods/variables = `snake_case` (compiler-enforced); `[const] $` constants = `SCREAMING_SNAKE_CASE`; packages/libraries = `PascalCase`. Enforced by compiler + formatter. | ✅ |

## Notes / consequences to watch

- **#5 / #4 — two-level `$` / `$$`** ✅ resolved. `$x` immutable, `$$x` mutable; bare reassignment
  only on `$$`; flex-type → `$$x?`. Compile-time constants = `[const] $x = …` (the `[const]`
  attribute applies to immutable `$` bindings only; `[const] $$x` is contradictory → error).
- **#13 — runtime guards** ✅ resolved. Guards may read values; identical signatures = compile
  error; two different guards both true at runtime = runtime error. Trade-off to keep in mind: this
  is not zero-cost (a guard check per call, plus a possible runtime-error path), so design guards to
  be mutually exclusive. Consider letting the compiler *warn* when it can prove two guards overlap.
  - *Toolchain idea (not a decision):* since guards are runtime, an ambiguous call only fails when
    that specific input occurs. To catch it earlier, have the compiler emit a warning when it can
    statically prove two guards overlap.
- **#17 — trait-style contracts** ✅ resolved. No inheritance, no dynamic dispatch, no instance
  data; contracts declare required methods and may supply default bodies, used as static generic
  bounds (monomorphized). The diamond problem doesn't arise.
- **#15** keeps both `switch`/`case` and `match`. Document the boundary: `switch` = compare a value
  against constants; `match` = destructure/patterns with exhaustiveness.
- **#16**: spell out what happens to the orphaned `when` / `do` / `repeat` / `until` keywords
  (removed, or folded into `for`/`loop`).
- **#9 + #10:** `^` = power and word-form bitwise ops are gone, so XOR is *only* `%|`. Prominent note
  needed for C/Python migrants who expect `^` = XOR.
- **#19 — memory model (FULLY DECIDED).** Move-by-default, no implicit copy, compile-time borrow
  checking, Jolt-simplified. All previously-residual items are now **resolved**:
  - **Assignment:** `$b = a` moves; use-after-move = compile error; `copy(a)` to duplicate.
  - **Implicit copy:** a type is implicitly copied **iff it conforms to the `Copy` contract** (#17);
    primitives (ints/floats/bool/char) are `Copy`, everything else moves by default.
  - **Borrows:** `ref()` = shared borrow (`Ref<T>`); `ref_mut()` = mutable borrow (`RefMut<T>`,
    requires a `$$` source). Shared XOR exactly-one-mutable. `unref()` = **dereference only**, never
    frees. Borrows are **non-lexical** (end at last use).
  - **Lifetimes:** inferred, no surface syntax in the common case. Rare-case escape hatch: declare
    with `|life L|` in the generic bracket and attach as a second type parameter, `Ref<T, L>` — no
    new sigil.
  - **Shared ownership:** `[shared]` = non-atomic counted ownership (single-thread);
    `[shared, sync]` = atomic (cross-thread). Cycles are **not** auto-collected — break them with a
    `[weak]` reference (does not keep its target alive).
  - **Allocators:** implicit default allocator + explicit allocators via the **`[alloc: name]`**
    attribute (attaches to functions/blocks/types). Containers **remember** their allocator. The
    process default is swapped with `[alloc: …]` on `@main`.
  - **`[unsafe]` powers (exactly five):** (1) create/deref raw `Pointer<T>` via `ptr()`,
    (2) pointer arithmetic, (3) call other `[unsafe]` fns, (4) read/write `union` fields, (5) inline
    `asm`. `[unsafe]` does **not** disable the borrow checker on safe references.

  The borrow checker implementation ("easier than Rust") remains the main engineering challenge, but
  all language-surface decisions are now settled.

### Error handling (#21) — DECIDED, with details to spec out

Model: **both `Result<T,E>` and `!T` coexist**, with **Zig-style open error sets**, **distinct
operators** (`?` = error propagation, `??` = Option unwrap/coalesce), `defer` runs on error paths,
**no exceptions**. Worked sketch:

```jolt
// !T : error union with an inferred, open error set
@read_config() !Config ->
    $f = open("cfg")?;        // ? propagates the error, auto-unions into this fn's error set
    return parse(f);
;;

// Result<T,E> : explicit when you want to name/inspect the error type
@parse(s: String) Result<Config, ParseError> -> ... ;;

// Option uses ?? (distinct from error ?)
$$name = lookup(id) ?? "anonymous";   // coalesce on None
```

Resolved details:
  - **`!T` ⇄ `Result`** ✅ `!T` is sugar for `Result<T, E>` where `E` is the function's inferred
    **open** error set. Single underlying type → `?` works on `!T` and `Result` alike.
  - **Open by default, closed on demand** ✅ `!T` → open set (auto-unions across `?`; `match` on it
    requires an `else`/`default` arm, no exhaustiveness). Explicit `Result<T, NamedEnum>` → closed
    set, exhaustively matchable. Authors pick per function.
  - **Operators** ✅ `?` = error-only propagation; `??` = Option-only unwrap/coalesce. Using `?` on
    an `Option` (or `??` on a `Result`) is a **compile error** with a hint pointing to the other
    operator.
  - **`error` / `Error`** ✅ `error NotFound;` declares an error value/member; `Error` is the root
    contract every error conforms to, and an "open error set" is conceptually a set of `Error`s.
  - **`defer` / `errdefer`** ✅ both exist; LIFO ordering. `defer` always runs on scope exit;
    `errdefer` runs only when the scope exits via an error return. Both run on error paths.

### Notes on #20–#28

- **#20 = #17.** Contracts serve as both the trait system and the interface/bound mechanism; there
  is no second construct. Generics (#22) get their bounds from contracts: `|T: Comparable|`.
- **#23 (modules) — path-based, Jolt syntax.** Forms: `import Utils.Print;` (qualified path),
  `from Utils import Print;` (selective), `using Math;` (whole package), `export { A as a, B as b }`.
  All read left→right. Three tiers are surfaced: **`library`** (reuse/import unit), **`package`**
  (distribution unit, contains libraries), **`program`** (compilation unit with `@main` → an
  executable). Visibility is `[public]`-driven (#24); no api-file split. The Elm-style
  `module … exposing (...)` form is dropped in favor of `export` + `[public]`.
- **#24 (visibility):** `[public]` is private's opposite; consider whether you also need a
  library-private-but-not-file-private middle tier (Carbon has api-private vs impl-private). Probably
  not needed initially.
- **#25 + #26 overlap:** procedural macros (#25) and `comptime` (#26) are the same machinery —
  proc-macros are just `comptime` functions that emit code. Keep them unified to avoid two
  metaprogramming systems. Declarative pattern macros remain a separate, simpler surface.
- **#26 vs generics:** with contracts + `|T|` generics already providing static polymorphism,
  `comptime` is for value-level compile-time eval and codegen, *not* the primary generics mechanism
  (unlike Zig, where comptime *is* generics). Worth stating explicitly so the two don't compete.
- **#27:** moving `len`/`log`/`typeinfo`/`help` out of keywords means they can be shadowed and must
  be resolved via the prelude/import system (#23). Make sure the prelude is auto-imported.
- **#28:** `[const]` constants as `SCREAMING_SNAKE_CASE` is a convention, not enforced like
  snake_case variables — confirm whether the compiler *enforces* or merely *lints* constant casing.
