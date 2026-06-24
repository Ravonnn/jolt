# Jolt — Project Index

> **Jolt** is a general-purpose, statically-typed, low-level systems language with modern
> memory safety, compile-time execution, structured concurrency, and a large toolchain. This is the
> front door to the design corpus — what each document is, what's canonical, and where to start.

---

## What Jolt is, in one screen

- **Systems language, safe by default.** Move semantics + a compile-time borrow analysis (**the
  Custodian**) give memory safety with no GC; `[unsafe]` is opt-in, marked, and grantable/deniable.
- **Immutable by default.** `$x` immutable, `$$x` mutable, `[const] $x` compile-time constant.
- **Distinctive surface syntax.** `@fn` functions, `->`/`;;` blocks (blocks are expressions),
  `//`-comments, `#`-macros, `|T|` generics, `@(+)` operator methods.
- **Contracts, not classes.** Trait-style contracts are the only abstraction — static bounds +
  default methods, plus `dyn` for runtime polymorphism. No inheritance.
- **Errors as values.** `!T` (open inferred error set) and `Result<T,E>` (closed), `?` to propagate,
  `??` for `Option` (`Some`/`Nothing`); no exceptions, no null.
- **Compile-time everything.** `comptime` evaluation, value generics (`Array<T, N>`), typed config —
  which **eliminates runtime type confusion** as a vulnerability class.
- **Concurrency + I/O, safe by the type system.** Structured (`scope`/`spawn`) and raw threads; I/O in
  two tiers (completion-based `Io`, green-thread `Fiber`) — no `async`/`await`. `Sendable`/`Shareable`
  contracts make the Custodian reject data races.
- **Capability- & permission-secured.** Static capability attributes (`[noalloc]`, `[constanttime]`,
  …) + Deno-style runtime permission grants, unified end to end including the build sandbox.
- **A huge toolchain.** One `jolt` driver: build, test (incl. deterministic simulation), bench, fmt,
  lint, doc, REPL, profiler, coverage, verify, bindgen, supply-chain, migration, LSP, debugger — all
  over one query-based, aggressively-cached compiler.

```jolt
using Std;
[public]
@main() !None ->
    $nums = [1, 2, 3, 4];
    $evens = nums.iter().filter(|n| -> n % 2 == 0 ;;).collect<Array>();
    println("evens: {evens}");
;;
```

---

## Canonical documents (the current design)

Read these for the current state of the language.

| Doc | What it covers |
| --- | -------------- |
| **`jolt-spec-v0.4.md`** | **The language specification** — the authoritative reference for syntax & semantics. |
| **`jolt-tour.md`** | **A Tour of Jolt** — the friendly, build-as-you-go tutorial for newcomers. Start here to *learn*. |
| **`jolt-cheatsheet.md`** | One-page quick reference for the whole language. |
| **`jolt-grammar.md`** | Formal EBNF grammar (lexical → declarations → expressions → patterns → `build.jolt`). |
| **`jolt-decisions.md`** | The decisions log — every design choice (1–28 + memory) with rationale. |
| **`jolt-stdlib-outline.md`** | The standard library: 20 modules + a low-level/systems layer. |
| **`jolt-toolchain.md`** | Core toolchain: compiler pipeline, diagnostics, build, package manager, fmt, lint, LSP, debug, targets, **security (§13)**. |
| **`jolt-toolchain-extended.md`** | REPL, fix/migrate, profiler, coverage, supply-chain, verify, explain, bindgen, inspection, IDE depth, telemetry. |
| **`jolt-build-system.md`** | `build.jolt` (declarative + comptime) & `jolt.toml` manifest; `jolt new` templates. |
| **`jolt-caching-system.md`** | Aggressive incremental, content-addressed, shareable build cache. |
| **`jolt-testing.md`** | Testing system: macro assertions, property, fuzz, snapshot, mutation, **deterministic simulation**. |
| **`jolt-security-model.md`** | Deny-by-default permissions (runtime + CLI + build), unified with capabilities. |
| **`jolt-safety-attributes.md`** | The capability/safety attribute catalog (`[noalloc]`, `[constanttime]`, …). |
| **`jolt-compiletime-safety.md`** | Why compile-time type/generics/config eliminate runtime type-confusion. |
| **`jolt-implementation-plan.md`** | The build phase: compiler + toolchain implementation plan (Rust stage-0, interpreter-first, Custodian-early, Tiny→Core→Full, self-host). |

---

## Supporting documents (process & rationale)

Useful for understanding *why*, not needed to use the language.

| Doc | What it is |
| --- | ---------- |
| `jolt-changes-v0.4.md` | Changelog v0.3 → v0.4 + the open-question resolutions for that round. |
| `jolt-design-options.md` | The 28-section options menu that drove the early decisions. |
| `jolt-review.md` | The critical review of v0.3 that produced the v0.4 expansion. |
| `jolt-memory-naming-options.md` | The naming-scheme menu that led to "the Custodian." |

---

## Historical / superseded

Kept for provenance; **do not use as current**.

| Doc | Status |
| --- | ------ |
| `jolt-spec-v0.2.md` | Superseded by v0.3 → v0.4. Early consolidation with known contradictions. |
| `jolt-spec-v0.3.md` | Superseded by v0.4. First fully-consistent spec; lacks concurrency/dyn/testing/etc. |

---

## Reading paths

**"I want to learn the language."**
`jolt-tour.md` (the tutorial) → `jolt-cheatsheet.md` → `jolt-spec-v0.4.md` → `jolt-stdlib-outline.md`.

**"I want to understand the design rationale."**
`jolt-decisions.md` → `jolt-design-options.md` → `jolt-review.md` → `jolt-changes-v0.4.md`.

**"I want to build the compiler/tools."**
`jolt-grammar.md` → `jolt-spec-v0.4.md` → `jolt-toolchain.md` (§1 pipeline) → `jolt-caching-system.md`
→ `jolt-toolchain-extended.md`.

**"I care about safety/security."**
`jolt-compiletime-safety.md` → `jolt-safety-attributes.md` → `jolt-security-model.md` →
Custodian (`jolt-spec-v0.4.md` §9) → `jolt-testing.md` §9 (DST).

**"I want to write Jolt programs / set up a project."**
`jolt-build-system.md` → `jolt-toolchain.md` → `jolt-testing.md`.

---

## Design status

| Area | Status |
| ---- | ------ |
| Core language (syntax, types, memory, errors, contracts, generics, comptime) | ✅ decided & specified |
| Concurrency & I/O model | ✅ decided |
| Standard library surface (+ low-level layer) | ✅ outlined |
| Capability & permission/security model | ✅ designed |
| Toolchain (core + extended) | ✅ designed |
| Build system, caching, testing | ✅ designed |
| Formal grammar | ✅ written (accepting grammar; parser-level details noted) |
| **Open design questions** | a handful of small ones, tracked in each doc's "open questions" |

**The build phase:** a reference implementation — lexer → parser (against `jolt-grammar.md`) →
resolver → type checker → the Custodian → capability checker → monomorphizer → MIR → backends. The
plan for this (phases, milestones, module layout, stage-0→self-host path, how each tool slots in) is
in **`jolt-implementation-plan.md`**. Implementation itself has not begun.

---

## Remaining open questions (cross-doc summary)

Small, non-blocking; each lives in its doc's "open questions":
- **Backend strategy** (LLVM-primary + fast debug backend) — `jolt-toolchain.md`.
- **`requires`/`ensures` + SMT** (runtime asserts first vs. proving) — `jolt-toolchain-extended.md`.
- **DST scheduler** (random-by-seed vs. bounded-exhaustive) — `jolt-testing.md`.
- **Mutation operator set, snapshot storage, corpus sharing** — `jolt-testing.md`.
- **Telemetry wire format** (OpenTelemetry vs native) — `jolt-toolchain-extended.md`.
- **bindgen C++ scope** (C-first vs full C++) — `jolt-toolchain-extended.md`.
- **Hash function / cache key normalization, GC policy** — `jolt-caching-system.md`.

The language design itself has no open *forks* — only implementation-level refinements remain.
