# Year 3 — Concurrency, the Full Language, and the Standard Library

> Phases 7–9. Outcome: the **complete v0.4 language** plus a substantial standard library — Jolt as
> specified, end to end (still Rust-hosted; toolchain breadth comes in Year 4).

---

## Phase 7 — Concurrency & I/O

**Objective.** Implement the concurrency models and two-tier I/O, with thread-safety enforced by the
type system.

**Implements (docs).** `jolt-spec-v0.4.md` §concurrency (structured `scope`/`spawn`, raw threads,
`Sendable`/`Shareable`) & §I/O (Tier 1 `Io`, Tier 2 `Fiber`, no async); `jolt-stdlib-outline.md`
`Io`/`Fiber`/`Concurrent` (+ their low-level extensions); `jolt-decisions.md` (concurrency rationale,
why no async/await); `jolt-testing.md` §9 (DST foundation that rides on this).

**Workstreams**
1. **Safety contracts:** `Sendable`/`Shareable` (auto-derived); the Custodian rejects cross-thread
   moves/shares of non-conforming types and non-atomic `[shared]`.
2. **Structured concurrency:** `scope`/`spawn` with join-before-exit; borrows into the scope proven
   valid.
3. **Raw threads:** `Thread::spawn` (owned-capture only).
4. **I/O Tier 1 (completion-based, `Io`):** ring/submit/complete; OS backends (io_uring/IOCP/kqueue).
5. **I/O Tier 2 (fibers, `Fiber`):** green-thread runtime over Tier 1; blocking-style code, no
   coloring; `[noblock]` enforcement.
6. **Sync primitives:** `Channel`, `Mutex` (guard via `Dispose`), `RwLock`, `Atomic`, `Once`,
   `Barrier`; `[shared, sync]` atomic ownership; `[weak]`.

**Deliverables.** Working concurrency + I/O; data races rejected at compile time.

**Definition of Done**
```
[ ] scope/spawn run concurrently and join at scope exit; borrows into scope verified
[ ] Sending a non-Sendable value across threads is a compile error (custody/capability)
[ ] Non-atomic [shared] across a thread boundary is rejected (no silent upgrade)
[ ] Fibers run blocking-style I/O without coloring; [noblock] rejects blocking calls
[ ] Channels/Mutex/Atomic work; Mutex guard auto-unlocks via Dispose
[ ] At least one OS completion backend functional; fiber runtime on top of it
[ ] tests/sim: a basic deterministic concurrency test passes (full DST in Phase 11)
```

**Verification Gate.** Concurrency correctness suite passes; a curated set of "would-be data races"
all fail to compile; a fiber-based I/O sample overlaps work correctly; `[noblock]` violations are
rejected. Sign-off: runtime owner + language lead.

---

## Phase 8 — Full Language Features

**Objective.** Finish the remaining v0.4 surface so the language is feature-complete.

**Implements (docs).** `jolt-spec-v0.4.md` §macros, §operator-overloading, §FFI/`extern`, §inline
asm/c, §string-interpolation, §lifetimes-escape-hatch; `jolt-grammar.md` §5 (`@(+)` operators), §11
(extern/macro forms), §1/§8.3 (interpolation + format specs); `jolt-stdlib-outline.md` `Abi` (FFI);
`jolt-changes-v0.4.md` #10 (interpolation resolution).

**Workstreams**
1. **Macros:** declarative (`#macro`, hygienic) + procedural (comptime-backed); `[attr]` user
   attributes as proc-macros.
2. **Operator overloading:** `@(+)`-style contract methods (`Plus`/`Minus`/`Equals`/…).
3. **FFI:** `extern` declarations, `[extern: "C"]`, calling conventions, `CStr`/`CString`, callbacks.
4. **Inline asm** (`asm`) and inline C (`c`) blocks; `[unsafe]` powers fully enforced.
5. **String interpolation** with format specifiers (`{x:hex}`) finalized; `Display`/`Debug`.
6. **Lifetimes escape hatch** (`|life L|`) for the rare ambiguous case.

**Deliverables.** A language matching `jolt-spec-v0.4.md` in full.

**Definition of Done**
```
[ ] Declarative + procedural macros expand correctly and hygienically
[ ] Operator overloading via @(+) works; resolves through contracts
[ ] FFI calls into a C library work; emitted C-ABI surface callable from C
[ ] Inline asm/c blocks compile and run; [unsafe] gates exactly the 5 powers
[ ] Interpolation + format specifiers implemented; Display/Debug wired
[ ] Lifetime escape hatch works on the ambiguous cases that need it
[ ] All v0.4 spec worked examples (incl. Appendix A) compile and run
```

**Verification Gate.** The v0.4 spec's every code example compiles and runs; macro hygiene tests pass
(no accidental capture); an FFI round-trip (call C, be called by C) works; differential test still
green across the now-full language. Sign-off: language lead.

---

## Phase 9 — Standard Library

**Objective.** Build the rich stdlib (written in Jolt) per `jolt-stdlib-outline.md`, including the
low-level layer.

**Implements (docs).** `jolt-stdlib-outline.md` (entire — all ~20 modules + the low-level/systems
layer + each module's low-level extensions + the three nostd/noalloc flavors); `jolt-spec-v0.4.md`
§contracts (Core contracts) & §comptime (`Reflect`); `jolt-safety-attributes.md` (`[constanttime]`/
`[zeroize]` for `Crypto`, `[noalloc]` flavors); `jolt-compiletime-safety.md` (typed-config story).

**Workstreams**
1. **Core + Prelude:** the foundational contracts (`Copy`, `Dispose`, `Clone`, `Comparable`, `Hash`,
   `Display`, `Iterator`/`Iterable`, `From`/`Into`, …) and prelude functions; minimal Rust intrinsic
   surface, everything else in Jolt.
2. **Data + text + math:** `Mem`, `Collections`, `Text` (incl. interpolation/regex/unicode), `Iter`,
   `Math` (Complex/Rational/BigInt/LinAlg/Random).
3. **System:** `Time`, `Fs`, `Net`, `Os`, building on `Io`/`Fiber` (Phase 7).
4. **Data interchange & security:** `Encoding` (JSON/… + `Serialize`/`Deserialize` derive), `Crypto`
   (with `[constanttime]`/`[zeroize]`).
5. **Low-level layer:** `Layout`, `Ptr`, `Volatile`, `Bits`, `Abi`, `Intrin`/`Simd`, `Arch`,
   `Interrupt`, `Boot`, `Embed`, `Inline`.
6. **`[nostd]`/`[noalloc]` flavors** of the library for freestanding/embedded targets.
7. `Log` and `Reflect` (comptime reflection over `typeinfo`).

**Deliverables.** A usable, documented standard library with three flavors (full / nostd+alloc /
nostd+noalloc).

**Definition of Done**
```
[ ] Core contracts + prelude complete; auto-imported
[ ] Collections/Text/Iter/Math usable and unit-tested (in Jolt)
[ ] Io/Fiber-backed Fs/Net/Os/Time work on at least one OS target
[ ] Encoding (JSON + derive) and Crypto (with constant-time primitives) work
[ ] Low-level layer compiles for an embedded target; a [noalloc] blink example builds
[ ] nostd/noalloc stdlib flavors build and select automatically by target
[ ] Stdlib has its own test suite (written with the early test runner) and docs
```

**Verification Gate.** Stdlib test suite green on full and `nostd+noalloc` flavors; a non-trivial
program (e.g. a small HTTP client) works using only the stdlib; an embedded sample builds for a
microcontroller target and runs in an emulator. Sign-off: stdlib owner + language lead.

---

### Year-3 exit state
The full Jolt language as specified, with concurrency, all features, and a rich standard library
(including bare-metal support) — everything except toolchain breadth and self-hosting.
