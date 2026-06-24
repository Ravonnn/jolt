# Year 1 — Foundations, Front End, the Custodian, and a Running Tiny Language

> Phases 0–3. Outcome: **Jolt 0.1 (preview)** — a safe, Tiny subset of Jolt that runs (interpreted),
> with the Custodian validated. This is the year that proves the design is real.

---

## Phase 0 — Project Foundation & Infrastructure

**Objective.** Stand up the repo, CI, stage-0 scaffold, and engineering practices so every later
phase has solid ground.

**Implements (docs).** `00-repo-structure.md` (entire layout); `jolt-implementation-plan.md` §1
(`libjolt` crate architecture) & §strategy; `jolt-caching-system.md` §2 (query-engine model, for the
skeleton).

**Workstreams**
1. Repo per `00-repo-structure.md`; license, contributing guide, code of conduct, security policy.
2. Rust workspace + pinned toolchain; empty stage crates that compile and are wired into `libjolt`.
3. CI: build + test + `fmt`/`clippy` on every PR; nightly artifact; caching of CI builds.
4. Test harness conventions: how `tests/ui`, `tests/run`, `tests/custody` are written and run.
5. ADR process; import the design corpus into `docs/`.
6. The query engine (`jolt-query`) skeleton — memoization + dependency recording — since everything
   depends on it.

**Deliverables.** A cloneable repo where `cargo build` + `cargo test` pass on empty stages; CI green;
a documented "how to add a test" guide; a working toy query with red/green invalidation.

**Definition of Done**
```
[ ] Repo structure matches 00-repo-structure.md; all stage crates compile
[ ] CI runs build + test + lint on PRs and is green
[ ] Query engine demo: a memoized query recomputes only on changed inputs (unit-tested)
[ ] Test harness documented; a sample test in each of ui/run/custody runs
[ ] Design corpus imported under docs/; ADR template in place
[ ] CONTRIBUTING + coding standards published
```

**Verification Gate.** A fresh checkout on a clean machine builds and tests green via CI; a
deliberately-introduced change to a query input recomputes exactly the dependent query and nothing
else (asserted in a test). Sign-off: build/infra owner.

---

## Phase 1 — Front-End Skeleton (Tiny)

**Objective.** Turn Tiny source into a fully type-checked program representation. (Tiny = functions,
`Int`/`Bool`, operators, `$`/`$$` bindings, `if`/`loop`/`for`, blocks, calls, `println`.)

**Implements (docs).** `jolt-grammar.md` §1 (lexical), §2–§6 & §9 (declarations, bindings, blocks,
expressions — restricted to Tiny); `jolt-spec-v0.4.md` §bindings/§functions/§types (Tiny subset);
`jolt-tour.md` §1–§3 (the surface being parsed); `jolt-implementation-plan.md` §2 (Tiny definition).

**Workstreams**
1. **Lexer** for Tiny tokens; max-munch rules (`>>>`/`>>`/`>`, `;`/`;;`, sigils).
2. **Parser** → AST; the `->`/`;;` block grammar incl. block-as-expression; error-recovery
   productions for the LSP.
3. **Resolver** over the query engine: scopes, name binding, the `$`/`$$`/reassign rules.
4. **Type checker (Tiny):** every expression typed; no implicit conversions; immutable-binding
   enforcement; structured diagnostics with spans.
5. **`jolt fmt`** (pulled early): parse → canonical print.

**Deliverables.** A `jolt check` that accepts valid Tiny and rejects invalid Tiny with good errors;
an idempotent formatter.

**Definition of Done**
```
[ ] Lexer + parser round-trip a Tiny corpus to AST and back (via fmt)
[ ] tests/ui: valid Tiny accepted; invalid Tiny produces expected, snapshot-tested diagnostics
[ ] Reassigning a `$` (immutable) binding is a typed error with a clear message
[ ] No implicit numeric conversions; `-3` infers signed Int
[ ] `jolt fmt` is idempotent (property test: fmt(fmt(x)) == fmt(x))
[ ] Incrementality: editing one function re-checks ~that function (query smoke test)
```

**Verification Gate.** The Tiny UI corpus (≥50 accept + ≥50 reject cases) passes with exact expected
diagnostics; `fmt` idempotence property holds over the corpus; a measured incremental re-check after
a one-function edit touches a bounded set of queries. Sign-off: front-end owner.

---

## Phase 2 — The Custodian (Tiny) — *de-risk the central claim*

**Objective.** Implement ownership, moves, and borrowing on Tiny — and **validate that "safe without
GC, easier than Rust" actually holds** before anything is built on top.

**Implements (docs).** `jolt-spec-v0.4.md` §9 (the Custodian: move/borrow/claim/deref, shared-XOR-
mutable, non-lexical liveness); `jolt-decisions.md` (memory section + rationale);
`jolt-memory-naming-options.md` (vocabulary); `jolt-tour.md` §4 (the intended ergonomics);
`jolt-compiletime-safety.md` §6 (how the Custodian complements type safety).

**Workstreams**
1. **Move analysis:** single-owner tracking; use-after-move is a compile error.
2. **Borrows:** `borrow`/`claim`/`deref`; shared-XOR-mutable rule; non-lexical (last-use) liveness.
3. **Copy types:** `Int`/`Bool`/`Char` copy implicitly via the `Copy` notion; everything else moves.
4. **Lifetime inference** sufficient for Tiny (no annotations needed).
5. **Diagnostics:** custody violations show the move/borrow site, the conflict, and a suggested fix.
6. **Ergonomics evaluation:** write real Tiny programs; document friction vs. Rust; adjust the model
   while it's cheap to change.

**Deliverables.** A Custodian pass; the `tests/custody` conformance suite; a written ergonomics report.

**Definition of Done**
```
[ ] Use-after-move, double-claim, and claim-while-borrowed are all rejected with clear errors
[ ] Multiple shared borrows accepted; shared+mutable simultaneously rejected
[ ] Non-lexical liveness: common patterns compile without manual reordering
[ ] tests/custody: curated should-accept / should-reject suite green
[ ] Custody diagnostics snapshot-tested (message quality locked)
[ ] Ergonomics report written; any model changes recorded as ADRs
```

**Verification Gate.** The custody conformance suite passes; an independent reviewer confirms the
should-reject cases are genuinely unsafe and the should-accept cases are genuinely safe; the
ergonomics report concludes the model is viable (or triggers a documented redesign **now**, not
later). **This is the project's highest-stakes gate.** Sign-off: language lead + an independent
reviewer.

---

## Phase 3 — Execution & MVP (Tiny interpreter)

**Objective.** Make Tiny programs *run*, completing the first end-to-end pipeline
(parse → type → Custody → execute).

**Implements (docs).** `jolt-implementation-plan.md` §1 (MIR + interp backend), §3 Phase C;
`jolt-toolchain.md` §1 (pipeline), §9 (test runner), §1.1 (interpreter backend);
`jolt-toolchain-extended.md` §1 (interpreter mode foundation); `jolt-testing.md` §1–§2 & §12 (the
runner + assertions being stood up).

**Workstreams**
1. **MIR + lowering** for Tiny (desugar `if`/loops/blocks).
2. **MIR interpreter** (`jolt-backend-interp`) — also the basis for comptime and the REPL later.
3. **`jolt run --interpret`** wired into the CLI; a stub `println` + minimal prelude.
4. **`tests/run`** harness: program → expected stdout.
5. **Test runner** (`jolt test`, pulled early): discover `[test]`, run, report; it will run the
   compiler's own future suites.

**Deliverables.** `jolt run` executes Tiny programs; `jolt test` runs `[test]` functions.
**Tag: Jolt 0.1 (preview).**

**Definition of Done**
```
[ ] A suite of Tiny programs runs and produces correct output via the interpreter
[ ] `jolt run --interpret hello.jolt` works end to end
[ ] `jolt test` discovers and runs [test] functions with pass/fail reporting
[ ] The full pipeline (parse→type→custody→run) is exercised by CI on every PR
[ ] Tutorial §1–§3 examples actually run
[ ] Known limitations (no generics/structs/concurrency yet) documented
```

**Verification Gate.** Every program in `tests/run` produces its expected output; `jolt test` passes
its own meta-tests; the §1–3 tutorial snippets execute as shown. The preview is tagged and a short
"what works / what doesn't" note is published. Sign-off: language lead.

---

### Year-1 exit state
A safe Tiny language that compiles, borrow-checks, and runs — the design (especially the Custodian)
is **proven on real code**, on the smallest possible surface, before scaling up.
