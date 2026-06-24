# Jolt — Caching System

> An aggressive, incremental cache that makes rebuilds proportional to *what changed*, not to project
> size. It spans the whole pipeline (parse → type → Custodian → capability → monomorphize → codegen →
> link), is content-addressed and shareable across projects and machines, and is safe by
> construction (verified, sandboxed, reproducible). Built on the query-based compiler architecture
> (`jolt-toolchain.md` §1) — incrementality is not bolted on, it's how the compiler is shaped.

---

## 1. Goals

1. **Edit one function → recompile ~that function.** Rebuild cost tracks the *blast radius* of a
   change, not the codebase size.
2. **Never recompute a known answer.** Every stage's output is cached and reused when its inputs are
   unchanged — across runs, projects, and (optionally) machines.
3. **Correct before fast.** A cache hit must be *provably* equivalent to recompilation; staleness
   bugs are unacceptable. Reproducible builds make this checkable.
4. **Shared & distributed.** A team/CI shares one cache; you rarely compile what a colleague already
   did.
5. **Safe.** Cache entries are verified and the things that produce them (build scripts, proc-macros)
   run sandboxed (ties to the security model).

---

## 2. Architecture: query-based incremental compilation

The compiler is a graph of **pure queries** (demand-driven, memoized), e.g.
`type_of(fn)`, `custody_check(fn)`, `mir_of(fn)`, `object_for(module)`. Each query:
- is keyed by the **content hash** of its inputs (not timestamps),
- records the **other queries it read** (its dependency edges),
- caches its result.

On rebuild, the compiler **re-runs a query only if a transitive input's hash changed**. This is the
red/green (dirty/validated) model:

```
edit file → rehash changed items → mark dependent queries "maybe-dirty"
  → for each, recompute; if the new result hash == old → mark "green" (stop propagation)
  → only genuinely changed results propagate further
```

The crucial property — **early cutoff**: if you edit a comment or reformat code, the *parsed AST*
changes but `type_of` produces the same hash, so nothing downstream reruns. Cost reflects semantic
change, not textual change.

---

## 3. What is cached (every stage)

| Stage | Cache key (hash of…) | Cached value |
| ----- | -------------------- | ------------ |
| Lex/parse | file bytes | token stream / AST per item |
| Macro/comptime expand | AST + macro defs + comptime inputs | expanded AST |
| Name resolution | item + visible imports | resolved symbols |
| Type check | typed inputs + signatures it reads | typed HIR per fn |
| **Custodian** | typed fn + borrowed signatures | ownership/borrow proof |
| Capability check | fn + callee capability annotations | capability result |
| Monomorphization | generic fn + concrete type args (+ comptime guard values) | concrete instance |
| Lowering | typed/mono fn | MIR per fn |
| **Codegen** | MIR + target + opt level | object code per fn/module |
| Link | object hashes + link flags | final artifact |

Granularity is **per-item (function/type)**, not per-file — editing one function in a 5,000-line file
recompiles that function, not the file.

---

## 4. The content-addressed store (CAS)

All cached values live in a **content-addressed store**, keyed by the hash of (inputs + compiler
version + flags):

```
~/.jolt/cache/
  cas/<hash>            # immutable cached values (objects, MIR, metadata)
  index/                # query-graph metadata (dep edges, hashes) per build
```

- **Keys include everything that affects the output:** input hashes, target triple, opt level,
  feature flags, compiler version, and relevant capability/permission policy. Change any → new key →
  no false reuse.
- **Immutable & deduplicated:** identical results (common across projects depending on the same lib)
  are stored once. Two projects using `json@0.4` for the same target share its compiled objects.
- **Garbage-collected:** `jolt cache gc` (LRU + reachable-from-recent-builds); `jolt cache size`,
  `jolt cache clear`.

---

## 5. Aggressive reuse layers

1. **In-process** — within a single `jolt` invocation, queries are memoized in memory.
2. **On-disk local** — the CAS persists across invocations on one machine; the dominant everyday win.
3. **Dependency cache** — compiled dependencies (from the registry) are cached as objects, not
   recompiled per project; prebuilt stdlib flavors (`full`/`nostd+alloc`/`nostd+noalloc` per target)
   ship as cache entries.
4. **Shared/remote cache** — an optional team/CI cache server (or object-store bucket). A cold
   machine downloads cache hits instead of compiling. CI populates it; developers consume it.
5. **Distributed compilation (future)** — farm out cache-miss codegen queries to a build cluster
   (each query is pure + content-addressed, so this is natural).

---

## 6. Incremental linking & codegen

- **Per-function objects + incremental link:** codegen emits objects at function/module granularity;
  the linker reuses unchanged objects and relinks only what moved (incremental/ThinLTO-style).
- **Debug fast path:** the fast in-house backend (toolchain §1.1) caches at fine granularity for
  near-instant edit-run loops; LLVM/release uses ThinLTO so cross-module inlining doesn't force whole-
  program recompiles.
- **Codegen units** are sized to balance cache granularity vs optimization scope (smaller = better
  reuse, larger = better inlining); tunable per profile.

---

## 7. Interaction with language features

- **Generics/monomorphization:** each concrete instantiation is a cache entry keyed by `(generic fn,
  type args, comptime values)`. `Vec<Int>` compiled once is reused everywhere, cross-project.
- **`comptime`:** comptime evaluation is a cached query keyed by its inputs — a comptime table or
  generated source isn't recomputed unless its inputs change. **Requires comptime to be
  deterministic** (no ambient I/O) → which the security sandbox already enforces (§9).
- **The Custodian:** borrow/ownership proofs are cached per function and invalidated only when the
  function or a *signature it borrows against* changes — so safety checking is incremental too, not a
  whole-program repass.
- **Capability checks:** cached per function; a callee's capability change invalidates only its
  callers (a localized call-graph reproof).
- **Macros/`[attr]` proc-macros:** their *expansion* is cached keyed by input AST + macro version; a
  proc-macro must be deterministic (sandboxed) for its expansion to be cacheable.

---

## 8. Build-system & test integration

- **`build.jolt` steps are cache nodes:** a `generate`/codegen step reruns only if its declared inputs
  changed; outputs are CAS entries other steps consume. The build graph *is* a query graph.
- **`jolt test` caches results:** a test whose transitive inputs are unchanged is **skipped** (shown
  as "cached pass"); only tests touching changed code rerun. Same for doctests and benches (with a
  flag to force).
- **Cross-target:** the cache is keyed by target, so switching between native and `wasm32`/embedded
  builds doesn't thrash — each target keeps its own warm entries.

---

## 9. Correctness & safety (why aggressive caching is still trustworthy)

- **Hash everything that matters:** compiler version, target, flags, feature set, and permission
  policy are part of every key — no environment-skew reuse.
- **Reproducible builds:** deterministic codegen means a cache hit is byte-identical to a fresh
  compile; CI can *verify* the cache by recompiling a sample and comparing hashes.
- **Sandboxed producers:** anything whose output is cached and could be non-deterministic — `comptime`
  code, proc-macros, `build.jolt` — runs under the security sandbox (`[noio]` by default), so cached
  results can't depend on hidden state. This is the link between caching correctness and the security
  model: **determinism is enforced, not assumed.**
- **Shared-cache integrity:** remote cache entries are **content-verified on fetch** (the hash must
  match) and optionally **signed**, so a poisoned shared cache can't inject bad objects.
- **Self-healing:** a corrupt or mismatched entry is discarded and recomputed; the cache is always a
  *performance* layer, never a *correctness* dependency — deleting it only costs time.

---

## 10. CLI surface

```
jolt build                 # uses the cache automatically (incremental)
jolt build --no-cache      # ignore cache (debug a suspected staleness issue)
jolt build --verify-cache  # recompile and assert hits match (CI integrity check)
jolt cache size            # report cache size & hit stats
jolt cache gc              # garbage-collect unreachable/old entries
jolt cache clear           # wipe local cache
jolt cache push / pull     # sync with the shared/remote cache
jolt build --remote-cache=<url>   # use a team/CI cache
jolt build --timings       # per-query timing + cache hit/miss breakdown
```

---

## 11. Expected behavior (illustrative)

| Change | Recompiled |
| ------ | ---------- |
| reformat / edit a comment | nothing (early cutoff at AST→type) |
| edit a function body (signature unchanged) | that function + its codegen; **no callers** |
| change a function's signature | that function + direct callers (type/Custodian/capability reproof) |
| add a field to a struct | the struct + code that uses the new field |
| bump a dependency version | that dependency + things that use changed APIs (not the whole tree) |
| switch target | first build of that target; subsequent builds warm (per-target cache) |
| pull from CI cache on a clean machine | download hits, compile only local changes |

---

## 12. Open questions

1. **Hash function & key scheme** — pick a fast strong hash (e.g. BLAKE3) and the exact key
   normalization (path-independence so caches are relocatable/shareable).
2. **Cache granularity tuning** — per-function everywhere, or per-module for codegen with per-function
   for front-end stages? Trade reuse vs. metadata overhead.
3. **Remote cache trust model** — who may *write* the shared cache (CI only?), signing requirements,
   and how `--verify-cache` integrates into CI gating.
4. **GC policy** — size cap + LRU, time-based, or reachability from pinned builds; defaults.
5. **comptime determinism enforcement** — is non-deterministic comptime a hard error (can't cache) or
   a cache-disabling warning? (Lean: hard error — determinism is required for the model to hold.)
6. **Debug-info & cache** — ensure cached objects carry correct debug info per build path so the
   debugger experience isn't degraded by reuse.
