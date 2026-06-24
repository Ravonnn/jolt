# Jolt — Extended Toolchain

> Adds eleven capabilities to the core toolchain (`jolt-toolchain.md`), each reusing machinery Jolt
> already has — the query-based compiler, the comptime engine, the capability/permission system, the
> incremental cache, and the WASM target — rather than bolting on independent tools. All are `jolt`
> sub-verbs fronting `libjolt`.

---

## 1. REPL & interpreter — `jolt repl` / `jolt run --interpret`

An interactive evaluator powered by the **comptime engine** (the compiler already has a MIR
interpreter for compile-time execution; the REPL exposes it for runtime use).

- **REPL:** evaluate expressions, define functions/types, inspect values and **types** (`:type expr`),
  see the inferred Custodian/capability state of a binding, and hot-reload edited definitions.
- **Interpreter mode:** `jolt run --interpret` runs a program through the interpreter for instant
  startup (no codegen) — scripting use, quick iteration; the same source compiles AOT unchanged.
- **JIT tier (optional):** hot functions in interpreter mode can be JIT-compiled via the fast backend
  for a middle ground between interpret and full AOT.
- **Capability-aware:** the REPL runs under the permission model (deny-by-default); `:allow net` grants
  interactively. `:` commands: `:type`, `:doc`, `:caps`, `:time`, `:load <file>`, `:reset`, `:allow`.
- **Notebook bridge:** the same evaluator backs a kernel for notebook environments (literate Jolt).

```
> $x = [1,2,3].iter().map(|n| -> n*2 ;;).collect<Array>()
[2, 4, 6] (Array<Int>)
> :type x
Array<Int>
> :caps x
no capabilities required (pure value)
```

---

## 2. Migration & fix — `jolt fix` / `jolt migrate`

The mechanism for evolving code across language `edition`s and deprecations (you have editions +
stable/beta/nightly channels but no migration tool yet).

- **`jolt fix`** applies compiler-suggested fixes en masse: every diagnostic that carries a
  machine-applicable suggestion (unused imports, `[must_use]` results, `borrow` where `claim` isn't
  needed, deprecated API replacements) is auto-rewritten.
- **`jolt migrate --edition 2028`** runs edition-migration rewrites: syntactic/semantic changes
  between editions are encoded as **rewrite rules** (themselves comptime programs operating on the
  AST), applied automatically with a diff preview.
- **Deprecation-driven:** `[{deprecated: "use parse_v2", since, replacement: "parse_v2"}]` carries a
  machine-applicable replacement, so `jolt fix` can rewrite call sites.
- **Safe by construction:** fixes are applied through the parser/AST (not text regex), formatted by
  `jolt fmt`, and the result must still compile — `jolt fix` re-checks before writing.
- `--dry-run` shows the diff; integrates with version control (one commit per fix class).

---

## 3. Profiler — `jolt profile`

CPU, memory, allocation, and concurrency profiling that understands Jolt's models.

- **CPU:** sampling + instrumented profiles, flamegraphs, per-function attribution; `--pgo` emits a
  profile that feeds profile-guided optimization in the LLVM backend.
- **Allocation/memory:** tracks allocations **per allocator** (understands the allocator model) — see
  arena vs default vs pool usage, peak/leak, fragmentation; flags allocations in `[noalloc]`-eligible
  paths.
- **Concurrency:** fiber/task profiling — where fibers park, channel contention, lock wait times,
  scheduler latency; pairs with the structured-concurrency model.
- **Deterministic profiling under simulation:** profile inside a DST run (`jolt-testing.md` §9) for
  reproducible performance numbers free of wall-clock noise.
- Outputs: flamegraph (SVG/HTML), `pprof`-compatible, Chrome trace; integrates with `jolt bench`
  baselines.

---

## 4. Coverage — `jolt coverage`

Promotes the `jolt test --coverage` flag to a first-class tool.

- **Line, branch, and region** coverage; uses the compiler's own instrumentation (accurate, not
  guessed from debug info).
- **Reports:** terminal summary, HTML (annotated source), LCOV/Cobertura for CI dashboards.
- **Gating:** `jolt coverage --min-line 80 --min-branch 70` fails CI below thresholds; per-package
  and per-file thresholds.
- **Differential coverage:** `--diff` reports coverage *of the changed lines only* (great for PR
  gating), using the same incremental dep info as the cache.
- Combines unit + doctest + integration + property/fuzz runs into one merged report.

---

## 5. Supply-chain & dependency tooling

Leans hard into Jolt's **capability transparency** — this is a differentiator, not just parity.

| Command | Purpose |
| ------- | ------- |
| `jolt tree` | visualize the dependency graph (with versions, features) |
| `jolt why <dep>` | explain why a dependency is in the tree (paths to it) |
| `jolt outdated` | show deps with newer versions; flag breaking vs compatible |
| `jolt audit` | check against the advisory database (existing) |
| `jolt deps --capabilities` | **aggregate permission footprint** of the whole tree — which deps want net/fs/ffi/run, transitively |
| `jolt licenses` | collect/validate dependency licenses; policy enforcement |
| `jolt sbom` | emit a Software Bill of Materials (SPDX/CycloneDX) |
| `jolt vendor` | vendor dependencies for air-gapped/embedded builds |

- **Capability diffing on update:** `jolt update` shows if a new version of a dep *requests more
  capabilities* than the old ("`http@1.3` now wants `run` — review?"), surfacing supply-chain creep
  at the point of change.
- Ties to `jolt.toml [permissions]`: a dep whose footprint exceeds project policy is rejected at
  resolve time (security model §package integration).

---

## 6. Static analysis & verification — `jolt verify`

A deeper analysis layer beyond `jolt lint`, leaning into the safety identity.

- **Capability verification:** prove `[constanttime]` (no secret-dependent branches/indexing),
  `[noalloc]`/`[nopanic]`/`[bounded_stack]` claims — already enforced by the compiler; `jolt verify`
  reports *why* and surfaces near-misses.
- **Contracts as pre/post-conditions:** optional `requires(cond)` / `ensures(cond)` on functions,
  checked statically where decidable (and as debug assertions otherwise) — design-by-contract.
- **Bounds-check analysis:** report which array/slice accesses the compiler proved safe (elided
  checks) vs. which retain a runtime check — actionable for hot paths.
- **Panic/abort reachability:** prove a function or `[nopanic]` region can never trap.
- **Optional SMT backend:** for `requires`/`ensures` and bounds proofs that need a solver
  (opt-in, since it's heavier) — `jolt verify --smt`.
- **Side-channel & taint:** verify `[secret]`/`[tainted]` data-flow (no secret reaches I/O without
  declassification).

---

## 7. `jolt explain` — interactive diagnostics

Exists for error codes; extended into a teaching tool for Jolt's hardest concepts.

- `jolt explain E0123` — full write-up with examples (existing).
- **Custodian explainer:** `jolt explain --last` reconstructs the **ownership/borrow timeline** of the
  most recent custody violation — where the value was created, moved, borrowed, and where the conflict
  is — as a step-by-step narrative or a visual.
- **Capability-chain explainer:** expands a capability error into the full call path
  (`hot_path → build_list → alloc`).
- **DST failure explainer:** replays a simulation failure's event timeline (`--trace`) with virtual
  timestamps, scheduling decisions, and injected faults annotated.
- Machine-readable (`--format json`) so the LSP can render these inline.

---

## 8. Cross-language bindings — `jolt bindgen`

Two directions, feeding/served-by the `Abi` module.

- **Consume C/C++:** `jolt bindgen header.h` generates typed Jolt `extern` declarations + `[repr: C]`
  structs from C/C++ headers, so wrapping a native library is one command. Honors the `ffi`
  permission.
- **Emit for others:** generate **C headers** from a Jolt library's `[public, extern: "C"]` surface;
  **WASM bindings** (JS glue + `.d.ts`) for the wasm32 target; **Python/other FFI** stubs.
- **Comptime-driven:** binding generation is a comptime program over the type's reflection
  (`typeinfo`), so it's deterministic and cacheable, and custom binding emitters can be written in
  Jolt.

---

## 9. Build graph & artifact inspection

Visibility into what the build produces and why.

| Command | Purpose |
| ------- | ------- |
| `jolt graph` | render the build/dependency graph (steps, targets, cache hits) |
| `jolt bloat` | binary-size attribution by function/module/crate — essential for embedded |
| `jolt size` | section sizes (text/data/bss) for firmware images (existing, expanded) |
| `jolt asm` / `jolt ir` | per-function assembly / MIR dump (existing) + **size & cost annotations** |
| `jolt timings` | per-query build timing + cache hit/miss breakdown (from the cache system) |
| `jolt deadcode` | report unreachable/unused public items across the workspace |

- **`jolt bloat`** is especially valuable given the embedded focus: it attributes every byte of the
  output to source, with `--diff` against a baseline to catch size regressions in CI.
- All read the query graph + CAS, so they're cheap (no extra full build).

---

## 10. Editor / IDE depth (`jolt lsp` extensions)

Beyond the baseline LSP (completions, hover, go-to-def, rename, inlay hints):

- **Semantic refactorings:** extract function, inline, **change-signature with call-site updates**,
  introduce/merge variable, convert closure↔function, derive a contract impl, add `[must_use]`.
- **Macro-expansion preview:** expand a `#macro`/proc-macro/`[attr]` inline to see generated code.
- **Custodian visualization:** hover a binding to see its borrow/move state and lifetime extent
  (inlay-rendered, from §7's machinery).
- **Capability lens:** show a function's inferred capability footprint inline; one-click to tighten
  (`add [noalloc]`).
- **Debug adapter (DAP):** wire `jolt debug` into the editor with the Jolt-aware pretty-printers and
  ownership view.
- **Test/bench/sim lenses:** run/debug a `[test]` or replay a `[simulation]` seed from a gutter
  action.
- **Comptime evaluation preview:** show the value a `comptime` block computes, inline.

---

## 11. Telemetry & observability hooks

Built-in, capability-gated, zero-cost-when-off.

- **`[instrument]` attribute:** auto-wire a function into tracing — entry/exit spans, args (redacting
  `[secret]` fields), duration — feeding the `Log` module's structured sinks.
- **Metrics API:** counters/gauges/histograms in `Log`/`Observe`; an `[metered]` attribute auto-emits
  call-count/latency metrics.
- **Distributed tracing:** span context propagates across fibers, channels, and (via headers) network
  calls; OpenTelemetry-compatible export.
- **Capability-gated:** emitting telemetry to the network/disk requires the matching permission, so
  observability can't become an exfiltration path; `[secret]`/`[tainted]` data is auto-redacted from
  spans and logs.
- **Zero-cost when disabled:** instrumentation compiles out entirely when the telemetry feature is
  off (comptime-gated), so `[noalloc]`/realtime builds aren't penalized.

---

## How these reuse existing machinery

| New tool | Reuses |
| -------- | ------ |
| REPL / interpreter | the comptime MIR interpreter; permission model |
| fix / migrate | parser/AST + comptime rewrite rules; `jolt fmt` |
| profiler | compiler instrumentation; allocator model; fiber runtime; DST clock |
| coverage | compiler instrumentation; incremental dep info |
| supply-chain | capability transparency; `jolt.toml [permissions]`; resolver |
| verify | capability passes; `typeinfo`; optional SMT |
| explain | Custodian/capability passes; DST traces; LSP rendering |
| bindgen | `Abi`; `typeinfo` reflection; comptime; WASM target |
| inspection | query graph + CAS (no extra build) |
| IDE depth | `libjolt` front end; DAP; comptime preview |
| telemetry | `Log`/`Observe`; capabilities (redaction + gating); comptime gating |

Nothing here is a standalone tool — each is a projection of `libjolt` + the language's own features,
which keeps the toolchain coherent and the tools always consistent with the compiler.

---

## Open questions

1. **REPL state model** — how do redefinitions interact with the Custodian (re-binding moved values)?
   Lean: REPL bindings are a fresh scope each line, prior values borrowable but not moved-from.
2. **`requires`/`ensures` scope** — ship as runtime debug-assertions first, add SMT proving later?
3. **SMT dependency** — bundle a solver or make `jolt verify --smt` an optional component?
4. **Telemetry vendor neutrality** — OpenTelemetry as the wire format, or a Jolt-native format with
   exporters?
5. **bindgen C++ scope** — full C++ (hard) or C + `extern "C"` C++ surfaces (pragmatic) first?
