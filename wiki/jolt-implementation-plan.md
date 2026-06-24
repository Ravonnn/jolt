# Jolt — Compiler & Toolchain Implementation Plan

> The build phase: how to go from the design corpus (`jolt-index.md`) to a working compiler and
> toolchain. Decisions locked for this plan: **stage-0 in Rust**, **self-host eventually** (not
> early), **interpreter-first** backend, **Custodian built early**, **Tiny→Core→Full** language
> growth, **compiler before tools** (with `fmt` + test runner pulled early).
>
> Source-of-truth references: grammar → `jolt-grammar.md`; semantics → `jolt-spec-v0.4.md`;
> pipeline/cache/security → the toolchain/caching/security docs.

---

## 0. Strategy at a glance

```
Stage 0 (Rust):  build a working Jolt compiler + core tools in Rust.
                 Grow the language Tiny → Core → Full.
                 De-risk the Custodian as early as possible.
                 Interpreter executes code first; LLVM backend added once the front end is proven.

Stage 1 (port):  once the language is stable & self-capable, port the compiler to Jolt (self-host),
                 keeping the Rust compiler as the bootstrap until the Jolt one passes the same tests.

Throughout:      query-based architecture from day one (so incrementality/caching/LSP fall out),
                 and the compiler is itself tested with the testing system it enables (incl. DST).
```

The guiding rule: **build the novel, risky parts on the smallest possible language**, prove them,
then scale the language up underneath an already-validated pipeline.

---

## 1. `libjolt` architecture (the shared core)

Everything — CLI, LSP, fmt, tools — calls one library. Build it query-based from the start.

```
libjolt/
  source/        SourceMap, file ids, spans, content hashing
  lexer/         tokens (max-munch rules: >>> vs >> vs >, etc.)
  parser/        AST per item; error-recovery productions for the LSP
  ast/           AST types + visitor
  query/         the query engine: memoization, dependency tracking, red/green invalidation
  resolve/       names, modules (library/package/program), imports, visibility
  types/         type representation, inference, contract resolution
  custody/       THE CUSTODIAN — ownership/move/borrow analysis  ← highest risk, built early
  capability/    [noalloc]/[noio]/... transitive checking
  comptime/      MIR interpreter (powers comptime AND the runtime interpreter & REPL)
  mono/          monomorphization; value-generic + comptime-guard resolution
  mir/           mid-level IR + lowering (desugar match, loops, defer/errdefer, dispose)
  backend/
    interp/      MIR interpreter execution (FIRST backend)
    llvm/        LLVM codegen (SECOND backend)
    fast/        in-house fast debug backend (LATER)
  diagnostics/   structured errors, codes, suggestions, JSON output
  cache/         content-addressed store + query-graph persistence
```

**Why query-based from day one:** incrementality, the cache, and the LSP are all just consequences
of memoized content-hashed queries. Retrofitting this later is a rewrite; doing it first is nearly
free and shapes every module.

---

## 2. Language growth: the three subsets

Each subset is a complete, compilable language — never a half-working one.

**Tiny** (first end-to-end target)
- `@fn` with params + return, `Int`/`Bool`, arithmetic/comparison/logic operators
- `$`/`$$` bindings, `if/else`, `loop`/`for`, `return`/`break`/`next`
- `->`/`;;` blocks (incl. block-as-expression), `//` comments
- function calls; `println` as a builtin
- **move semantics + the Custodian on this subset** (the whole point of going Tiny early)
- No generics, contracts, structs, concurrency, or stdlib beyond a stub prelude.

**Core** (a real, minimal language)
- `struct`/`enum`/`union`, methods (`Type::method`), pattern matching (`match`, full patterns)
- `Option`/`Result`, `?`/`??`, `error`/`defer`/`errdefer`
- basic generics (`|T|`), contracts (`@@`) + bounds + default methods, `dyn`
- `String`, `Array`, `Map` and an early `Collections`/`Text` slice of the stdlib
- attributes incl. `[const]`, `[public]`, capability attrs (`[noalloc]`, …)
- `comptime` (value-level) + value generics

**Full** (the v0.4 spec)
- concurrency + I/O tiers, fibers, channels, `Sendable`/`Shareable`
- macros (declarative + proc), operator overloading `@(+)`, FFI/`extern`, inline `asm`
- the full stdlib + low-level layer
- the extended toolchain features that need language support (telemetry attrs, etc.)

---

## 3. Phased milestones (with exit criteria)

Each milestone is "done" only when its **exit test** passes. The compiler accrues its own test
suite the whole way (§7).

### Phase A — Front-end skeleton (Tiny)
- **A1 Lexer + parser** for Tiny, producing AST. *Exit:* round-trips a corpus of Tiny files to AST;
  `jolt fmt` (parse→print) is idempotent.
- **A2 Query engine + resolver.** Names/scopes resolve; the query graph records dependencies.
  *Exit:* re-resolving after an edit only recomputes affected items (incrementality smoke test).
- **A3 Type checker (Tiny).** Every expression typed; no implicit conversions; errors with spans.
  *Exit:* a type-error corpus produces the expected diagnostics.

### Phase B — The Custodian (Tiny) ← de-risk here
- **B1 Move/ownership analysis:** use-after-move is a compile error on Tiny.
- **B2 Borrows:** `borrow`/`claim`, shared-XOR-mutable, non-lexical liveness.
- **B3 Inference:** no lifetime annotations needed for Tiny cases.
- *Exit:* a curated "should-reject / should-accept" suite passes; **and** a written evaluation of
  whether "easier than Rust" holds on real Tiny examples. **This milestone validates the language's
  central claim before anything is built on top of it.** If the model is wrong, this is where we
  learn it cheaply.

### Phase C — Execution (Tiny)
- **C1 MIR + lowering** for Tiny.
- **C2 MIR interpreter** — Tiny programs *run*. *Exit:* a Tiny program suite produces correct output.
- **C3 `jolt run --interpret`** wired to the CLI. *Exit:* end-to-end `hello world` and small programs
  run from source.
- **Now there is a full, validated pipeline (parse→type→Custody→run) on a small language.**

### Phase D — Core language
- **D1** structs/enums/unions + methods + pattern matching (extend types, Custodian, interpreter).
- **D2** Option/Result/error/`?`/`??`/defer/errdefer.
- **D3** generics + contracts + `dyn` + monomorphization; capability checking.
- **D4** comptime engine generalized (reuse the interpreter) + value generics.
- *Exit per step:* the growing language suite + Custodian suite stay green; Core programs run.

### Phase E — Native codegen
- **E1 LLVM backend** for Core: emit object code, link, run natively. *Exit:* the Core program suite
  produces identical results compiled vs interpreted (differential test).
- **E2 Incremental codegen + linking**, per-function objects.
- **E3 Optimization levels**, debug info (DWARF) for the debugger.

### Phase F — Caching & incremental build (productionize)
- **F1 Content-addressed store**; persist the query graph across runs.
- **F2 Early-cutoff validation:** editing a comment recompiles nothing; editing a body recompiles
  ~that function. *Exit:* the §11 "what recompiles" table from `jolt-caching-system.md` is met.
- **F3 Shared/remote cache** (later; after local is solid).

### Phase G — Full language
- Concurrency + I/O tiers + fibers + channels + `Sendable`/`Shareable`; macros; `@(+)`; FFI; asm;
  full stdlib + low-level layer.
- *Exit:* the v0.4 spec's worked examples (incl. Appendix A) compile, run, and pass.

### Phase H — Self-hosting (Stage 1)
- Port `libjolt` to Jolt, module by module, compiled by the Rust stage-0 compiler.
- *Exit:* the Jolt-written compiler passes the **same** test suite as stage-0, and **stage-2 ==
  stage-3** (the Jolt compiler compiling itself produces a byte-identical compiler — the classic
  three-stage bootstrap check). Rust stage-0 is retired (kept archived) only after this holds.

---

## 4. Toolchain build order (compiler-first, two pulled early)

```
with the parser (Phase A):     jolt fmt            (parse → canonical print; trivial once AST exists)
with execution (Phase C):      jolt test runner    (cheap, and it accelerates ALL later work)
                               jolt run / build / check
after Core (Phase D):          jolt doc            (needs decls + /// comments)
                               jolt repl           (reuse the interpreter)
                               jolt lint           (reuse type/capability passes)
after codegen (Phase E):       jolt debug (DWARF), jolt asm/ir
after caching (Phase F):       jolt build --timings, incremental everything
with package mgr:              jolt add/update/fetch, jolt.toml resolver, jolt.lock, jolt audit
extended (post-Full):          jolt profile, coverage, verify, bindgen, tree/why/sbom,
                               fix/migrate, graph/bloat, LSP depth, telemetry
LSP:                           jolt lsp grows continuously — it's just libjolt's query API exposed;
                               every front-end milestone immediately improves the editor experience.
```

**Why `fmt` and `test` early:** `fmt` is almost free once the parser exists and keeps the codebase
consistent from the start; the **test runner pays for itself immediately** because the compiler's own
test suite (§7) runs on it. Everything else is genuinely easier after the compiler works, so it
follows.

---

## 5. Dependency graph (what unblocks what)

```
lexer → parser → AST ──┬→ resolver ──→ type checker ──→ CUSTODIAN ──→ MIR ──→ interpreter ──→ run
                       │                                  │            │           │
                       └→ fmt                             │            │           └→ comptime → REPL
                                                          │            └→ LLVM → native → debugger
query engine ──(underlies all of the above)              └→ capability check
                       │
                       └→ cache ──→ incremental build ──→ remote cache
package manager ──→ build system (build.jolt) ──→ targets/cross-compile
testing system ──(rides on the runner)──→ DST, fuzz, mutation, coverage
security/permissions ──(enforced in: capability pass, runtime, build sandbox)
```

Critical path to "a usable language": **lexer → parser → resolver → types → Custodian → MIR →
interpreter**. Everything else hangs off that spine.

---

## 6. Self-hosting strategy (detail)

1. Keep stage-0 (Rust) as the **trusted bootstrap** throughout Phase H.
2. Port `libjolt` modules to Jolt in dependency order (lexer first, backend last), compiling each with
   stage-0 and testing against the same suite.
3. **Three-stage check:** stage-0(Rust) compiles stage-1(Jolt source) → `compiler_A`; `compiler_A`
   compiles the same Jolt source → `compiler_B`; `compiler_B` compiles it again → `compiler_C`.
   Require `compiler_B` and `compiler_C` byte-identical (reproducible builds make this checkable).
4. Only then is self-hosting declared; stage-0 is archived (needed again only for a from-scratch
   rebuild on a new platform).
5. The Custodian and comptime engine are the trickiest ports (they encode the language's hardest
   semantics) — port them with the most test coverage.

---

## 7. Testing the compiler itself

The compiler is the first and most important Jolt "user," and it's tested with the system from
`jolt-testing.md`:
- **Unit + integration suites** per pass (lexer/parser/types/Custodian/codegen), grown every phase.
- **Differential testing:** interpreter output == native output for the whole program corpus (catches
  codegen bugs).
- **Snapshot tests** for diagnostics (error message quality is a feature — lock it down).
- **Fuzzing** the parser/type checker (random + structured input must never crash, only diagnose).
- **Property tests:** e.g. `fmt(parse(x))` idempotent; `parse(fmt(ast)) == ast`.
- **Custodian conformance suite:** an ever-growing should-accept / should-reject corpus — the
  language's safety guarantee, pinned.
- **DST on the compiler's own concurrency** once it parallelizes (the cache + parallel codegen are
  concurrent — simulate them deterministically).
- **Three-stage bootstrap equality** as the ultimate self-host correctness test (§6).

---

## 8. Risks & mitigations

| Risk | Mitigation |
| ---- | ---------- |
| **The Custodian is too hard / "easier than Rust" doesn't hold** | Built in Phase B on Tiny, *before* anything depends on it — fail cheap, redesign if needed. |
| LLVM integration drag | Interpreter-first means the front end is fully usable before LLVM; LLVM is additive, not blocking. |
| Query architecture retrofit | Adopted day one; not retrofittable, so it's non-negotiable up front. |
| Scope explosion | Tiny→Core→Full gates; each subset must be complete & green before growing. |
| Self-host doubling work | Deferred (not early); only begun once the language is stable, with stage-0 as a safety net. |
| Diagnostic quality erosion | Snapshot-tested from Phase A; treated as a feature, not an afterthought. |

---

## 9. Suggested sequencing (phase dependencies, not calendar)

```
A (front end) → B (Custodian) → C (interpreter/run)        ← MVP: a safe language that runs
     → D (Core) → E (LLVM/native) → F (caching)            ← a real, fast, incremental compiler
          → G (Full language + stdlib) 
               → package mgr + build system + extended toolchain
                    → H (self-host)                         ← credibility milestone
```

Two natural "releasable" points: **after C** (interpreted Tiny — proves the design, esp. the
Custodian) and **after F** (native, incremental Core — the first genuinely useful compiler). Full +
self-host follow.

---

## 10. Open implementation questions (refinements, not forks)

These don't block starting; resolve as you reach them.
1. **LLVM binding approach** — use a Rust LLVM crate (e.g. inkwell-style safe wrapper) vs. raw FFI to
   the C API. Affects E1 speed vs control.
2. **Cache hash** — BLAKE3 recommended; confirm and define key normalization (path-independence).
3. **Diagnostic catalog ownership** — assign stable `E####` codes as passes are built, or batch later.
4. **Stdlib bootstrapping** — how much stdlib must exist in Rust intrinsics vs. written in Jolt (the
   `[nostd]` core suggests a minimal intrinsic surface, most of the library in Jolt).
5. **MIR stability** — is MIR a stable interface (so tools/backends target it) or internal-only? Lean
   internal-only until post-Full.
6. **Parallelism granularity** — per-function query parallelism from the start, or serial until Core?
   (Lean: serial first, parallelize once correct — easier to debug.)
```
