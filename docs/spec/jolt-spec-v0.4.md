# Jolt — Language Specification (v0.4)

> General-purpose, statically-typed, low-level systems language with move-based memory safety,
> compile-time execution, structured concurrency, and a large toolchain. v0.4 expands v0.3 with
> concurrency, dynamic dispatch, deterministic destruction, real pattern matching, iterators,
> strings, operator overloading, generics-by-value, FFI, and tooling — and renames the borrow
> checker to the **Custodian** (see §9).

This document supersedes v0.3. New or changed sections are marked **(new)** / **(changed)**.

---

## 0. Cross-cutting rule: blocks are expressions (changed)

Every `-> … ;;` block is a **statement list**. Each statement ends with `;`. An optional **final
expression with no `;`** is the block's value (its implicit return). This single rule governs
function bodies, `if`, `match`, loops, and bare blocks.

```jolt
@double(x: Int) Int -> x * 2 ;;          // final expression, no `;` → returned

@classify(n: Int) String ->
    $label = if n < 0 -> "neg" else -> "non-neg" ;;   // if is an expression
    label                                              // final expression → returned
;;

$y = -> $a = 1; $a + 1 ;;                 // bare block expression → y == 2
```

`return` still works for early exit. A block whose last statement ends in `;` has value `None`
(the void type, §10).

---

## 1. Hello, Jolt

```jolt
using Std;

[public]
@main() !None ->
    println("Hello, Jolt!");
;;
```

`@` introduces a function; `@main` is the entry point. `[public]` exposes it. `!None` means "returns
`None` (void) or fails with an error" (§12). Comments use `//`.

---

## 2. Comments

```jolt
// single-line
/// doc comment (repeatable lines), consumed by the doc generator
```

`#` is reserved for macros (§17). No `/* */`.

---

## 3. Naming (enforced)

| Kind | Convention |
| ---- | ---------- |
| variables, functions, methods | `snake_case` |
| types, contracts, enums | `PascalCase` |
| `[const]` constants | `SCREAMING_SNAKE_CASE` (linted) |
| packages, libraries | `PascalCase` |

Unicode identifiers allowed. `;` ends a statement; `->`/`;;` open/close a block; `_` discards.

---

## 4. Variables & bindings

Immutable by default. `$` = new immutable binding, `$$` = new mutable binding, bare name reassigns.

```jolt
$x = 10;          // immutable
$$y = 10; y = 11; // mutable
[const] $Z = 64;  // compile-time constant
$name: String = "Aoi";     // explicit type (colon)
$$slot? = 9.0; slot = "hi"; // ? = re-typeable; only on $$ bindings
$a, $b, _ = 1, 2, 3;        // destructure
```

Negative literals infer to `Int`; bare decimals to `Float64`.

---

## 5. Types

### 5.1 Integers
`Int Uint` (architecture width), `Int8/16/32/64/128`, `Uint8/16/32/64/128`, `Bool`. Aliases:
`Byte`=`Uint8`, `Short`=`Int16`, `Long`=`Int64`.

### 5.2 Floats
`Float16 Float32 Float64 Float128`; `Double`=`Float64`.

### 5.3 Overflow & conversions (new)
- Default arithmetic **traps on overflow in debug builds, wraps in release** (configurable).
- Explicit variants in the prelude: `wrapping_add`, `checked_add` (→ `T?`), `saturating_add`, etc.
- Narrowing/lossy conversions are **never implicit**; use `as`:

```jolt
$big: Int64 = 5_000_000_000;
$small = big as Int32;     // explicit, may truncate
$f = 3 as Float64;          // widening still explicit for clarity
```

`as` also does checked type tests on `dyn` values (§15.4).

### 5.4 Text (new detail)
- `Char` — 32-bit Unicode scalar (`'λ'`).
- `String` — UTF-8 bytes. Concatenate with `+` (via the `Plus` contract, §16). Interpolate with
  `{}`; escape a literal brace as `{{`.

```jolt
$who = "Aoi";
$msg = "hello, {who}! 1+1 = {1 + 1}";   // interpolation
for c in msg.chars() -> ... ;;           // iterate scalars
for b in msg.bytes() -> ... ;;           // iterate raw bytes
$slice = msg[0..5];                       // Slice<Char-view> (§5.7)
```

### 5.5 Composite & built-in
`Raw`, `Array<T>`, `Set<T>`, `Map<K,V>`, `Tuple`, `Pair<A,B>`, `Slice<T>` (§5.7), `Symbol`, `enum`,
`struct`, `union`, `Error`, `None` (void). `Empty` = empty collection only.

```jolt
$nums: Array<Int> = [1, 2, 3];
$lookup: Map<String, Int> = {"a": 1, "b": 2};
```

### 5.6 Fixed-size arrays via value generics (new)
Types may be parameterized by **comptime values** (§19), not just types:

```jolt
$buf: Array<Byte, 256> = Array::zeroed();   // length is part of the type
```

### 5.7 Slices (new)
`Slice<T>` is a borrow-checked view (pointer + length) into contiguous memory — no copy. Slicing
uses ranges:

```jolt
$xs = [10, 20, 30, 40];
$mid: Slice<Int> = xs[1..3];   // view of {20, 30}; bounds-checked
```

### 5.8 Memory & concurrency types
`Borrow<T>` / `Claim<T>` (borrows, §9), `Pointer<T>` (raw, unsafe), `Shared<T>` (`[shared]`, §9.5),
`Channel<T>` (§11), `Mutex<T>`, `Atomic<T>` (§11).

---

## 6. Operators

### 6.1 Precedence (high → low)
```
postfix     a++  a--  a[]  a?[]  a.b  a?.b
prefix      -a   !a   ++a  --a
multiplicative  *  /  %  ^        // ^ = POWER
additive    +  -
shift       <<  >>  <<<  >>>  <<|  >>|
bitwise     &   ~   |   %|   ~&   ~|   ~%|
relational  >=  >  <=  <  as
equality    ==  !=
logical-and &&  and
logical-or  ||  or
negation    not
ternary     c ? a : b
assignment  =  +=  -=  *=  /=  ^=  %=  //=  &=  |=  %|=  …  >>=  <<=  …
range       ..
spread      ...
```

`^` is power; XOR is `%|`; no `**`. **Unicode math aliases (`÷ √ ≠ ≈ ≡`) are removed** — use ASCII /
stdlib (`sqrt`, `!=`, etc.).

### 6.2 Arithmetic
`+ - * / // %  ^` (floor-div `//`, power `^`). `++x/--x` prefix, `x++/x--` postfix.

### 6.3 Bitwise (symbols only)
`& | ~ %|(xor) ~&(nand) ~|(nor) ~%|(xnor)`; shifts `<< >> <<< >>>`, rotates `<<| >>|`.

---

## 7. Functions

```jolt
@double(x: Int) Int -> x * 2 ;;

@greet(name: String) None -> println("Hi, {name}"); ;;
```

### 7.1 Named & default arguments (new)
```jolt
@connect(host: String, port: Uint = 8080, tls: Bool = false) !Conn -> ... ;;

connect("example.com");                 // port=8080, tls=false
connect("example.com", tls: true);      // name later args; skip defaults
```

### 7.2 Generics & value generics
```jolt
@max |T: Comparable| (a: T, b: T) T -> if a.compare(b) >= 0 -> a else -> b ;; ;;

@dot |T: Plus, comptime N: Uint| (a: Array<T, N>, b: Array<T, N>) T -> ... ;;
```

### 7.3 Multiple dispatch with guards (changed: guards see real params)
Guards read declared parameters. Identical signatures = compile error; two guards both true at
runtime = runtime error (the compiler warns on provable static overlap).

```jolt
@convert(m: Float64, unit: String, {unit == "inch"}) Float64 -> m / 2.54 ;;
@convert(m: Float64, unit: String, {unit != "inch"}) Float64 -> m * 2.54 ;;
```

**Comptime guards.** A guard that mentions **only comptime parameters** is evaluated at
monomorphization, not at runtime: a failing instantiation is a **compile error** and a passing one
emits **no runtime check** — so it acts as a compile-time constraint (like a `where` bound). A guard
touching any runtime value stays runtime-checked; a mixed guard is split (comptime part gates
instantiation, runtime part checked at the call).

```jolt
@head |T, comptime N: Uint| (a: Array<T, N>, {N > 0}) T -> a[0] ;;   // {N > 0} is compile-time
$x = head([1, 2, 3]);        // N=3 → ok, zero runtime cost
// $y = head(empty_array);   // N=0 → COMPILE ERROR, not a runtime failure
```

### 7.4 Closures
```jolt
$inc = |x: Int| Int -> x + 1 ;;
$doubled = map(nums, |x| -> x * 2 ;;);
```

---

## 8. Control flow

### 8.1 if / else (an expression, §0)
```jolt
$t = if score >= 60 -> "pass" else -> "fail" ;;

if ->                       // condition-list form
    x < 0:  println("neg");
    x == 0: println("zero");
    default: println("pos");
;;
```

### 8.2 match (pattern matching) — the only multi-branch construct (changed/expanded)
`match` is Jolt's pattern-matching construct and an expression. `switch` has been **removed** — with
literal arms, or-patterns, and range patterns, `match` covers every case `switch` did.

Supported patterns:
- literals: `200`, `"red"`, `true`
- enum variants with destructuring: `Circle(r)`, `Rect(w, h)`
- tuples/structs: `(x, y)`, `Point{x, y}`
- wildcard `_`; rest `..`
- or-patterns: `"y" | "yes"`
- arm guards: `Circle(r) if r > 0.0 ->`
- whole-value capture with `:=` (reused; `:=` is otherwise unused): `c := Circle(r) ->`
- range patterns: `0..18`

```jolt
$desc = match shape ->
    Circle(r) if r > 0.0  -> "circle area {3.14159 * r ^ 2}";
    Circle(_)             -> "degenerate circle";
    Rect(w, h)            -> "rect {w}x{h}";
    whole := Tri(a, b, c) -> describe_tri(whole);
;;

match code ->
    200 | 201 | 204 -> "ok";
    400..500        -> "client error";
    _               -> "other";
;;
```

Exhaustiveness is enforced for closed types (enums, closed `Result`); open types need a `_`/`default`.

### 8.3 Loops (with labels — C2)
```jolt
loop -> if done -> break; ;; step(); ;;       // infinite
for i in 0..10 -> println(i); ;;               // iterate
for x < 100 -> x = x * 2; ;;                    // "while" form

[label: outer]
for row in grid ->
    for cell in row ->
        if cell == target -> break outer; ;;   // labeled break
    ;;
;;
```
`break [label]`, `next [label]`, `return`.

---

## 9. Memory & the Custodian (changed: renamed from borrow checker)

Jolt is **move-by-default with compile-time custody analysis** — the pass that guarantees memory
safety is the **Custodian**. Every value has a single owner; the Custodian tracks who may read or
write it at any moment. A violation is reported as a **custody violation** ("custody violation:
`data` is already claimed mutably"). No implicit copy, no null.

> **Terminology note.** v0.3 called this "the borrow checker" with `ref`/`ref_mut`/`unref`. v0.4
> renames the checker to the **Custodian** and the borrow operations to `borrow`/`claim`/`deref`.
> The semantics are identical to a Rust-style borrow checker (shared-XOR-mutable, move,
> non-lexical); only the vocabulary differs.

### 9.1 Moves
```jolt
$a = Buffer::new();
$b = a;            // a MOVED into b; using a now is a custody violation (compile error)
```

### 9.2 Copy types
A type copies implicitly **iff it conforms to the `Copy` contract** (primitives do); otherwise move,
or `copy(x)` explicitly.

```jolt
$x = 5; $y = x;       // Int is Copy → x still valid
$dup = copy(buffer);  // explicit
```

### 9.3 Borrows
`borrow(x)` = shared read-only borrow (`Borrow<T>`); `claim(x)` = exclusive mutable borrow
(`Claim<T>`, requires a `$$` source). `deref(b)` reads/writes through a borrow (never frees). Rule:
**many shared borrows XOR one claim**. Borrows are **non-lexical** (end at last use).

```jolt
$$data = Buffer::new();
$v1 = borrow(data); $v2 = borrow(data);   // multiple shared borrows: fine
read(deref(v1));
$e = claim(data);                          // custody violation if v1/v2 still live
write(deref(e), 42);
```

### 9.4 Lifetimes
Inferred; no syntax in normal code. Rare escape: declare with `life` in the generic bracket and
attach as a second type parameter.

```jolt
@first |life L| (hay: Borrow<Array<Int>, L>, needle: Int) Borrow<Int, L> -> ... ;;
```

### 9.5 Shared ownership
`[shared]` → counted ownership (`Shared<T>`), non-atomic. `[shared, sync]` → atomic, thread-safe.
Cycles leak unless broken with a `[weak]` reference.

```jolt
[shared] $tree = Node::new();
$alias = tree;                 // both own; freed at last drop
child.parent = weak(tree);     // break the cycle
```

### 9.6 Deterministic destruction — `Dispose` (new, B3)
Memory frees automatically, but other resources (files, sockets, locks) need ordered teardown. A
type that conforms to the **`Dispose`** contract gets its `@dispose($$self)` called automatically
when its owner goes out of scope or is dropped (RAII). This is Jolt's destructor mechanism — distinct
from `defer` (which is per-scope, ad hoc).

```jolt
struct File -> $$handle: Raw; ;;

File::Dispose ->
    @dispose($$self) None -> os_close(self.handle); ;;
;;

@read_once(path: String) !String ->
    $f = File::open(path)?;    // f.dispose() runs automatically at scope exit,
    read_all(borrow(f))           // including on the error path
;;
```

### 9.7 Raw pointers & `[unsafe]`
`ptr()`/`Pointer<T>` exist only in `[unsafe]`. `[unsafe]` unlocks exactly: (1) raw pointer
create/deref, (2) pointer arithmetic, (3) calling `[unsafe]` fns, (4) `union` field access,
(5) inline `asm`. It does **not** disable the Custodian on safe borrows.

### 9.8 Allocators
Implicit default allocator + explicit via `[alloc: name]` (on fn/block/type). Containers remember
their allocator. Swap the process default with `[alloc: …]` on `@main`.

```jolt
[alloc: arena]
@build() !Tree -> ... ;;
```

---

## 10. Absence: None vs Option (changed, A6)

- `None` is the **void type** (functions that return nothing).
- `Option<T>` represents optional values with cases **`Some(value)`** and **`Nothing`** — *not*
  `None`. `T?` is sugar for `Option<T>`. There is no null.

```jolt
@find(xs: Array<Int>, target: Int) Int? ->
    for i in 0..len(xs) -> if xs[i] == target -> return Some(i); ;; ;;
    Nothing
;;

$$idx = find(nums, 7) ?? -1;          // ?? = Option coalesce
match find(nums, 7) ->
    Some(i) -> println("at {i}");
    Nothing -> println("absent");
;;
```

`??` is Option-only; `?` is error-only (§12); mixing is a compile error.

---

## 11. Concurrency & I/O (new, B1)

Jolt separates two concerns and makes **both** memory-safe by the same mechanism — the Custodian plus
two safety contracts:
- **Concurrency** (running work in parallel): two models, **A structured** and **B raw threads**.
- **I/O** (waiting on the outside world): two **tiers**, a zero-runtime **completion-based**
  foundation and an ergonomic **green-threaded** layer on top. There is **no `async`/`await`** —
  green threads remove function coloring entirely.

Whatever you choose, the compiler rejects data races and use-after-move across threads. Safety is not
a property of the model; it's a property of the type system beneath all of them.

### 11.1 The universal safety layer
Two contracts gate everything that crosses a thread boundary, in *every* model and tier:
- **`Sendable`** — a value of this type may be **moved** to another thread.
- **`Shareable`** — a `Borrow` of this type may be **shared** across threads.

(These are the `Send`/`Sync` analogues, renamed.) Most types derive them automatically. Types holding
raw pointers or non-atomic `[shared]` are **not** `Sendable`/`Shareable`, and the Custodian refuses
to move or share them across threads — a **custody violation** at compile time. Non-atomic
`[shared]` crossing a thread boundary is rejected (use `[shared, sync]`), never silently upgraded.

> **Key guarantee.** The models and tiers below differ in *ergonomics and control*, never in
> *safety*. You cannot pick a "less safe but faster" option; you pick a more or less *convenient* one.

### 11.2 Concurrency Model A — Structured (default, recommended)
`scope` opens a concurrency scope; `spawn` starts a task **guaranteed to finish before the scope
exits**. No task outlives its scope, so borrows into the scope are provably valid — the safest and
easiest model, and the one to reach for first.

```jolt
using Concurrent;

@parallel_sum(data: Array<Int>) Int ->
    $$total = Atomic::new(0);
    scope ->
        for chunk in data.chunks(1024) ->
            spawn -> total.add(sum(chunk)); ;;   // chunk must be Sendable
        ;;
    ;;   // all spawned tasks joined here — borrows guaranteed live throughout
    total.load()
;;
```

Because the scope joins all tasks, a `spawn` inside it may safely **borrow** scope-local data
(`Shareable`), not just own `Sendable` values — the Custodian can prove the data outlives the tasks.

### 11.3 Concurrency Model B — Raw threads (manual, for long-lived/detached work)
`Thread::spawn` returns a `Thread` handle you join explicitly. A detached thread can outlive the
spawning scope, so the Custodian only permits it to capture **`Sendable` owned values** (moved in),
never borrows — that restriction is what keeps it safe without a scope.

```jolt
$handle = Thread::spawn(|| -> heavy_work(owned_data) ;;);  // owned_data MOVED in
// ... do other work ...
$result = handle.join()?;     // retrieve the result (or error)
```

Try to capture a borrow here and you get a custody violation: a detached thread has no proof the
borrowed data still exists.

### 11.4 I/O — two tiers, no async/await
Jolt has **no `async`/`await`** and no function coloring. Instead, I/O comes in two layers:

**Tier 1 — Completion-based I/O (`io`, zero-runtime foundation).** The lowest level: submit
operations to a queue and harvest completions yourself (io_uring / IOCP style). No scheduler, no
hidden runtime — suitable for kernels, embedded, and building your own executors. Explicit and
verbose by design.

```jolt
using Io;

$$ring = io.Ring::new(entries: 64);
$op = ring.submit(io.read(fd, borrow(buf)));   // queue a read; returns a Completion handle
// ... submit more, do other work ...
$n = ring.wait(op)?;                             // harvest one completion
```

**Tier 2 — Green threads / fibers (`fiber`, ergonomic default).** Write ordinary blocking-looking
code; the fiber runtime parks the *fiber* (not the OS thread) on a blocking call and runs others
meanwhile. No coloring — a normal function works whether called from a fiber or not — and it sits on
top of Tier 1. This is what application code uses.

```jolt
using Fiber;

@fetch(url: String) !Bytes ->
    $conn = connect(url)?;        // looks blocking; the fiber parks, thread keeps working
    conn.read_all()               // no `await`, no `async` — just normal code
;;

@main() !None ->
    scope ->                       // structured concurrency drives the fibers
        $a = spawn -> fetch("a.com"); ;;   // each spawn is a fiber
        $b = spawn -> fetch("b.com"); ;;
    ;;   // both joined here; I/O overlapped automatically
;;
```

Because Tier 2 reuses `scope`/`spawn` (Model A), concurrent I/O is just structured concurrency over
fibers — the same safety guarantees apply, and there's no second "async" world to learn. Code that
needs zero runtime drops to Tier 1; everything else uses Tier 2.

A `[noblock]` function (§21) is statically guaranteed not to park a fiber or block a thread — the
compiler rejects any blocking call inside it, which keeps executors and realtime paths stall-free.

### 11.5 Message passing & sync primitives (work everywhere)
Channels and locks are model- and tier-agnostic — usable from structured tasks, raw threads, and
fibers.

```jolt
$ch: Channel<Int> = Channel::new(capacity: 16);
spawn -> ch.send(compute()); ;;
$value = ch.recv();          // parks the fiber / blocks the thread until a value arrives
```

Stdlib: `Channel<T>`, `Mutex<T>` (lock returns a `Claim`-like guard that `Dispose`s on scope exit),
`RwLock<T>`, `Atomic<T>`, `Once`, `Barrier`.

```jolt
$m = Mutex::new(0);
scope -> spawn ->
    $g = m.lock();           // guard; auto-unlocks via Dispose
    deref(g) = deref(g) + 1;
;; ;;
```

### 11.6 Choosing a model / tier
| Need | Use |
| ---- | --- |
| parallelism within a function/scope, borrows allowed | **Concurrency A: `scope`/`spawn`** |
| long-lived or detached worker, owns its data | **Concurrency B: `Thread::spawn`** |
| ergonomic I/O, blocking-style code, high concurrency | **I/O Tier 2: fibers (`spawn` in a `scope`)** |
| zero-runtime I/O for kernel/embedded/custom executors | **I/O Tier 1: completion-based `io`** |

Everything shares `Channel`/`Mutex`/`Atomic` and the `Sendable`/`Shareable` guarantees.

---

## 12. Error handling

Errors are values; no exceptions.

- `!T` — returns `T` or fails; error type is an **inferred open error set** (Zig-style).
- `Result<T, E>` — explicit `E`, **closed** and exhaustively matchable. `!T` is sugar for
  `Result<T, E>` with inferred open `E`, so `?` works on both.

```jolt
@read_config(path: String) !Config ->
    $f = open(path)?;            // ? propagates, unions into this fn's set
    parse(read_all(borrow(f))?)
;;

error NotFound;                   // declare an error; Error = root contract
error Timeout;

@load(p: String) Result<Data, LoadError> -> ... ;;   // closed set → exhaustive match
```

`defer` runs on every scope exit (LIFO); `errdefer` runs only on error exit.

```jolt
@process(p: String) !None ->
    $f = open(p)?;
    defer close(f);
    $buf = alloc(1024)?;
    errdefer free(buf);
    fill(buf)?;
;;
```

---

## 13. Visibility (changed: granularity, B7)

Private by default. `[public]` exposes to importers; `[public: package]` exposes only within the
current package (not to external dependents).

```jolt
@helper() None -> ... ;;            // private to its library
[public: package] @internal() None -> ... ;;
[public] @api() None -> ... ;;
```

---

## 14. Structs, enums, unions (changed: field mutability rule, A3/A8)

Each field declares its own mutability with `$` / `$$`. A mutable field can be changed **only when
the owning binding is itself mutable**; an immutable binding freezes all fields regardless of their
markers.

```jolt
struct Person ->
    $name: String       // immutable field
    $$age: Uint         // mutable field
;;

union Word ->
    $as_int: Uint32     // union fields take sigils too (A8)
    $as_bytes: Array<Byte>
;;

$p  = Person { name: "Aoi", age: 30 };
// p.age = 31;          // UNGROUNDED: p is an immutable binding
$$q = Person { name: "Bo", age: 20 };
q.age = 21;             // ok: q is mutable AND age is a $$ field
// q.name = "X";        // error: name is an immutable field
```

Methods live in `Type::method` blocks with an explicit receiver: `self` (shared) or `$$self`
(mutable).

```jolt
Person::greet(self) None -> println("Hi, {self.name}"); ;;
Person::birthday($$self) None -> self.age = self.age + 1; ;;
```

---

## 15. Contracts: static & dynamic (changed: adds dyn, B2)

Contracts are Jolt's single abstraction mechanism (interfaces + generic bounds). Trait-style: no
inheritance, no instance data; required methods + optional defaults.

### 15.1 Definition & adoption
```jolt
@@Comparable ->
    @compare(self, other: Self) Int;
    @max(self, other: Self) Self -> if self.compare(other) >= 0 -> self else -> other ;; ;;
;;

Person::Comparable -> @compare(self, o: Self) Int -> self.age - o.age ;; ;;
```

### 15.2 Static use (zero-cost, monomorphized)
```jolt
@largest |T: Comparable| (xs: Array<T>) T? ->
    if len(xs) == 0 -> return Nothing; ;;
    $$best = xs[0];
    for i in 1..len(xs) -> best = best.max(xs[i]); ;;
    Some(best)
;;
```

### 15.3 Dynamic use — `dyn` (new)
For heterogeneous collections and runtime polymorphism, `dyn Contract` is a boxed object with a
vtable. The cost (indirection + box) is explicit in the type.

```jolt
@@Drawable -> @draw(self) None; ;;

$$shapes: Array<dyn Drawable> = [];
shapes.push(Circle::new(5.0));
shapes.push(Rect::new(2.0, 3.0));
for s in shapes -> s.draw(); ;;        // dynamic dispatch
```

### 15.4 Downcasting
`as` does a checked downcast on a `dyn` value, yielding an `Option`:

```jolt
match (s as Circle) ->
    Some(c) -> println("radius {c.radius}");
    Nothing -> println("not a circle");
;;
```

### 15.5 Operator overloading via contracts (new, C5)
Overloadable operators are contracts named for the operation. The method name is the operator
wrapped in parentheses after the `@` function sigil — `@(+)`, `@(==)`, etc. The parentheses keep the
parser unambiguous (the `@` sigil expects a name; `(+)` names the operator).

```jolt
@@Plus   -> @(+)(self, other: Self) Self; ;;     // also: Minus, Times, Over, Mod, Power
@@Equals -> @(==)(self, other: Self) Bool; ;;

Complex::Plus ->
    @(+)(self, o: Self) Self -> Complex::new(self.re + o.re, self.im + o.im) ;;
;;

$z = a + b;     // dispatches to Complex::Plus
```

### 15.6 Common built-in contracts
`Copy`, `Dispose`, `Sendable`, `Shareable`, `Comparable`, `Equals`, `Plus`/`Minus`/`Times`/`Over`,
`Iterator`/`Iterable` (§18), `Display` (for `to_string`/interpolation).

---

## 16. Strings & formatting (new section, B6)

- Concatenation: `+` (via `Plus` on `String`).
- Interpolation: `"…{expr}…"`; `{{`/`}}` for literal braces.
- `Display` contract powers interpolation & `to_string`.
- Iterate `.chars()` (scalars) or `.bytes()` (raw UTF-8).
- Slice with ranges (`s[a..b]`), bounds-checked.

```jolt
struct Money -> $cents: Uint; ;;
Money::Display -> @show(self) String -> "${self.cents / 100}.{self.cents % 100}" ;; ;;

$m = Money { cents: 1599 };
println("price: {m}");     // uses Display → "price: $15.99"
```

---

## 17. Macros

`#`-prefixed (since comments use `//`). Two tiers: hygienic **declarative** pattern macros, and
**procedural** macros that run at comptime (§19) to emit code. User attributes (§11/§14 etc.) are
procedural macros applied to a declaration.

```jolt
#macro swap(a, b) -> $tmp = a; a = b; b = $tmp; ;;   // hygienic
#swap(x, y);
```

---

## 18. Iterators (new, B5)

`for` desugars to the **`Iterator`** contract; anything `Iterable` can be looped. Ranges (`..`) and
collections are ordinary `Iterator`s, and lazy adapters compose.

```jolt
@@Iterator -> @next($$self) ItemType?; ;;     // Nothing ends iteration
@@Iterable -> @iter(self) (it: dyn Iterator); ;;

// custom iterator
struct Countdown -> $$n: Uint; ;;
Countdown::Iterator ->
    @next($$self) Uint? ->
        if self.n == 0 -> Nothing
        else -> self.n = self.n - 1; Some(self.n + 1) ;;
    ;;
;;

for x in Countdown { n: 3 } -> println(x); ;;   // 3, 2, 1

// lazy adapters
$evens = (0..100).filter(|n| -> n % 2 == 0 ;;).take(5);   // 0,2,4,6,8
```

---

## 19. Compile-time execution & reflection (expanded, C7)

`comptime` evaluates values/args at compile time (also backs procedural macros). It is for
value-level work and codegen — generics come from contracts, not comptime.

```jolt
@pow2(comptime n: Uint) Uint -> 1 << n ;;
comptime -> $$TABLE = build_table(); ;;
```

Comptime **reflection** API (via `typeinfo`): iterate fields, read type names/sizes, inspect
contracts a type conforms to — enough to write serializers/derive-style macros as comptime code.

```jolt
#macro derive_show(T) ->                      // a comptime/proc macro
    comptime ->
        $$out = "";
        for f in typeinfo(T).fields -> out = out + "{f.name}={...};"; ;;
        emit_method(T, "show", out);
    ;;
;;
```

---

## 20. FFI & inline asm (expanded, B10)

Call external C with `extern` + a calling-convention attribute; inline `asm`/`c` blocks remain.

```jolt
[extern: "C"]
@malloc(size: Uint) Pointer<None>;        // declaration only; linked externally

[extern: "C", link: "m"]
@cos(x: Float64) Float64;

asm -> movq $60, %rax; syscall; ;;
c   -> /* inline C compiled & linked */ ;;
```

---

## 21. Safety & capability attributes (new, C10)

Attributes annotate a function (or block/type) with guarantees the compiler **enforces
transitively**: a function may only call others compatible with the capabilities it declares. The
stdlib is annotated so user guarantees check against it (e.g. `alloc` is not `[noalloc]`, `println`
is not `[noio]`/`[noblock]`). Capabilities are modeled as auto-derived marker contracts (like
`Sendable`), reusing the contract machinery rather than a separate system. `[unsafe]` is the one
escape hatch and **cannot** silently satisfy a capability — an `[unsafe]` block inside `[noalloc]`
still may not allocate.

```jolt
[noalloc, nopanic, constanttime]
@aes_round(state: Block, key: Block) Block -> ... ;;   // attributes stack
```

### 21.1 Resource-use restrictions
| Attribute | Guarantees |
| --------- | ---------- |
| `[noalloc]` | no heap allocation (realtime/embedded hot paths) |
| `[noio]` | no I/O — files, net, syscalls (pure compute, sandboxing) |
| `[nopanic]` | no trapping/panicking path; all errors are values |
| `[noblock]` | will not block the thread or park a fiber (no lock waits, no blocking I/O) — keeps executors and realtime paths stall-free |
| `[nostd]` | no standard library, only core/intrinsics (freestanding/bare-metal) |
| `[norecurse]` | no recursion, direct or mutual (bounded stack) |
| `[bounded_stack: N]` | provably uses ≤ N bytes of stack (hard realtime, interrupt handlers) |

### 21.2 Purity & determinism
| Attribute | Guarantees |
| --------- | ---------- |
| `[pure]` | no side effects, no I/O, no alloc, no arg mutation; same inputs → same output |
| `[constfn]` | the function is evaluable at compile time — a declarable promise, stronger than inferred `comptime`. (Distinct from `[const]` on a *binding*, §4, which declares a compile-time constant value.) |
| `[idempotent]` | calling twice == calling once |
| `[total]` | provably terminates and is defined for all inputs (no infinite loops, no partial functions) |

### 21.3 Concurrency & memory (tie into the Custodian, §9/§11)
| Attribute | Guarantees |
| --------- | ---------- |
| `[shared]` / `[shared, sync]` / `[weak]` | counted ownership / atomic / cycle-breaking (§9.5) |
| `[unsafe]` | unlocks raw pointers, pointer arithmetic, unsafe calls, union access, inline `asm` (§9.7) |
| `[threadsafe]` | checked safe to call from multiple threads concurrently |
| `[atomic]` | a block executes as one uninterruptible unit (hardware-backed where possible) |
| `[main_thread]` | may only run on the main thread (UI toolkits, some OS APIs) |
| `[no_capture]` | a closure parameter may not capture its environment (no hidden escaping references) |

### 21.4 Security & information-flow
| Attribute | Guarantees |
| --------- | ---------- |
| `[constanttime]` | execution time independent of secret inputs — no secret-dependent branches/indexing (defeats timing side channels) |
| `[zeroize]` | the value's memory is wiped on drop (keys, passwords) — built on `Dispose` (§9.6) |
| `[tainted]` / `[untrusted]` | marks external-input data; tracked by the compiler until `[sanitized]` |
| `[secret]` | may not be logged, printed, or sent to I/O without explicit declassification (PII, credentials) |

### 21.5 API evolution & lint
| Attribute | Meaning |
| --------- | ------- |
| `[deprecated{message, since}]` | warns on use |
| `[must_use]` | warn if the return value is discarded — especially on `Result`/`!T`, stops silently-ignored errors |
| `[experimental]` | requires an opt-in flag; API may change |
| `[stable: "x.y"]` | marks the version an API stabilized in |

### 21.6 Enforcement rules
1. **Transitive** — the compiler walks the call graph; a violation names the offending call.
2. **Stdlib annotated** — core/stdlib carry these so user guarantees can be verified.
3. **Composable** — attributes stack: `[noalloc, nopanic, constanttime]`.
4. **Contracts under the hood** — capabilities are auto-derived marker contracts.
5. **`[unsafe]` is not an exemption** — it never silently satisfies a capability.

```jolt
[pure]    @hash(x: Int) Int -> ... ;;       // no side effects, no I/O, no alloc
[noblock] @poll() Event? -> ... ;;          // safe to call from an executor
[must_use] @parse(s: String) !Config -> ... ;;   // caller must handle the Result
```

---

## 22. Testing & docs (new, C8/C9)

- `[test]` marks a test function; the toolchain discovers and runs them.
- `assert(cond, msg?)` is a prelude function (traps on failure in test/debug).
- Doc comments (`///`) may contain runnable **doctest** code fences the toolchain executes.

```jolt
/// Adds two ints.
/// ```
/// assert(add(2, 3) == 5);
/// ```
@add(a: Int, b: Int) Int -> a + b ;;

[test]
@adds_positives() None -> assert(add(2, 3) == 5); ;;
```

---

## 23. Built-in / prelude functions

Ordinary stdlib functions (not keywords), auto-imported: `len`, `log`, `print`, `println`,
`to_string`, `assert`, `copy`, `typeinfo`, `help`, numeric helpers (`wrapping_add`, `checked_add`,
`saturating_add`, `sqrt`, …).

---

## 24. Modules, packages & imports

Path-based, left→right. Tiers: `library` (reuse/import unit) → `package` (distribution) →
`program` (has `@main`, builds an executable).

```jolt
using Math;
import Utils.Print;
from Utils import Print as P;
export { Add as add, Sub as sub };

package MyApp;
library MyApp.Core;
program -> using MyApp.Core; [public] @main() !None -> ... ;; ;;
```

---

## 25. Keywords (reserved)

```
// packages & items
using import export from as package library program module macro

// control flow
if else for loop match then return next break default

// memory, safety, concurrency
borrow claim deref ptr weak life defer errdefer comptime asm c
scope spawn

// types & decls
struct enum union error type in implicit empty extern dyn

// literals
true false None Some Nothing Ok Err Self self
```

Removed since earlier drafts: `when do while repeat until imply infer` (and `ref`/`ref_mut`/`unref`,
replaced by `borrow`/`claim`/`deref`; and `switch`/`case`, now subsumed by `match`). Loops are
`loop`/`for`; multi-branch dispatch is `match`.

---

## Appendix A — A worked program

```jolt
using Std;
using Concurrent;

/// A 2-D point.
struct Point -> $$x: Float64; $$y: Float64; ;;

Point::distance_to(self, o: Self) Float64 ->
    sqrt((self.x - o.x) ^ 2 + (self.y - o.y) ^ 2)
;;

@@Named -> @name(self) String; @describe(self) String -> "I am {self.name()}" ;; ;;
Point::Named -> @name(self) String -> "a point" ;; ;;

@@Drawable -> @draw(self) None; ;;
Point::Drawable -> @draw(self) None -> println("• {self.x},{self.y}"); ;; ;;

[public]
@main() !None ->
    $a = Point { x: 0.0, y: 0.0 };
    $b = Point { x: 3.0, y: 4.0 };
    println("distance = {a.distance_to(b)}");      // 5.0
    println(a.describe());

    // dynamic dispatch over a heterogeneous list
    $$scene: Array<dyn Drawable> = [];
    scene.push(a); scene.push(b);
    for s in scene -> s.draw(); ;;

    // structured concurrency + channel
    $ch: Channel<Int> = Channel::new(capacity: 4);
    scope ->
        spawn -> ch.send(compute_heavy()); ;;
    ;;
    println("result = {ch.recv()}");

    // option + match
    $$first_even = [1, 3, 4, 7].iter().find(|n| -> n % 2 == 0 ;;);
    match first_even ->
        Some(n) -> println("first even {n}");
        Nothing -> println("none");
    ;;
;;
```

---

*End of v0.4. See `jolt-changes-v0.4.md` for the delta from v0.3 and the open questions this round
surfaced.*
