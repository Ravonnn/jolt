# Year 2 — Core Language, Native Code, and Incremental Builds

> Phases 4–6. Outcome: **Jolt 0.3 (alpha)** — a real (if minimal) language, compiled to native code,
> with an aggressive incremental cache. The first genuinely useful compiler.

---

## Phase 4 — Core Language

**Objective.** Grow Tiny into Core: data types, abstraction, error handling, and compile-time code —
each feature extended consistently through types, the Custodian, and the interpreter.

**Implements (docs).** `jolt-spec-v0.4.md` §structs/enums/unions, §pattern-matching, §generics &
contracts, §errors (`!T`/`Result`/`?`/`??`/`defer`), §comptime; `jolt-grammar.md` §7 (patterns), §8
(declarations), §10 (types incl. value generics); `jolt-safety-attributes.md` (capability catalog for
the capability pass); `jolt-compiletime-safety.md` §2 (generics/config); `jolt-tour.md` §5–§8.

**Workstreams**
1. **Structs / enums / unions**, methods (`Type::method`, `self`/`$$self`).
2. **Pattern matching:** `match` with literals, enums, tuples/structs, ranges, or-patterns, guards,
   `_`, capture (`:=`); exhaustiveness checking.
3. **Option/Result/errors:** `Some`/`Nothing`, `Ok`/`Err`, `!T`/`Result`, `?`/`??`, `error`,
   `defer`/`errdefer`.
4. **Generics + contracts:** `|T|`, `@@` contracts, bounds, default methods, **monomorphization**,
   and `dyn` (heap-boxed) for dynamic dispatch.
5. **Capability checking** (`[noalloc]`/`[noio]`/…) — transitive call-graph analysis.
6. **comptime + value generics:** generalize the interpreter for compile-time eval; `Array<T, N>`;
   comptime guards resolved at monomorphization.
7. Extend the Custodian to all new constructs (moves through fields, borrows of struct fields, `dyn`
   ownership, `Dispose`).

**Deliverables.** A Core language that type-checks, borrow-checks, and runs (interpreted).

**Definition of Done**
```
[ ] Structs/enums/unions + methods work; field-level mutability rules enforced
[ ] match is exhaustive-checked; full pattern grammar parses and lowers
[ ] Option/Result + ?/?? work; ? on Option (or ?? on Result) is a compile error
[ ] Generics monomorphize; contract bounds verified at instantiation; dyn dispatch runs
[ ] Capability attributes enforced transitively (tests/capability green)
[ ] comptime evaluates; value generics + comptime guards resolve at compile time
[ ] Custodian conformance suite extended to all Core constructs and green
[ ] Tutorial §4–§8 examples run
```

**Verification Gate.** Core program corpus runs correctly (interpreted); the extended custody +
capability suites are green; exhaustiveness and bound-checking reject the expected negative cases;
`Dispose` runs deterministically (incl. on error paths) in tests. Sign-off: language lead.

---

## Phase 5 — Native Codegen (LLVM)

**Objective.** Compile Core to native machine code, matching interpreter behavior exactly.

**Implements (docs).** `jolt-toolchain.md` §1 (pipeline → backend), §1.1 (LLVM backend), §1.2 (opt
levels), §debug; `jolt-implementation-plan.md` §1 (`jolt-backend-llvm`), §3 Phase E, §10 Q1 (LLVM
binding approach); `jolt-toolchain.md` §11 (targets/cross-compilation).

**Workstreams**
1. **LLVM backend** (`jolt-backend-llvm`): lower MIR → LLVM IR → object code → link.
2. **`jolt build` / `jolt run`** (native path) and **`jolt asm`/`jolt ir`** dumps.
3. **Incremental codegen:** per-function objects; incremental link.
4. **Optimization levels** (`-O0..-O3`, `-Osize`) and **debug info (DWARF)** for the debugger.
5. **Differential testing:** native output must equal interpreter output across the whole corpus.
6. Begin **cross-compilation** plumbing (target triples; first alt-target builds).

**Deliverables.** Native binaries from Core programs; identical results to the interpreter.

**Definition of Done**
```
[ ] Core programs compile to native binaries and run with correct output
[ ] Differential test: native == interpreter on the entire run corpus
[ ] Optimization levels produce correct code at each -O
[ ] DWARF emitted; a debugger can set a breakpoint and inspect a variable
[ ] At least one cross-target (e.g. wasm32 or aarch64) builds and runs a sample
[ ] asm/ir dumps available per function
```

**Verification Gate.** 100% of the run corpus matches between interpreter and native at `-O0` and
`-O2`; cross-target sample runs (in an emulator if needed); a benchmark baseline is recorded for
Phase 6 to improve against. Sign-off: backend owner + language lead.

---

## Phase 6 — Incremental Caching & Build Performance

**Objective.** Make rebuilds proportional to *what changed*, and builds reproducible.

**Implements (docs).** `jolt-caching-system.md` (entire — esp. §2 query model, §3 what's cached, §4
CAS, §9 correctness/reproducibility, §10 CLI, §11 the "what recompiles" table that the gate checks);
`jolt-implementation-plan.md` §3 Phase F, §10 Q2 (hash function).

**Workstreams**
1. **Content-addressed store** (`jolt-cache`): hash inputs + compiler version + target + flags.
2. **Persist the query graph** across runs; red/green invalidation with **early cutoff**.
3. **Incremental everything:** parse/type/custody/capability/codegen all cache per item.
4. **Reproducible builds:** deterministic codegen; verify by recompiling and comparing hashes.
5. **CLI:** `--no-cache`, `--verify-cache`, `jolt cache size/gc/clear`, `--timings`.
6. **Benchmark suite** for incremental scenarios (edit comment / body / signature / dep bump).

**Deliverables.** A fast, incremental, reproducible compiler. **Tag: Jolt 0.3 (alpha).**

**Definition of Done**
```
[ ] Editing a comment / reformatting recompiles nothing downstream (early cutoff verified)
[ ] Editing a function body recompiles ~that function; signature change recompiles direct callers
[ ] The "what recompiles" table (jolt-caching-system.md §11) is met by measured tests
[ ] Builds are reproducible: --verify-cache passes (recompiled == cached, byte-identical)
[ ] Cache GC + size reporting work; cache is purely a perf layer (deletion only costs time)
[ ] Incremental benchmark suite green against thresholds
```

**Verification Gate.** The incremental benchmark suite meets its thresholds (e.g. one-function edit
rebuild within target time); `--verify-cache` is byte-identical on the corpus; deleting the cache and
rebuilding yields identical artifacts. Alpha tagged. Sign-off: build/perf owner + language lead.

---

### Year-2 exit state
A native, optimizing, incrementally-cached compiler for a real (Core) language with generics,
contracts, errors, and compile-time code — fast enough and useful enough to write nontrivial programs.
