# Year 5 — Self-Hosting and 1.0

> Phases 14–15. Outcome: **Jolt 1.0** — a self-hosted, stable language with a living ecosystem.

---

## Phase 14 — Self-Hosting

**Objective.** Rewrite the Jolt compiler in Jolt and prove it correct via the three-stage bootstrap.

**Implements (docs).** `jolt-implementation-plan.md` §3 Phase H & §6 (self-hosting strategy,
three-stage bootstrap); `jolt-caching-system.md` §9 (reproducibility — basis of bootstrap equality);
`jolt-testing.md` §9 (DST on the compiler's own parallelism); the full `libjolt` architecture in
`jolt-implementation-plan.md` §1 (the modules being ported).

**Workstreams**
1. **Port `libjolt` to Jolt**, module by module, in dependency order (lexer → parser → resolver →
   types → **Custodian** → capability → comptime → mono → MIR → backends), each compiled by the Rust
   stage-0 and tested against the *same* suites.
2. **Hardest ports last/most-tested:** the Custodian and comptime engine carry the most semantics;
   give them the deepest coverage.
3. **Three-stage bootstrap:** stage-0(Rust) → `compiler_A`; `compiler_A` → `compiler_B`; `compiler_B`
   → `compiler_C`; require `compiler_B` ≡ `compiler_C` (byte-identical).
4. **Performance parity:** the Jolt compiler must be within an acceptable factor of stage-0 (it's
   now also the language's largest dogfood program — use the profiler/cache to close gaps).
5. **Retire stage-0** to archive (kept only for from-scratch platform bring-up).
6. **DST on the compiler's own concurrency** (parallel queries + codegen) for determinism.

**Deliverables.** A Jolt compiler written in Jolt that compiles itself reproducibly.

**Definition of Done**
```
[ ] Every libjolt module ported to Jolt and passing the existing test suites
[ ] Three-stage bootstrap: compiler_B and compiler_C are byte-identical
[ ] The self-hosted compiler passes the full UI/run/custody/capability/property/fuzz suites
[ ] Compiler performance within target factor of stage-0; regressions profiled and addressed
[ ] Parallel compilation passes DST (deterministic under simulated scheduling)
[ ] Rust stage-0 archived with documented bring-up instructions
```

**Verification Gate.** Bootstrap equality (`compiler_B ≡ compiler_C`) holds; the self-hosted compiler
reproduces every prior test result; a clean-machine bootstrap (stage-0 → self-host) is reproducible.
**This is the credibility gate.** Sign-off: language lead + build/infra owner + independent reviewer.

---

## Phase 15 — 1.0 Release & Ecosystem

**Objective.** Stabilize, document, and seed the ecosystem for a 1.0 anyone can rely on.

**Implements (docs).** `jolt-index.md` (the doc set to finalize); `jolt-tour.md` + `jolt-cheatsheet.md`
+ `jolt-spec-v0.4.md` (polish to 1.0); `jolt-toolchain.md` §12 (`joltup`, channels, editions);
`jolt-toolchain-extended.md` §8 (playground via WASM bindgen); every doc's "open questions"/"resolved
decisions" sections (finalize or defer with rationale).

**Workstreams**
1. **Language stabilization:** freeze the `2026` edition; finalize any remaining open questions
   across the design docs; publish a stability guarantee + edition policy.
2. **Documentation:** polish spec/tutorial/cheatsheet; auto-published API docs (docs.jolt.dev-style)
   with runnable doctests; migration guides (from Rust/C/Go).
3. **Playground:** WASM-based in-browser compiler/runner for shareable snippets.
4. **Ecosystem seeding:** a curated set of first-party packages (http, json, cli args, async patterns,
   common embedded HALs); a package-discovery site; `joltup` distribution on major OSes.
5. **Hardening:** fuzz/DST the compiler and stdlib at scale; security review of the permission model
   and `[unsafe]` surface; performance pass (compile-time and runtime).
6. **Release engineering:** signed releases, reproducible from source, supported-platform matrix,
   support/EOL policy.

**Deliverables.** **Jolt 1.0** — stable, documented, distributable, with a starter ecosystem.

**Definition of Done**
```
[ ] 2026 edition frozen; stability + edition policy published
[ ] All design-doc open questions resolved or explicitly deferred with rationale
[ ] Spec/tutorial/cheatsheet complete; API docs auto-published with passing doctests
[ ] Playground compiles & runs snippets in-browser
[ ] joltup installs Jolt on Linux/macOS/Windows; signed, reproducible releases
[ ] Starter package set published and buildable; registry production-ready
[ ] Compiler + stdlib pass large-scale fuzz/DST campaigns; security review complete
[ ] Supported-platform matrix + support policy published
```

**Verification Gate.** A new user can install via `joltup`, follow the tour, and ship a cross-compiled
program using registry packages — all on a clean machine. The compiler/stdlib survive a sustained
fuzz/DST campaign with no open criticals. Releases are signed and reproducible. 1.0 tagged. Sign-off:
language lead + security owner + release owner.

---

### Year-5 exit state — Jolt 1.0
A self-hosted, stable, fully-documented systems language with a complete toolchain (build, test incl.
DST, profile, verify, bindgen, LSP, debugger, supply-chain), a unified safety/security model, an
incremental reproducible compiler, and a starter ecosystem — built and verified phase by phase from
the ground up.

---

## Beyond 1.0 (backlog, not scheduled)
- Additional backends (Cranelift fast-debug productionized; GPU; more targets).
- Distributed/remote build cache at scale.
- Async via the same fiber model if demand emerges (kept out by design so far).
- SMT-backed `requires`/`ensures` verification productionized.
- IDE ecosystem depth (JetBrains, more refactors), package docs hosting, governance/RFC process.
