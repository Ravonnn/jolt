# Jolt — Compile-Time Safety Guarantees

> Type validation, generic instantiation, and configuration all happen at **compile time**. This
> moves an entire class of bugs — runtime type confusion, unchecked casts, configuration drift —
> out of the runtime, where they become attack surface, and into the compiler, where they become
> rejected programs. This note states the guarantee precisely, scopes it, and shows how each piece
> is enforced.

---

## 1. The guarantee

**If a Jolt program compiles, it is free of type-confusion at runtime.** There is no runtime type
coercion that can reinterpret one type's bytes as another's, no implicit narrowing, no null
masquerading as a value, no generic specialized to the wrong type, and no "stringly-typed"
configuration parsed late and wrong. Every type relationship is decided and checked before a single
instruction runs.

Type confusion is one of the most damaging vulnerability classes in systems software (it underlies a
large share of memory-corruption CVEs: a value used as the wrong type leads to out-of-bounds access,
forged pointers, or control-flow hijack). Jolt removes the preconditions for it by construction.

---

## 2. What happens at compile time (and therefore cannot go wrong at runtime)

### 2.1 Type validation
- **Static, total type checking.** Every expression has a type known at compile time; every use is
  checked against it. There is no `any`/dynamic-by-default type.
- **No implicit conversions.** Widening and narrowing both require explicit `as`; there is no silent
  int↔float, signed↔unsigned, or pointer↔integer coercion that could reinterpret a value.
- **No null.** Absence is `Option<T>` (`Some`/`Nothing`); a `T` is always a valid `T`. The classic
  "null where an object was expected" confusion cannot arise.
- **Checked downcasts only.** Going from `dyn Contract` to a concrete type uses `as`, which yields an
  `Option` (`Some(c)` / `Nothing`) — a *checked* operation, never a blind reinterpret-cast. There is
  no C-style `(Type*)ptr`.
- **Exhaustive matching.** `match` on closed types is exhaustiveness-checked, so a value can't fall
  through into code that assumes the wrong variant.

### 2.2 Generics
- **Monomorphized at compile time.** A generic `|T|` function is specialized to each concrete `T`
  *before* codegen; there is no runtime type parameter to get wrong, no type erasure, no boxing-and-
  hoping. `largest<Int>` and `largest<String>` are distinct, fully-checked functions.
- **Contract bounds verified at instantiation.** `|T: Comparable|` is proven for each concrete `T`
  when it's instantiated — you cannot instantiate a generic with a type that lacks the required
  operations, so the body never performs an operation the type doesn't support.
- **Value generics checked too.** `Array<Int, 16>` carries its length in the type; a length mismatch
  is a compile error, not a runtime buffer overrun. Comptime guards (`{N > 0}`) become compile-time
  constraints.

### 2.3 Configuration
- **Configuration is typed and resolved at compile time**, not parsed from strings at startup:
  - `[cfg: …]` conditional compilation, feature flags, and target queries (`target.os`,
    `pointer_width`) are evaluated by the compiler — a misspelled feature or impossible config is a
    build error, not a silent no-op at runtime.
  - **`comptime`** turns runtime configuration parsing into compile-time computation: lookup tables,
    schema-derived code, and constant config are built and *type-checked* during compilation. A
    malformed comptime config fails the build.
  - Build options (`build.jolt`, `jolt.toml [features]`) are typed; an invalid option value is
    rejected before anything runs.
- The effect: **no "configuration drift"** where a string config key, env var, or feature toggle is
  read at runtime, mistyped, and silently mishandled — the canonical source of late-binding security
  holes.

---

## 3. Why this eliminates the vulnerability class

| Classic runtime failure | Root cause | Removed by |
| ----------------------- | ---------- | ---------- |
| reinterpret-cast corruption | unchecked `(T*)` casts | no blind casts; `dyn`→concrete is checked `Option` |
| integer truncation/overflow surprise | implicit narrowing | explicit `as` + defined overflow policy |
| null-pointer deref | null in the type system | no null; `Option<T>` |
| wrong enum variant handled | non-exhaustive switch | exhaustive `match` on closed types |
| generic used at wrong type | type erasure / runtime generics | monomorphization + bound-checking at instantiation |
| buffer length mismatch | length tracked at runtime, if at all | value generics encode length in the type |
| config key typo → wrong behavior | stringly-typed late config | typed, compile-time `[cfg]`/`comptime`/options |

Each row is a category of exploit precondition that simply has no representation in a compiling Jolt
program.

---

## 4. How it's enforced (pipeline view)

These guarantees are produced by specific, ordered compiler passes (see `jolt-toolchain.md` §1):

```
type checker      → every expression typed; no implicit conversions; no null
monomorphizer     → generics specialized; contract bounds & value-generic/comptime guards proven
comptime engine   → configuration computed & type-checked at build time
capability check  → I/O/alloc/etc. constraints verified (defense in depth)
Custodian         → memory/aliasing safety (complements type safety)
```

A program that would exhibit type confusion is **not representable as a passing compilation** — the
relevant pass rejects it.

---

## 5. Scope & honest limits

- **Inside `[unsafe]`**, the programmer can use raw `Pointer<T>` and reinterpret memory; type
  confusion *is* possible there. That is the point of `[unsafe]`: the dangerous operations are
  syntactically marked, auditable, and grantable/deniable (a deployment can forbid `unsafe`
  entirely). Outside `[unsafe]`, the guarantee is total.
- **FFI (`extern`/`c {}`)** crosses into untyped foreign code; the boundary is typed on the Jolt
  side, but Jolt cannot vouch for what the foreign side does with the bytes. FFI requires the `ffi`
  permission (security model), so it's gated, not ambient.
- **Logic bugs are out of scope.** Eliminating type *confusion* does not eliminate incorrect *logic*;
  a program can still compute the wrong answer with perfectly well-typed values.

The honest framing: Jolt makes type confusion **impossible in safe code, marked-and-gated in unsafe
code, and contained at FFI boundaries** — which removes it as a pervasive, ambient hazard and
confines it to small, reviewable surfaces.

---

## 6. Relationship to the rest of the design

- **Complements the Custodian.** The type system prevents *what* a value is from being confused; the
  Custodian prevents *when/where* it may be accessed from being violated. Together they cover the two
  halves of memory safety (type safety + aliasing/lifetime safety).
- **Reinforced by capabilities & permissions.** Even a correctly-typed program is held to its
  declared I/O/alloc capabilities and runtime permission grants — layered defense.
- **Enables aggressive caching safely.** Because all of this is decided at compile time and is
  deterministic, results are cacheable and reproducible (see `jolt-caching-system.md`); compile-time
  decisions don't just improve safety, they're what makes the incremental cache sound.
