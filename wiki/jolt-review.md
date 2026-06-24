# Jolt v0.3 — Review: What to Improve & Expand

> A critical pass over the v0.3 spec. Three buckets: **(A) internal tensions** (things already in the
> spec that fight each other or are underspecified), **(B) missing core features** (a systems
> language is expected to have these and they're absent), and **(C) expansion ideas** (optional,
> would strengthen the language). Each item notes severity and a concrete suggestion.
>
> Severity: 🔴 blocking-ish (will bite early) · 🟡 important · 🟢 nice-to-have.

---

## A. Internal tensions & underspecified corners

### A1. 🔴 The `;` vs `;;` rule is ambiguous in one-liners
`@double(x: Int) Int -> x * 2 ;;` ends a block with `;;`, but single statements end with `;`. In
`if done -> break; ;;` we see `break;` then `;;`. Yet `@convert(...) Float64 -> m / 2.54 ;;` has no
inner `;`. So is the body `m / 2.54` an expression (no `;`) or a statement (needs `;`)? **Define:**
either (a) `->` blocks always contain statements each ended by `;` and `;;` closes the block, or
(b) a single expression after `->` is allowed without `;`. Pick one; right now examples do both.

### A2. 🔴 `$`/`$$` sigil collides conceptually with "mutable = `$$`"
`$$` means *both* "mutable binding" (§4) and appears in `$$self`, `$$age`, `$$x`. That's consistent.
But the precedence table (§6.1) lists `$expr` and `$$expr` as **postfix unary operators**, and the
macro section uses `$tmp` for macro-introduced bindings. So `$` is a binding sigil, a unary operator,
*and* a macro variable marker. **Clarify** what `$expr`/`$$expr` as operators actually do — this is
never explained anywhere in the spec body. (Best guess from old drafts: variable *interpolation* or
"denote a binding," but it needs a definition or removal.)

### A3. 🟡 Mutable fields vs immutable bindings interaction
`struct Person -> $name: String; $$age: Uint; ;;` — `age` is a mutable field. But if I bind
`$p = Person{...}` (immutable binding), can I still mutate `p.age`? In Rust, mutability is the
binding's property, not the field's. Jolt puts it on the field. **Define** the rule: does field-level
`$$` override binding-level immutability, or must *both* the binding and field be mutable? This
materially changes the mental model.

### A4. 🟡 Multiple-dispatch guards reference parameters that aren't bound
`@convert(m: Float64, {unit == "inch"})` — `unit` appears in the guard but is **not a parameter**.
Where does `unit` come from? Is it a second parameter whose name was omitted, a field of `m`, or an
outer variable? **Fix the example and define** guard scope: guards presumably see the function's
parameters, so the signature needs a `unit` parameter: `@convert(m: Float64, unit: String, {unit == "inch"})`.

### A5. 🟡 `switch`/`match` overlap is still thin
The division ("constants vs patterns") is stated but the line blurs: matching an enum *variant* with
no payload is both "a constant" and "a pattern." **Decide** whether `switch` is just sugar for a
`match` with only literal arms, or a genuinely separate construct, and whether `switch` can match
strings/ranges. If it can't justify itself, fold it into `match` (the earlier redundancy note flagged
this; it survived into v0.3).

### A6. 🟡 `None` the type vs `None` the Option case
`@main() !None` uses `None` as a unit/void return type (§1), but §10 uses `None` as the empty
`Option` case (`Some`/`None`). These are two different `None`s. **Rename one** — conventionally the
void type is `Unit` or `()` and the Option case stays `None`. As written, `!None` reads as "Option of
nothing," which isn't the intent.

### A7. 🟢 Unicode math aliases interact with `^`
`√16.0` is an alias for `sqrt`, fine. But `≈` ("approx-equal helper") and `≡` ("identity helper")
have no defined semantics or signature. **Define or drop.** Floating approx-equality needs a tolerance
parameter, which an operator can't easily carry.

### A8. 🟢 `as_int`/`as_bytes` union fields use no sigil
`union Word -> as_int: Uint32; as_bytes: Array<Byte>; ;;` — struct fields use `$`/`$$` but union
fields don't. **Make consistent** with struct field syntax.

---

## B. Missing core features (a systems language needs these)

### B1. 🔴 No concurrency / threading model
The spec mentions `[shared, sync]` for atomic refcounts but there is **no way to spawn a thread,
no async, no channels, no mutex, no atomics**. For a modern systems language this is the biggest
hole. **Proposal:** decide the concurrency story now because it shapes the type system:
- structured concurrency (`spawn`/`scope`) vs async-await vs both;
- a `Send`/`Sync`-style contract pair (ties beautifully into the contract system) so the borrow
  checker can reject data races at compile time;
- channels and a `Mutex<T>`/`Atomic<T>` in stdlib.
This is the natural place for Jolt to differentiate (Rust's `Send`/`Sync` + structured concurrency,
made lighter).

### B2. 🔴 No trait objects / dynamic dispatch escape hatch
Contracts are static-only (§15). That's great for zero-cost, but *every* real program eventually
needs a heterogeneous collection (`Array` of "things that are Drawable"). With no `dyn`-equivalent,
you can't write that. **Proposal:** add an opt-in dynamic form, e.g. `dyn Drawable` or `Drawable?`
boxed object, explicitly separate from static bounds so the cost is visible. Without it, contracts
are strictly less expressive than they need to be.

### B3. 🟡 No `Drop`/destructor mechanism
Memory is freed automatically (§9), but what about *other* resources — file handles, sockets, locks?
`defer` handles per-scope cleanup, but value types that own a resource need deterministic
destruction when they go out of scope or are dropped. **Proposal:** a `Drop` contract with a
`@drop($$self)` method the compiler calls automatically at end of ownership (RAII). This pairs with
the allocator model.

### B4. 🟡 Pattern matching is shallow
`match` destructures enum variants (§8.2) but there's no shown support for: nested patterns,
literal patterns mixed with bindings, guards on arms (`Circle(r) if r > 0 ->`), `or`-patterns,
binding the whole value (`x @ Circle(r)`), or matching on tuples/structs. **Proposal:** flesh out a
real pattern grammar — this is high-leverage because `match` is the primary control structure.

### B5. 🟡 No iterator / range protocol shown
`for i in 0..10` works and `for x in iterable` is mentioned, but what makes something iterable?
There's no `Iterator`/`Iterable` contract. **Proposal:** define an `Iterator` contract (`@next($$self) T?`)
that `for` desugars to. This also makes `..` ranges just a stdlib `Iterator`, and lets users write
custom iterables. Lazy iterator adapters (`map`/`filter`/`take`) then come for free.

### B6. 🟡 String handling is underspecified
`String` is "UTF-8 bytes" and `Char` is a 32-bit scalar — but how do you index, slice, iterate
chars vs bytes, concatenate (is `+` defined?), format/interpolate? The examples use
`"a" + to_string(x)` which is verbose. **Proposal:** string interpolation (e.g. `"x = {x}"`),
define `+` or a `concat`, and specify byte-vs-char iteration explicitly given the UTF-8 decision.

### B7. 🟡 No visibility granularity
`[public]` is all-or-nothing (§13). Systems code usually wants "public to my package but not the
world" (Rust's `pub(crate)`). **Proposal:** `[public]` and `[public: package]` (or similar), so
library-internal APIs don't leak to dependents.

### B8. 🟢 No const generics / value generics
`@pow_of_two(comptime n: Uint)` shows comptime *args*, but you can't parameterize a *type* by a
value (e.g. `Array<Int, 16>` fixed-size array, `Matrix<3, 3>`). **Proposal:** allow comptime values
in `|…|`: `struct Vec |T, comptime N: Uint| -> ...`. Big for low-level/embedded work.

### B9. 🟢 No explicit integer-overflow / numeric-conversion policy
For a low-level language this must be pinned: does `Int` arithmetic wrap, trap, or saturate? Are
narrowing conversions implicit or required via `as`? **Proposal:** trap on overflow in debug, wrap in
release (or provide `wrapping_add`/`checked_add` like Rust), and require explicit `as` for any lossy
conversion. The `as` operator already exists in the precedence table but is never defined.

### B10. 🟢 No FFI detail beyond inline `c {}`
Inline C blocks exist (§19) but there's no story for *calling* external C functions with proper
signatures, linking, or `extern`. **Proposal:** an `extern` declaration form and a calling-convention
attribute (`[extern: "C"]`).

---

## C. Expansion ideas (optional, raise the ceiling)

- **C1. 🟢 Named & default arguments.** `connect(host, port = 8080)`. Reduces overload/guard pressure.
- **C2. 🟢 Tagged/labeled loop break.** `break outer;` for nested loops — small, high-value.
- **C3. 🟢 `match` as an expression.** `$x = match c -> ... ;;` returning a value, not just control flow.
  (Most modern languages make `if`/`match` expressions; the spec treats them as statements.)
- **C4. 🟢 Block-as-expression.** Let `-> ... ;;` blocks yield a value (last expression), enabling
  `$x = -> compute(); ;;`. Pairs with C3.
- **C5. 🟢 Operator overloading via contracts.** Define `Add`/`Mul` contracts so `+`/`*` work on user
  types. Natural fit with the contract system and needed for `Complex`/`Rational`/`Matrix` to be
  ergonomic.
- **C6. 🟢 Slices / views.** A `Slice<T>` (pointer + length) view into arrays without copying —
  fundamental for systems perf and pairs with the borrow checker.
- **C7. 🟢 Compile-time reflection surface.** `typeinfo` exists; expand it into a defined comptime
  reflection API (field iteration, type names) so serializers/ORMs can be written as comptime code.
- **C8. 🟢 Testing & tooling hooks in-language.** A `[test]` attribute and `assert` were in the early
  drafts; reintroduce a first-class test story given "huge toolchain" is a stated goal.
- **C9. 🟢 Documentation example execution.** Since `///` doc comments feed a doc generator, allow
  runnable examples in docs (doctests) — strong toolchain differentiator.
- **C10. 🟢 Effect/capability annotations.** Longer-term: mark functions that allocate, do I/O, or can
  panic, so embedded/realtime users can forbid them. Fits the attribute system.

---

## Top recommendations (if you only do a few)

1. **Concurrency model + `Send`/`Sync` contracts (B1)** — biggest gap, biggest differentiation
   opportunity, and it must inform the type system early.
2. **Dynamic dispatch escape hatch (B2)** — contracts are incomplete without it.
3. **`Drop`/RAII contract (B3)** — resource safety is half the point of the memory model; right now
   only memory is handled, not files/sockets/locks.
4. **Flesh out pattern matching (B4) + make `match`/`if` expressions (C3/C4)** — `match` is the core
   control structure; it's currently too weak.
5. **Resolve the `;`/`;;` rule and the `None` type-vs-Option-case clash (A1, A6)** — these are small
   but will confuse from line one.
6. **Iterator contract (B5) + operator overloading (C5)** — together these make the stdlib and user
   types feel first-class instead of special-cased.

These don't conflict with any locked decision; they fill space the spec simply hasn't reached yet.
The memory model, error model, and contract system are solid foundations — the gaps are mostly
"breadth not yet covered" rather than "wrong choices."
