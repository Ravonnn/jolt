# Year 4 — Build System, Testing, Security, and the Full Toolchain

> Phases 10–13. Outcome: **Jolt 0.9 (beta)** — the complete language *and* the complete toolchain.
> Everything a developer needs, just not yet self-hosted.

---

## Phase 10 — Build System & Package Manager

**Objective.** Ship the declarative `build.jolt` + `jolt.toml` system and a real package ecosystem.

**Implements (docs).** `jolt-build-system.md` (entire — three levels, `jolt.toml` manifest,
`build.jolt` declaration forms, `jolt new` templates, sandbox); `jolt-toolchain.md` §3 (build &
manifest), §4 (package manager), §11 (targets/cross-compile); `jolt-security-model.md` §build-sandbox
& §package-manager-integration.

**Workstreams**
1. **`jolt.toml` manifest:** package identity, dependencies, features, profiles; resolver + `jolt.lock`.
2. **`build.jolt`:** declarative target/step/`when` forms + `comptime` escape hatch; build sandbox
   (`[noio]` default).
3. **Zero-config builds:** convention-based inference (`src/main.jolt` etc.) + CLI flags;
   `jolt build --explain`.
4. **`jolt new` templates:** exe/lib/staticlib/dylib/freestanding/embedded/kernel/wasm/workspace/
   cffi/cli.
5. **Registry:** index + package hosting; publish/fetch; signing + checksums; `jolt audit`.
6. **Cross-compilation:** `jolt target add`; per-target stdlib flavor selection; firmware emit
   (`bin`/`hex`) + `jolt size`.

**Deliverables.** End-to-end project lifecycle: new → build → depend → publish → cross-compile.

**Definition of Done**
```
[ ] Zero-config build works (no build.jolt); --explain prints the implied build
[ ] Declarative build.jolt drives targets/steps; comptime generate step works; sandbox enforced
[ ] All jolt new templates scaffold buildable projects
[ ] Dependency resolution + jolt.lock reproducible; a published package can be added and built
[ ] Registry serves packages with signature + checksum verification; jolt audit works
[ ] Cross-compile to ≥2 targets incl. an embedded one; size/emit helpers work
```

**Verification Gate.** A multi-package workspace with a registry dependency builds reproducibly;
cross-compiling the embedded template flashes/boots in an emulator; the build sandbox blocks an
un-permitted network fetch in a `generate` step. Sign-off: tooling owner.

---

## Phase 11 — Testing System

**Objective.** Deliver the extensive built-in testing capability per `jolt-testing.md`.

**Implements (docs).** `jolt-testing.md` (entire — §1 declarations, §2 macro assertions, §3 fixtures,
§4 property, §5 fuzz, §6 sanitizers, §7 snapshot, §8 mutation, **§9 DST**, §10 mocking, §11 bench, §12
runner, §13 `Test` module surface); `jolt-stdlib-outline.md` `Test`; `jolt-caching-system.md` §8
(cache-aware test runner).

**Workstreams**
1. **Macro assertions** (source-capturing) + soft `expect_*`; fixtures (`setup`/`teardown` via
   `Dispose`); table-driven `cases`.
2. **Property testing:** `for_all` + shrinking; `Generator`/`Arbitrary` + `#derive(Arbitrary)`.
3. **Fuzzing:** coverage-guided `[fuzz]`; crashes → regression tests; sanitizers (address/UB/thread/
   leak).
4. **Snapshot** + **mutation** testing (mutation score).
5. **Deterministic Simulation Testing:** `[simulation]` + `Sim` (virtual clock, controlled scheduler,
   fault injection); seed-reproducible; time-travel traces.
6. **Runner:** parallel/isolated, cache-aware ("cached pass"), permission-sandboxed per test,
   coverage, `--watch`, JUnit/JSON; `[nostd]`/QEMU harness; `jolt bench` with baselines.

**Deliverables.** A complete testing toolkit, with DST as the flagship.

**Definition of Done**
```
[ ] Macro assertions print values/diffs; soft expects accumulate; fixtures clean up on failure
[ ] Property tests shrink to minimal counterexamples; seeds reproduce exactly
[ ] Fuzzing finds a planted bug and auto-saves a regression; sanitizers catch a planted UB
[ ] Snapshot + mutation testing work; mutation score reported
[ ] DST: a concurrent system test reproduces an injected-fault failure from its seed; trace replays
[ ] Runner is cache-aware (cached pass), permission-sandboxed, parallel; coverage + bench work
```

**Verification Gate.** A deliberately-racy distributed sample is caught by DST and the failure replays
deterministically from its seed; mutation testing reports a meaningful score on the stdlib; coverage
gating fails a sample PR below threshold. Sign-off: testing owner + runtime owner.

---

## Phase 12 — Security & Permissions

**Objective.** Implement the unified capability + Deno-style permission model end to end.

**Implements (docs).** `jolt-security-model.md` (entire — §domains, §CLI grants, §static/dynamic
interplay, §runtime API, §build sandbox, §package integration, §resolved decisions);
`jolt-safety-attributes.md` (the capability catalog); `jolt-compiletime-safety.md` (the proof that
removes runtime checks); `jolt-toolchain.md` §13 (security surface).

**Workstreams**
1. **Static capabilities:** ensure all attributes (`[noio]`/`[nonet]`/`[constanttime]`/…) are proven
   and zero-cost; capability errors name the call chain.
2. **Runtime permissions:** deny-by-default; `--allow-*`/`--deny-*`; `PermissionDenied` as a value;
   `--prompt` (opt-in); permission checks only where not statically eliminated.
3. **Manifest policy:** `jolt.toml [permissions]` ceiling; flags may only narrow; dependency-tree
   capability capping at resolve time.
4. **Build sandbox:** `build.jolt`/`comptime` deny-by-default; per-package build grants.
5. **Runtime API:** `Permission.query/revoke/scoped`; monotonic narrowing; fiber/subprocess
   inheritance.
6. **Supply-chain surfacing:** capability footprint of deps; capability diff on update.

**Deliverables.** A program/runtime/build pipeline governed by one permission model.

**Definition of Done**
```
[ ] A [noio] binary needs no flags and inserts no runtime checks (proven + measured)
[ ] Ungranted access returns PermissionDenied (value); --prompt opts into prompting
[ ] jolt.toml [permissions] caps the dependency tree; over-reaching dep rejected at resolve
[ ] Build sandbox blocks ungranted build-time I/O; build grants are per-package
[ ] Runtime Permission API narrows correctly; children inherit only a subset
[ ] jolt deps --capabilities shows the tree footprint; update flags capability creep
```

**Verification Gate.** A sample app runs least-privilege (only declared grants); a dependency that
adds an `ffi` capability is flagged on update and rejected under a policy that denies `ffi`; static
`[noio]` proof verified to insert zero runtime checks (codegen inspection). Sign-off: security owner +
language lead.

---

## Phase 13 — Extended Toolchain

**Objective.** Ship the remaining developer tools per `jolt-toolchain-extended.md`.

**Implements (docs).** `jolt-toolchain-extended.md` (entire — §1 REPL, §2 fix/migrate, §3 profiler,
§4 coverage, §5 supply-chain, §6 verify, §7 explain, §8 bindgen, §9 inspection, §10 IDE depth, §11
telemetry); `jolt-toolchain.md` §2 (diagnostics for `explain`), §5–§8/§12 (fmt/lint/LSP/debug
baseline); `jolt-stdlib-outline.md` `Log`/`Reflect` (telemetry + bindgen reflection).

**Workstreams**
1. **REPL + interpreter mode** (reuse comptime engine); `:type`/`:caps`/`:allow`.
2. **fix / migrate** (edition rewrites as comptime AST rules); machine-applicable fixes.
3. **Profiler** (CPU/alloc-per-allocator/fiber; flamegraphs; PGO data); **coverage** as a first-class
   tool (line/branch/region, differential).
4. **verify** (capability/contract/bounds; optional SMT); **explain** extended (custody timeline,
   capability chain, DST trace).
5. **bindgen** (consume C; emit C/WASM/FFI); **supply-chain** (`tree`/`why`/`outdated`/`licenses`/
   `sbom`/`vendor`).
6. **Inspection** (`graph`/`bloat`/`timings`/`deadcode`); **LSP depth** (refactors, macro-expand
   preview, custody/capability lenses, DAP debugging); **telemetry** (`[instrument]`/`[metered]`,
   capability-gated, secret-redacting, zero-cost off).

**Deliverables.** The "huge toolchain" complete. **Tag: Jolt 0.9 (beta).**

**Definition of Done**
```
[ ] jolt repl evaluates, inspects types/caps, and grants permissions interactively
[ ] jolt fix applies suggestions; jolt migrate rewrites a sample across an edition boundary
[ ] Profiler produces flamegraphs + per-allocator memory; coverage reports HTML/LCOV with gating
[ ] verify proves a [constanttime] fn and reports bounds-check elision; explain renders a custody timeline
[ ] bindgen wraps a C header and emits a usable C/WASM binding
[ ] Supply-chain tools (tree/why/sbom + capability footprint) work; inspection tools (bloat/graph) work
[ ] LSP offers refactors + macro preview + custody lens + DAP debugging
[ ] Telemetry instruments a fn, redacts [secret], and compiles out when disabled
```

**Verification Gate.** A realistic project uses the full toolchain in one session (write → fmt → lint
→ test → profile → coverage-gate → verify → bloat → publish); the LSP drives an editor with
go-to-def, rename, a refactor, and breakpoint debugging; telemetry adds zero overhead when off
(measured). Beta tagged. Sign-off: tooling owner + language lead.

---

### Year-4 exit state
The full language and the full toolchain — build system, package ecosystem, exhaustive testing
(incl. DST), unified security, and every developer tool — all working, Rust-hosted. Beta-quality.
