# Jolt — Safety & Capability Attribute Catalog

> Attributes the compiler can *enforce* on a function (or block/type), grouping by what they
> restrict. Each is checked **transitively**: a function may not call another that violates a
> capability it has declared. ✅ = already in v0.4. Severity of value: 🔵 core, 🟢 nice-to-have.

---

## 1. Resource-use restrictions (the "no-" family)

| Attribute | ✅ | Guarantees | Why it matters |
| --------- | -- | ---------- | -------------- |
| `[noalloc]` | ✅ | no heap allocation | realtime/embedded hot paths, audio, kernels |
| `[noio]` | ✅ | no I/O (files, net, syscalls) | pure compute, sandboxing, determinism |
| `[nopanic]` | ✅ | no trapping/panicking path; all errors are values | safety-critical code that must never abort |
| `[noblock]` 🔵 | | will not block the thread (no lock waits, no blocking I/O) | realtime + async executors (a blocking call stalls the scheduler) |
| `[nostd]` 🟢 | | no standard library, only core/intrinsics | freestanding/bare-metal targets |
| `[norecurse]` 🟢 | | no recursion (direct or mutual) | bounded stack — embedded, kernel stacks |
| `[bounded_stack: N]` 🟢 | | provably uses ≤ N bytes of stack | hard realtime, interrupt handlers |

`[noblock]` is the most valuable missing one — it's exactly what keeps async/green-thread executors
from deadlocking, and pairs directly with the concurrency models.

---

## 2. Purity & determinism

| Attribute | ✅ | Guarantees |
| --------- | -- | ---------- |
| `[pure]` | ✅ | no side effects, no I/O, no alloc, no mutation of args; same inputs → same output |
| `[const]` (fn) 🔵 | | evaluable at compile time (like `comptime` but as a guarantee/contract on the fn) |
| `[idempotent]` 🟢 | | calling twice == calling once (documented + optionally checked) |
| `[total]` 🟢 | | provably terminates and is defined for all inputs (no infinite loops, no partial functions) — strong, hard to check, great for verification |

`[pure]` is the strongest everyday one; `[const]`-fn is worth adding so purity-for-comptime is a
declarable promise, not just an inference.

---

## 3. Concurrency & memory safety (tie into the Custodian)

| Attribute | ✅ | Guarantees |
| --------- | -- | ---------- |
| `[shared]` / `[shared, sync]` / `[weak]` | ✅ | counted ownership / atomic / cycle-breaking (§9.5) |
| `[unsafe]` | ✅ | unlocks raw pointers etc. (§9.7) |
| `[threadsafe]` 🔵 | | asserts + checks the fn is safe to call from multiple threads concurrently |
| `[atomic]` 🟢 | | a block executes as one uninterruptible unit (compiler/hardware backed where possible) |
| `[main_thread]` 🟢 | | may only run on the main thread (UI toolkits, some OS APIs) |
| `[no_capture]` 🟢 | | a closure parameter may not capture its environment (no hidden references escape) |

---

## 4. Security / information-flow (higher-assurance)

| Attribute | Guarantees | Use |
| --------- | ---------- | --- |
| `[constanttime]` 🔵 | execution time independent of secret inputs (no secret-dependent branches/indexing) | crypto — prevents timing side channels |
| `[zeroize]` 🟢 | the value's memory is wiped on drop | keys, passwords, secrets |
| `[tainted]` / `[untrusted]` 🟢 | marks data from external input; compiler tracks it until `[sanitized]` | injection/validation enforcement |
| `[secret]` 🟢 | value may not be logged, printed, or sent to I/O without explicit declassification | PII, credentials |

`[constanttime]` and `[zeroize]` are standout differentiators — very few languages enforce these,
and a "safe systems language" claiming crypto-friendliness benefits enormously. They also lean on
machinery you already have (`Dispose` powers `[zeroize]`).

---

## 5. API-evolution / lint (not safety, but same mechanism)

| Attribute | ✅ | Meaning |
| --------- | -- | ------- |
| `[deprecated{...}]` | ✅ | warns on use, with message + since-version |
| `[must_use]` 🔵 | | warn if the return value is discarded (great for `Result`/`!T` — stops silently-ignored errors) |
| `[experimental]` 🟢 | | requires an opt-in flag to use; API may change |
| `[stable: "x.y"]` 🟢 | | marks the version an API stabilized in |

`[must_use]` is small but high-value: with errors-as-values, accidentally dropping a `Result` is a
classic bug; `[must_use]` on `Result`/`!T` catches it.

---

## How enforcement should work (one consistent rule)

1. **Transitive.** A `[noalloc]` fn may only call `[noalloc]`-compatible fns. The compiler walks the
   call graph; a violation is a compile error naming the offending call.
2. **Stdlib annotated.** Core/stdlib functions carry these attributes so user guarantees can be
   checked against them (e.g. `alloc()` is *not* `[noalloc]`; `println` is *not* `[noio]`/`[noblock]`).
3. **Composable.** Multiple attributes stack: `[noalloc, nopanic, constanttime] @aes_round(...)`.
4. **Capabilities are contracts in disguise.** Implementation-wise these can be modeled as auto-
   derived marker contracts (like `Sendable`), so they reuse the contract machinery rather than being
   a separate system.
5. **`[unsafe]` is the one escape hatch**, and it cannot silently satisfy a capability — an
   `[unsafe]` block inside a `[noalloc]` fn still may not allocate.

---

## Recommended priority set

If you add a handful beyond the current four, in order:

1. **`[must_use]`** — cheapest win; stops dropped `Result`s.
2. **`[noblock]`** — essential once executors/async exist; prevents scheduler stalls.
3. **`[constanttime]` + `[zeroize]`** — real differentiation; makes Jolt credible for crypto/security.
4. **`[const]` (fn)** — make compile-time-evaluability a declarable promise.
5. **`[threadsafe]`** — rounds out the concurrency safety story alongside `Sendable`/`Shareable`.

The rest (`[total]`, `[bounded_stack]`, taint tracking) are excellent long-term but are research-grade
to implement well — worth listing as a roadmap, not v0.5.
