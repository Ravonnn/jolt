# Jolt — Testing System & Built-in `Test` Library

> A first-class, extensive testing capability built into the language and toolchain: macro-based
> assertions, fixtures, property-based testing with shrinking, fuzzing, sanitizers, snapshot &
> mutation testing, statistical benchmarking, contract-based mocking, **deterministic simulation
> testing**, and a cache-aware parallel runner. Tests are ordinary Jolt, subject to the same
> capabilities/permissions/Custodian as production code.
>
> Decisions baked in: assertions are **macros** (capture expression source for great messages); soft
> (`expect`) assertions are **included**; mocking supports both **hand-written** contract mocks and
> **`#derive(Mock)`**; scope includes **snapshot and mutation** testing; **deterministic simulation**
> is a first-class mode.

---

## 1. Declaring tests

```jolt
[test]
@adds_positives() None -> assert_eq!(add(2, 3), 5); ;;

[test, should_fail: "divide by zero"]
@rejects_zero() None -> divide(1, 0); ;;

[test, skip: "flaky on CI", tags: ["net"]]
@fetches() None -> ... ;;

[test, only]                       // focus: run only [only] tests during dev
@wip() None -> ... ;;

[bench]
@hash_throughput(b: $$Bencher) None -> b.iter(|| -> hash(data) ;;); ;;

[fuzz]
@parser_never_crashes(input: Slice<Byte>) None -> parse(input); ;;
```

Attributes: `[test]`, `[bench]`, `[fuzz]`, plus modifiers `should_fail[: "msg"]`, `skip: "reason"`,
`only`, `tags: [...]`, `cases: [...]` (table-driven), `setup`/`teardown` hooks.

**Three scopes:** unit (same module — sees privates), integration (`tests/` dir — public API only),
doc (runnable `///` examples). All run under `jolt test`.

---

## 2. Assertions (macro-based)

Macros (`#`-prefixed per the macro system) capture the **source expression and operand values**, so
failures explain themselves:

```jolt
assert!(x > 0);
// FAIL: assert!(x > 0)  —  x = -3

assert_eq!(result, expected);
// FAIL: assert_eq!(result, expected)
//   left:  Config { port: 80,  tls: false }
//   right: Config { port: 443, tls: true  }   (structured diff via Debug)
```

Family: `assert!`, `assert_eq!`, `assert_ne!`, `assert_near!(a, b, tol)`, `assert_throws!(expr, Err)`,
`assert_matches!(value, pattern)` (uses the pattern grammar), `assert_snapshot!(value)` (§7).

**Soft assertions** (`expect_*`) record failures and keep going, so one run surfaces every problem;
the test fails at the end if any `expect` failed:

```jolt
[test]
@validates_all() None ->
    expect_eq!(user.name, "Aoi");
    expect!(user.age >= 0);          // both checked even if the first fails
    expect_matches!(user.role, Admin | Editor);
;;
```

---

## 3. Fixtures & lifecycle

```jolt
[setup]    @before(ctx: $$TestCtx) None -> ctx.db = open_temp_db(); ;;
[teardown] @after(ctx: $$TestCtx)  None -> ... ;;     // Dispose runs even on failure/panic

[test]
@uses_db(ctx: $$TestCtx) None ->
    ctx.db.insert("k", 1);
    assert_eq!(ctx.db.get("k"), Some(1));
;;
```

- `setup`/`teardown` at per-test or per-module scope; cleanup leans on the `Dispose` contract so
  resources release deterministically.
- **`TestCtx`** is a hermetic context handed to each test: a temp dir, a captured logger, a seeded
  RNG, and the simulation handle (§9) — so tests don't touch real global state.
- **Table-driven:** `[test, cases: [(2,3,5), (0,0,0), (-1,1,0)]]` runs the body once per tuple, each
  reported as a separate result.

---

## 4. Property-based testing

```jolt
[test]
@reverse_twice_is_identity() None ->
    for_all(vec_of(any<Int>()), |xs| ->
        assert_eq!(reverse(reverse(xs)), xs);
    ;;);
;;
```

- `for_all(generator, property)` with automatic **shrinking** to a minimal failing input.
- `Generator` contract + combinators: `int_range(a,b)`, `any<T>()`, `vec_of(g)`, `one_of([...])`,
  `map`, `filter`, `tuple`, `weighted`.
- Types derive a default generator: `#derive(Arbitrary)`.
- **Deterministic:** each run prints its seed; `--seed N` reproduces exactly. A discovered
  counterexample is auto-saved as a regression case.

---

## 5. Fuzzing

```jolt
[fuzz]
@decode_roundtrip(input: Slice<Byte>) None ->
    if Some(v) := try_decode(input) -> assert_eq!(encode(v), input); ;;
;;
```

- `jolt fuzz <target>` — coverage-guided engine, persistent corpus, minimization.
- Integrated with sanitizers (§6); a crash is **auto-converted into a regression `[test]`** with the
  triggering input embedded.
- Structured fuzzing: a `[fuzz]` target can take a typed input (`#derive(Arbitrary)`) instead of raw
  bytes, fuzzing at the value level.

---

## 6. Sanitizers & dynamic checks (test mode)

Complement the compile-time guarantees for code the Custodian can't fully see (i.e. `[unsafe]`/FFI):

- **AddressSanitizer / UBSanitizer** — out-of-bounds, use-after-free, undefined behavior in `unsafe`.
- **ThreadSanitizer** — data races (a backstop to the Custodian's `Sendable`/`Shareable` static
  checks, catching races that only arise through `unsafe`/FFI).
- **Leak detection** — flags resources not released by `Dispose`/allocator at test end.
- Enabled with `jolt test --sanitize=address,thread,...`; on by default in `jolt fuzz`.

---

## 7. Snapshot testing

```jolt
[test]
@renders_invoice() None ->
    assert_snapshot!(render(invoice));   // first run records; later runs diff
;;
```
- First run writes the expected output to a snapshot file; subsequent runs diff against it.
- `jolt test --update-snapshots` accepts changes (reviewed in version control).
- Great for serializers, formatters, codegen, and any large structured output.

---

## 8. Mutation testing

Measures whether the suite actually *catches* bugs, not just whether it passes.

- `jolt test --mutate` systematically perturbs the program (flip comparisons, swap operators, drop
  statements, alter constants) and reruns the affected tests.
- A mutant that **survives** (tests still pass) reveals an untested behavior → reported as a gap with
  the surviving mutation and location.
- Scoped by the incremental cache + coverage data so only tests reaching a mutation rerun (otherwise
  mutation testing is prohibitively slow) — Jolt's per-function caching makes this tractable.
- Produces a **mutation score** (% killed) as a suite-quality metric for CI gating.

---

## 9. Deterministic Simulation Testing (DST)

The flagship capability — run an **entire concurrent/distributed system on a virtual clock with a
controlled scheduler and injected faults**, fully deterministic and seed-reproducible (the
FoundationDB / TigerBeetle approach). Jolt is unusually suited to this: the **fiber runtime** and
**completion-based `Io`** can be backed by a *simulated* executor, so the same application code runs
under a deterministic simulator without modification.

### 9.1 How it works
- A `[simulation]` test runs the system inside a **`Sim`** runtime that replaces the real fiber
  scheduler, clock, RNG, network, and filesystem with deterministic, seed-driven models.
- **Virtual time:** the clock only advances when the simulator decides; `sleep`/timeouts are instant
  in wall-clock but ordered correctly in virtual time. A multi-hour scenario runs in milliseconds.
- **Controlled scheduling:** the simulator chooses task interleavings from the seed, exploring orderings
  a real scheduler might hit only rarely — surfacing race conditions reliably.
- **Fault injection:** the simulator can drop/delay/reorder/duplicate messages, crash and restart
  nodes, partition the network, fail disk writes, and clock-skew — all seed-driven.
- **Total determinism:** because *every* nondeterminism source (time, scheduling, RNG, I/O, faults)
  is funneled through the seeded `Sim`, a run is perfectly reproducible. Found a 1-in-a-billion bug?
  The seed replays it every time.

### 9.2 What makes it sound in Jolt
- **Capabilities pin the nondeterminism sources.** A `[simulation]` test runs the system under code
  that is `[noio]`-at-the-real-boundary — all I/O goes through `Io`/`Net`/`Fs`/`Time`, which the
  simulator substitutes. Anything reaching for real time/RNG/syscalls outside the injected interfaces
  is a capability violation, so the simulation can't be silently escaped.
- **Fibers are already an abstraction over the scheduler**, so swapping in a deterministic scheduler
  needs no application changes.
- **`comptime`/pure code is deterministic by construction** (enforced by the sandbox), so it never
  introduces hidden nondeterminism.

### 9.3 Surface
```jolt
[simulation, seed_range: 0..10000]      // runs many seeds; reports any that fail
@cluster_stays_consistent(sim: $$Sim) None ->
    $nodes = sim.spawn_nodes(5);
    sim.run_clients(100, |c| -> c.write(random_key(), random_val()) ;;);

    // inject faults during the run
    sim.partition([nodes[0], nodes[1]], duration: secs(30));
    sim.crash(nodes[2]);  sim.restart_after(nodes[2], secs(5));
    sim.delay_messages(min: ms(10), max: ms(500));

    sim.advance_until(idle());                 // run virtual time to quiescence
    assert!(all_nodes_agree(nodes));           // invariant must hold under all faults
;;
```
- `sim` exposes: `spawn_nodes`, `crash`/`restart`, `partition`/`heal`, `delay_messages`/`drop`/`reorder`,
  `clock_skew`, `advance`/`advance_until`, and **invariant hooks** (`sim.check_always(invariant)`)
  evaluated at every scheduling point.
- `seed_range` runs thousands of seeds (in parallel, cache-aware); each failing seed is reported with
  a one-command replay (`jolt test --seed N --trace`) that prints the exact event timeline.
- **Time-travel trace:** a failing run emits a full deterministic event log the debugger can step
  through forward and backward.

### 9.4 Why it matters
Concurrency and distributed-systems bugs are the hardest to reproduce and the most dangerous. DST
turns "happens once a month in production" into "fails every time on seed 8471," and lets CI explore
millions of fault interleavings deterministically. Pairing it with Jolt's static race-freedom
(`Sendable`/`Shareable`) means: the type system rules out *data* races statically, and DST hunts
*logic*/*ordering*/*fault-tolerance* races dynamically — together covering both halves.

---

## 10. Mocking / test doubles

Contract-based, no runtime magic:
- A test supplies a type that **conforms to the same contract** the code depends on (static dispatch).
- `#derive(Mock)` on a contract generates a recording stub: set expectations, capture calls, assert
  interactions.

```jolt
#derive(Mock)
@@Clock -> @now(self) Instant; ;;

[test]
@expires_tokens() None ->
    $clock = MockClock::new();
    clock.expect_now().returns(t0);
    $svc = Service::new(clock);
    ...
    clock.verify();                  // asserts the expected calls happened
;;
```

---

## 11. Benchmarking

```jolt
[bench]
@sort_1k(b: $$Bencher) None -> b.iter(|| -> sort(copy(data)) ;;); ;;
```
- Statistical: warmup, multiple samples, outlier detection, variance reporting.
- **Regression tracking:** compares against a stored baseline; CI fails on a significant slowdown.
- `--save-baseline` / `--compare-baseline`; integrates with PGO data.

---

## 12. The runner (`jolt test`)

- **Parallel & isolated** by default; `--jobs N`, `--serial`.
- **Filtering:** name/regex, `--tag net`, `--exclude slow`, `[only]`.
- **Cache-aware:** tests whose transitive inputs are unchanged show **"cached pass"** and are skipped;
  `--force` reruns. (Same query/CAS machinery as the build cache.)
- **Capability/permission-aware:** each test runs under the security model; a `[test]` gets only the
  permissions it declares, so a unit test can't accidentally hit the network or filesystem.
- **Output:** human (progress, colored structured diffs), `--format json`, JUnit XML for CI,
  `--coverage` (line + branch), `--watch` (rerun only affected tests on save via incremental deps).
- **`[nostd]`/embedded:** a no-alloc harness runs tests on-device or in QEMU.

---

## 13. The `Test` stdlib module (surface)

```
Test
  assert!  assert_eq!  assert_ne!  assert_near!  assert_throws!  assert_matches!  assert_snapshot!
  expect!  expect_eq!  expect_matches!            (soft assertions)
  Bencher                                          (b.iter, b.bytes, ...)
  Generator  Arbitrary  for_all  shrink           (property testing)
    int_range vec_of any one_of map filter tuple weighted
  Mock  #derive(Mock)  Expectation                (mocking)
  TestCtx                                          (temp dir, logger, seeded RNG, sim handle)
  Sim                                              (deterministic simulation, §9)
    spawn_nodes crash restart partition heal delay_messages drop reorder clock_skew
    advance advance_until check_always
```
Attributes (compiler-recognized): `[test]`, `[bench]`, `[fuzz]`, `[simulation]`, `[setup]`,
`[teardown]`, with modifiers `should_fail`, `skip`, `only`, `tags`, `cases`, `seed_range`.

---

## 14. How it integrates with the rest of Jolt

- **Doctests** run inside `jolt test` — examples never rot.
- **Capabilities in tests:** `[noalloc]`/`[noio]` tests *prove* allocation-freedom / purity of the
  code under test.
- **Incremental cache** makes mutation testing and thousand-seed simulation tractable (only affected
  tests/seeds rerun).
- **Static race-freedom + DST** = data races ruled out at compile time, ordering/fault bugs hunted at
  test time.
- **Security model** sandboxes tests by declared permission, so the suite is itself least-privilege.

---

## 15. Open questions

1. **DST scheduler strategy** — random interleaving from seed, or also exhaustive/bounded model
   checking for small scenarios? (Lean: random-by-seed default, optional bounded exhaustive mode.)
2. **Mutation operators set** — which mutations ship by default, and is the operator set extensible
   via comptime?
3. **Snapshot storage** — inline in the test file, or sidecar files under `snapshots/`? (Lean:
   sidecar, reviewable in VCS.)
4. **Fuzz/property corpus sharing** — commit corpora to the repo, store in the shared cache, or both?
5. **Sim fault-model surface** — how much fault vocabulary is built-in vs. user-defined fault
   injectors implementing a `Fault` contract.
