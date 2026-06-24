# Jolt — Design Options Menu (v0.2)

> For each open decision this lists **concrete alternatives** with tradeoffs and a **★ recommended** pick.
> Mix and match — they're mostly independent. Reference the section numbers from `jolt-spec-v0.2.md`.

---

## 1. The `#` collision (comments vs macros vs asm/c)  — DECIDED

The collision is resolved by **moving comments off `#`**, which frees `#` for macros:

- **Comments → `//`** (C/Rust-style). `#` is no longer a comment char.
  ```jolt
  // single-line comment
  /// multiline / doc-comment style — define the doc variant in §1a
  ```
- **Macros → `#`** prefix. No clash now that comments use `//`.
  ```jolt
  #ref(person)
  #(ref, person)
  ```
- **`asm` / `c` → arrow blocks** like every other construct (§2):
  ```jolt
  asm ->
      movq $60, %rax   // exit syscall (Linux)
      movq $2,  %rdi
      syscall
  ;;

  c ->
      /* C source */
  ;;
  ```

`[OPEN]` Decide the **doc-comment** form now that the base is `//`. Suggested set:
`//` line, `/// …` line doc, `/* … */` block, `/** … */` block doc — or keep your `?`-style
markers adapted to slashes (`//? … ?//`). Pick one and I'll lock it into the spec.

`[NOTE]` Knock-on effects to apply across the spec: drop `#asm`/`#c` (now `asm`/`c` blocks),
and re-spell the macro section (§13/§25) around the `#` prefix instead of `!`.

---

## 2. Block syntax: `-> … ;;` vs `{ }`

- **Option A — arrow blocks everywhere** `-> … ;;` ★ (for distinctiveness)
  Consistent with your function/if/match/class examples. Unique identity. Downside: `;;` is unusual, and nesting many `->`/`;;` can get visually busy.
- **Option B — braces everywhere** `{ }`
  Maximally familiar (C/Rust/Go). Plays well with editors/formatters out of the box. Loses the language's distinct flavor.
- **Option C — hybrid: `->` for single-expression bodies, `{ }` for multi-statement**
  `@double(x: Int) Int -> x * 2;` vs `@f() { … }`. Ergonomic (like Rust closures / Scala), but two forms to learn and the "when do I switch" rule must be crisp.

**★ Pick A or B and commit globally.** If you love `->`, go A and *delete* the brace examples. If you want fast tooling and onboarding, B. Avoid the current accidental mix.

---

## 3. Type annotation: colon vs space

- **Option A — colon** `$x: Int = 5` ★
  Unambiguous next to expressions; matches drafts 2/3, Rust, TS, Swift, Python.
- **Option B — space** `$x Int = 5`
  Go-like, less punctuation. But ambiguous in some positions (e.g. multi-name destructuring) and clashes with your `name type` function params.

**★ A.**

---

## 4. Declaration vs assignment model

- **Option A — sigil = new binding, bare = reassign** ★
  `$x = 1` (new), `x = 2` (reassign), `$$x` const, `$x?` flex-typed. Clean; drop `:=` entirely.
- **Option B — keep `:=` for declaration, `=` for assignment, no sigil**
  Go/Pascal feel. But then `$` is redundant — you'd drop the sigil too, losing a recognizable Jolt trait.
- **Option C — explicit keywords** `let` / `const` / `var`
  Most readable for newcomers, most verbose, and abandons the `$`/`$$` identity you've established.

**★ A.** It's the only model where `$`/`$$`/`?` all pull their weight.

---

## 5. Mutability default

You have `$` (mutable) and `$$` (const). For a *safety* language, consider the default:

- **Option A — `$` mutable, `$$` immutable** (current)
  Easy, but mutable-by-default is the opposite of Rust's safety stance.
- **Option B — `$` immutable, `$mut`/`$$` for mutable** ★ for the safety goal
  Immutable-by-default nudges safer code; mutation is opt-in and visible.
- **Option C — three levels**: `$` (immutable), `$mut` (mutable), `$$` (compile-time const)
  Most precise; distinguishes runtime-immutable from comptime-constant.

**★ C** if you're serious about "Rust-like safety"; **A** if you prioritize approachability.

---

## 6. `Char` width

- **Option A — 32-bit Unicode scalar** ★
  Holds any code point (emoji included). `String` = UTF-8 bytes; iterate as scalars or graphemes.
- **Option B — keep 16-bit**
  Compact, but can't represent code points above U+FFFF without surrogate hacks. Bad fit for a modern language.
- **Option C — `Char` = single UTF-8 *byte*, `Rune` = scalar**
  Go-like split. Two types, but honest about bytes vs characters for a low-level language.

**★ A** (or C if you want byte-level honesty).

---

## 7. `Complex` / `Rational`

- **Option A — stdlib generic types** ★
  `Complex<Float64>`, `Rational<Int64>`. Keeps the core small; most systems languages don't bake these in.
- **Option B — built-in primitives with fixed layout**
  `Complex` = two Float64s, `Rational` = two Int64s. Convenient for numeric/scientific users, but bloats the core type system.

**★ A.** Drop them from the primitive table; provide a `Math`/`Numeric` package.

---

## 8. Absence / null model (`None`, `Empty`, `?`, `??`)

- **Option A — one optional type, no null** ★
  `T?` is sugar for `Option<T>` (`Some(v)` / `None`). `Empty` = empty *collection* only (unrelated). `??` = null-coalescing on optionals. No raw null anywhere safe.
- **Option B — nullable references**
  `Ref<T>?` can be null; `None` is the null literal. Simpler to implement, reintroduces null-deref risk (against the safety goal).
- **Option C — distinct `None` (absence) and `Empty` (zero-value) both first-class**
  Expressive but confusing; users won't know which to use.

**★ A.** Collapse to `Option`/`T?`; keep `Empty` strictly for "empty container."

---

## 9. Power operator

- **Option A — `^` is power, no `**`** ★
  One operator. But `^` = power means XOR must be the symbolic `%|` (already your choice) — document loudly since C users expect `^`=XOR.
- **Option B — `**` is power, `^` is XOR (C/Python-compatible)**
  Least surprising to outsiders; frees `%|`. Changes your current operator table.
- **Option C — no power operator; use `pow(x, y)`**
  Cleanest precedence story (power's right-associativity is a common bug source). Slightly verbose.

**★ B** if interop-friendliness matters; **A** if you want Jolt's own identity. Either way, kill the duplicate.

---

## 10. Bitwise: words vs symbols

You have both `AND/OR/XOR/NAND/NOR/XNOR/NOT` and `&/|/%|/~&/~|/~%|/~`.

- **Option A — symbols only** ★
  Compact, conventional. Drop the word forms (they collide visually with logical `and`/`or`).
- **Option B — words only**
  Very readable for HDL-style bit work, verbose for everyday masking.
- **Option C — keep both as aliases**
  Flexible but doubles the surface area and invites inconsistent codebases.

**★ A.** Keep symbols; reserve words only if you target hardware/HDL users specifically.

---

## 11. `//` and `**` "to be implemented"

- **Option A — `//` = floor division, drop `**`** ★ (since `^`/`**` already cover power)
- **Option B — drop both**; provide `floor_div(x,y)` / `pow(x,y)`.
- **Option C — keep `//` floor-div and `**` power, drop `^`** — pick *one* power op (see §9).

**★ A.**

---

## 12. Math symbols (`÷ √ ≠ ≈ ≡`)

- **Option A — pure aliases** ★: `÷`≡`/`, `√x`≡`sqrt(x)`, `≠`≡`!=`. Nice-to-type, zero new semantics.
- **Option B — distinct operators** (e.g. `≈` = approx-equal with tolerance, `≡` = identity/same-object). Adds real meaning but real complexity.
- **Option C — drop them** from core; let editors/input-methods expand to ASCII.

**★ A** for `÷`/`√`/`≠`; consider **B** only for `≈` (float tolerance) and `≡` (identity) if you have a use.

---

## 13. Function argument guards (`{type != "inch"}`)

- **Option A — multiple dispatch / overload guards** ★
  Several `@convert` definitions; compiler picks by a boolean guard. Powerful, Julia-like. Needs clear resolution rules.
- **Option B — `where` clause** (type constraints only, not value predicates)
  `@max|T|(a:T,b:T) T where T: Ord -> …`. Standard generics path; simpler than value-dispatch.
- **Option C — default arguments + body branching** (no guards at all)
  Simplest; the `if type==...` lives in one function body.

**★ Split them:** use **B** (`where`) for *type* constraints and **C** for *value* behavior. Reserve **A** only if you truly want multiple dispatch as a headline feature.

---

## 14. Closures / function values  (currently missing)

- **Option A — `|params| Ret -> body ;;`** ★ (matches your generic `|…|` bracket)
  `$f = |x: Int| Int -> x*2 ;;`. Consistent with the rest of the syntax.
- **Option B — Rust-style `|x| x*2`** with inferred body
  Terse; needs inference rules for the return.
- **Option C — anonymous `@(x: Int) Int -> … ;;`**
  Ties to the `@` function family; reads as "a function literal."

**★ A.**

---

## 15. `match` vs `switch`/`case` vs `if ->` cond-list

- **Option A — keep `match` + `if/else` only** ★
  `match` = value/pattern dispatch (with destructuring + exhaustiveness); `if/else` = boolean. Drop `switch`/`case` and the `if ->` cond-list.
- **Option B — keep `switch`/`case` for values, `match` for patterns**
  Two tools; users must learn the boundary.
- **Option C — one mega-construct** `match` that also handles boolean guard arms
  `match -> cond1: …; cond2: …;` doubling as the cond-list. Fewest keywords, slightly magic.

**★ A.**

---

## 16. Loop forms (currently six)

- **Option A — three forms** ★: `loop` (infinite), `while cond`, `for x in iter`. Fold post-condition into `do … while`.
- **Option B — two forms**: `loop` + `for` (where `for cond` doubles as `while`). Minimal; `while` becomes sugar.
- **Option C — keep all, define each precisely** (`when` = one-shot conditional? `do` = do-while? `repeat/until` = post-condition?). Maximum expressiveness, heavy keyword budget.

**★ A.** Decide explicitly what `when`/`do`/`repeat`/`until` were for; most fold into the three.

---

## 17. `struct` vs `@@class`

- **Option A — structs + traits, drop classes** ★ (most Rust-like)
  Data in `struct`/`enum`; behavior in trait impls. One mental model; no inheritance footguns.
- **Option B — keep both, sharply differentiated**
  `struct` = plain value type (stack, no inheritance); `@@class` = reference type (heap, single inheritance, vtable). Familiar to OOP users; adds runtime cost + complexity.
- **Option C — structs only, with optional methods, no traits**
  Simplest; but you lose generic bounds and polymorphism (bad for the systems-generics goal).

**★ A.** It aligns with the safety pitch and unlocks generics cleanly.

---

## 18. Method definition site

- **Option A — separate `Type::method()` blocks** (draft 1) ★ for flexibility
  Lets you add methods across files; pairs naturally with trait impls.
- **Option B — methods inside the type body**
  Everything in one place; less flexible for extension.
- **Option C — `impl` blocks** `impl Person -> … ;;` and `impl Trait for Person -> … ;;`
  Rust-style; clean separation of inherent vs trait methods.

**★ C** if you adopt traits (§20); otherwise **A**.

---

## 19. Memory & ownership model  *(the big one)*

- **Option A — full ownership + borrow checker** (Rust-like) ★ for the stated goal
  Move-by-default, `Ref<T>` = borrow (shared XOR mutable), lifetimes inferred where possible. `Pointer<T>` only in `[unsafe]`. Strongest safety; hardest to implement; steepest learning curve.
- **Option B — ARC / reference counting**
  `Ref<T>` = counted reference, auto-freed; `Pointer<T>` raw/unsafe. Easier to learn (Swift-like), some runtime cost + cycle leaks.
- **Option C — manual with safety rails**
  Explicit alloc/free, compiler warns on obvious misuse, `[unsafe]` for raw work. Closest to C; least "safe."
- **Option D — regions / arenas + `[heap]` opt-in**
  Allocations tied to scopes/arenas; `[heap]` escapes to manual. Great for systems perf, niche learning curve.

For each, define: move vs copy on `$a = b`, what `Ref<T>` vs `Pointer<T>` mean, and the role of `:&`/`=&`.

**★ A** if "Rust-like safety" is the headline; **B** if you want safety with a gentler curve. Pick before anything else — it shapes generics, errors, and the stdlib.

Sub-decision — `ref`/`unref` spelling:
- **A1** lowercase functions `ref()`/`unref()`/`ptr()` ★
- **A2** type-constructor style `Ref<T>(x)` / `Unref(x)`
- **A3** operators `:&` (bind-ref) / `=&` (assign-ref), no functions
→ **★ A1**, drop A2/A3 to avoid three spellings.

---

## 20. Traits / interfaces  *(currently missing, needed for generics)*

- **Option A — Rust-style traits** ★
  `trait Ord -> @cmp(self, other: Self) Int; ;;` + `impl Ord for Int -> … ;;`. Static dispatch by default, dynamic via `dyn`. Best fit with Option 17-A/19-A.
- **Option B — Go-style structural interfaces**
  A type satisfies an interface implicitly if it has the methods. Less boilerplate, weaker intent signaling, harder error messages.
- **Option C — Swift-style protocols with associated types**
  Very expressive; complex to implement.
- **Option D — no traits, just duck-typed generics (`comptime`)**
  Zig-like: generic over any type, checked at instantiation. Simple compiler, worse error messages, no upfront bounds.

**★ A** for a safety-first static language; **D** if you'd rather lean on `comptime` and keep the type system small.

---

## 21. Error handling

- **Option A — `Result<T,E>` + `?` propagation** ★
  `@f() Result<T,Error> -> $x = g()?; …`. Errors are values; pairs with `Option`. Matches the safety theme.
- **Option B — exceptions** (`throw`/`try`/`catch`)
  Familiar, ergonomic, but hidden control flow — unusual for a low-level language.
- **Option C — error unions (Zig-style)** `!T`
  `@f() !T -> …; try g();`. Lightweight, integrates with `comptime`. Good systems fit.
- **Option D — multi-return `(value, err)`** (Go-style)
  Explicit, simple, verbose and easy to ignore the error.

**★ A or C.** Then unify the zoo: `?`=optional, `??`=coalesce, `Result`/`!T`=fallible, `error`/`Error`=the error type/keyword. Avoid having all of exceptions *and* Results.

---

## 22. Generics syntax

- **Option A — `|T|` brackets** (your current) ★
  `@max|T: Ord|(a:T,b:T) T -> …`. Distinct; consistent with closures (§14).
- **Option B — angle brackets `<T>`**
  Universally recognized; parser ambiguity with `<`/`>` comparisons (the classic C++ headache).
- **Option C — square brackets `[T]`** (Scala/Nim)
  No comparison ambiguity; mild clash with array/index syntax.

**★ A.** It avoids the `<>` ambiguity and matches your bracket-for-type-params instinct.

---

## 23. Module / import grammar

- **Option A — path-based, Rust/Python-like** ★
  `import Utils.Print;` / `from Utils import Print;` / `using Math;`. `export { A as a, B as b }`. Reads left→right.
- **Option B — keep `import X module Y` / `package` keywords**
  Distinct, but reads backwards and the module-vs-package distinction needs strict rules.
- **Option C — Elm-style `module Main exposing (…)`**
  Clean public-API declaration at the top of each file; commit fully (don't mix with `export`).

**★ A** for imports + **C's `exposing`** idea folded in as the file's export list, if you like declaring the public surface up top. Don't run both `export` *and* `exposing`.

Also: define **`program` vs `module`** — suggest `program` = has `@main`, compiles to an executable; `module`/`package` = library. `[OPEN]`

---

## 24. Visibility

- **Option A — private by default, `export`/`pub` to expose** ★ (safety-friendly)
- **Option B — public by default, `hidden`/`priv` to hide** (Python-ish, leakier)
- **Option C — per-file `exposing(...)` list** controls everything (Elm-style; ties to §23-C)

**★ A** (+ optional C for the file-level summary).

---

## 25. Macros (hygiene & power)

- **Option A — hygienic, declarative pattern macros** ★
  `#name(pattern => expansion)`. Safe (no accidental capture), limited power. Good default.
  (Spelled with the `#` prefix per the §1 decision.)
- **Option B — procedural / `comptime` macros**
  Run code at compile time to emit code. Very powerful, easy to abuse, slower builds.
- **Option C — both**: declarative for common cases, `comptime` proc-macros for advanced.

**★ A** to start; add **B** later behind `comptime` once that's defined.

---

## 26. `comptime` semantics

- **Option A — Zig-style** ★
  `comptime` values/args evaluated at compile time; doubles as the generics mechanism. Powerful, cohesive.
- **Option B — `const fn`-style** (Rust)
  Functions marked evaluable at compile time; narrower, simpler.
- **Option C — macro-only metaprogramming** (no general comptime eval)
  Smallest surface; least powerful.

**★ A** if you skip a full trait system (pairs with §20-D); **B** if you adopt traits (§20-A).

---

## 27. Builtins: keywords vs functions

`typeinfo`, `log`, `len`, `help`, `printd`.

- **Option A — ordinary stdlib functions, not keywords** ★
  `len(xs)`, `log(x)` live in `std`/`prelude`. Keeps the grammar small; users can shadow/extend.
- **Option B — keep as keywords**
  Guarantees availability, but bloats reserved words and blocks user definitions of those names.

**★ A.** Reserve keywords for control flow/declarations only. Standardize on `print`/`println` (drop `printd`).

---

## 28. Naming conventions (lock these in)

- **★ Recommended:** types/traits = `PascalCase`; functions/methods/variables = `snake_case` (enforced); constants `$$` = `SCREAMING_SNAKE_CASE`; modules = `PascalCase`. Document and enforce in the compiler/formatter.

---

## Suggested "house style" bundle (one coherent set)

If you just want a consistent default to start from, take all the ★ picks. In short:
arrow blocks `-> … ;;`, colon types, `$`/`$mut`/`$$` bindings, `//` comments, `#`-prefixed macros, symbols-only
bitwise, `Option`/`Result` + `?`, structs + Rust-style traits + `impl`, ownership/borrow memory
with lowercase `ref()`/`unref()`, `|T: Bound|` generics, path-based imports with private-by-default,
stdlib builtins. That gives a language that reads like "Rust's safety model with Jolt's `$`/`@`/`->`
surface syntax."
