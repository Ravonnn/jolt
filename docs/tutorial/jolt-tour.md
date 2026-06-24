# A Tour of Jolt

> A friendly, build-as-you-go introduction to Jolt — a low-level systems language that's safe by
> default. No prior Jolt needed; some programming experience assumed. Work top to bottom; each
> section builds on the last. (Reference: `jolt-spec-v0.4.md`. Cheat sheet: `jolt-cheatsheet.md`.)

---

## 1. Hello, Jolt

```jolt
using Std;

[public]
@main() !None ->
    println("Hello, Jolt!");
;;
```

Run it:
```
jolt run
```

> **Runnable now (Tiny):** [`tests/tutorial/hello.jolt`](../../tests/tutorial/hello.jolt) —
> `jolt run --interpret tests/tutorial/hello.jolt` (no `using` / `[public]` / `!None` in Tiny yet).

Let's read every piece, because a few things are unusual:
- `using Std;` brings in the standard prelude.
- `@main` — functions start with `@`. `main` is where programs begin.
- `[public]` — an **attribute** (in `[ ]`). It makes `main` visible outside its library.
- `->` opens a block, `;;` closes it. (Not braces. You'll get used to it fast.)
- `!None` is the return type: "returns nothing, *or fails with an error*." More on the `!` in §8.
- `//` starts a comment. `///` is a documentation comment.

> **Why `->`/`;;`?** Jolt blocks are *expressions* — they can produce a value. The arrow/semicolon
> pair makes that uniform across functions, `if`, `match`, and loops. You'll see the payoff in §6.

---

## 2. Values and bindings

Jolt is **immutable by default** — a strong nudge toward safer code.

```jolt
$x = 10;          // a new IMMUTABLE binding ($)
// x = 11;        // ERROR: x can't change

$$y = 10;         // a new MUTABLE binding ($$)
y = 11;           // fine

[const] $Z = 64;  // a compile-time constant
```

The `$` sigil means "I'm introducing a new name." Reassigning an existing name uses no sigil
(`y = 11`). Two dollar signs (`$$`) means "and this one can change."

Types are inferred, but you can annotate with a colon:

```jolt
$name: String = "Aoi";
$$count: Uint = 0;
```

Numbers infer sensibly: `$a = 5` is an `Int`, `$b = 1.0` is a `Float64`, `$c = -3` is a signed `Int`
(never an unsigned one — that would lose the sign).

> **Runnable now (Tiny):** [`tests/tutorial/bindings.jolt`](../../tests/tutorial/bindings.jolt).

---

## 3. Functions

```jolt
@double(x: Int) Int -> x * 2 ;;
```

Read it as: `@`name `(`params`)` return-type `->` body `;;`. The body here is a single expression
with **no semicolon** — that means "this is the value the function returns." You could also write
`return x * 2;` explicitly.

A multi-line function:

```jolt
@greet(name: String) None ->
    $msg = "Hi, {name}!";       // string interpolation with { }
    println(msg);
;;                               // last statement ends in ; → returns None (nothing)
```

Notice `"{name}"` — strings interpolate expressions inside `{ }` (use `{{` for a literal brace).

Functions can have **default and named arguments**:

```jolt
@connect(host: String, port: Uint = 8080, tls: Bool = false) None -> ... ;;

connect("example.com");                  // port=8080, tls=false
connect("example.com", tls: true);       // name an argument to skip past defaults
```

> **Runnable now (Tiny):** [`tests/tutorial/double.jolt`](../../tests/tutorial/double.jolt) — `@double`
> and a `main` that calls it (no interpolation or default args yet).

---

## 4. The big idea: ownership and the Custodian

This is what makes Jolt safe without a garbage collector, and it's the part worth slowing down for.

Every value has **one owner**. When you assign it somewhere else, it **moves** — the old name can no
longer use it:

```jolt
$a = Buffer::new();
$b = a;            // the buffer MOVES from a to b
// use(a);         // ERROR (a "custody violation"): a no longer owns the buffer
use(b);            // fine — b owns it now
```

Why? So two names can never both think they own (and free) the same memory. The compiler's
ownership analysis is called **the Custodian**, and a violation is reported as a *custody violation*.

If you actually want two copies, ask for one:

```jolt
$c = copy(big_thing);   // explicit — no accidental expensive copies
```

Small simple types (`Int`, `Bool`, `Char`, …) copy automatically, so everyday code isn't noisy:

```jolt
$n = 5; $m = n;   // both fine — Int copies freely
```

### Borrowing

Moving everywhere would be painful, so you can **borrow** a value temporarily instead of taking
ownership:

```jolt
$$data = Buffer::new();

$view = borrow(data);          // a shared, read-only borrow
read(deref(view));             // deref() reaches through the borrow

$editor = claim(data);         // an exclusive, mutable borrow
write(deref(editor), 42);
```

The one rule the Custodian enforces: **many shared `borrow`s, OR one exclusive `claim`, never both
at once.** That single rule is what prevents data races and dangling references — at compile time,
for free.

> **Coming from Rust?** This is Rust's borrow checker, renamed and tuned to be lighter: lifetimes are
> inferred (you almost never write them), and the vocabulary is `borrow`/`claim`/`deref` under "the
> Custodian." Coming from C? This is the safety C never had, with no runtime cost.

---

## 5. Structs, enums, and methods

```jolt
struct Point ->
    $$x: Float64        // $$ = this field can be changed
    $$y: Float64
;;

// methods live in a `Type::` block; `self` is the receiver
Point::distance_to(self, other: Self) Float64 ->
    sqrt((self.x - other.x) ^ 2 + (self.y - other.y) ^ 2)   // ^ is power
;;

$a = Point { x: 0.0, y: 0.0 };
$b = Point { x: 3.0, y: 4.0 };
println("{a.distance_to(b)}");      // 5.0
```

A method that *changes* the value takes `$$self` instead of `self`:

```jolt
Point::move_by($$self, dx: Float64, dy: Float64) None ->
    self.x = self.x + dx;
    self.y = self.y + dy;
;;
```

**Enums** hold one of several variants, optionally with data:

```jolt
enum Shape ->
    Circle(Float64);
    Rect(Float64, Float64);
;;
```

---

## 6. Control flow (it's all expressions)

`if` and `match` *produce values*, so you can assign their result:

```jolt
$label = if score >= 60 -> "pass" else -> "fail" ;;
```

**`match`** is the workhorse — it destructures and is checked for completeness:

```jolt
@area(s: Shape) Float64 ->
    match s ->
        Circle(r)  -> 3.14159 * r ^ 2;
        Rect(w, h) -> w * h;
    ;;   // the compiler checks you handled every variant
;;
```

Patterns can do a lot — literals, ranges, or-patterns, guards:

```jolt
match code ->
    200 | 201 | 204 -> "ok";          // or-pattern
    400..500        -> "client error"; // range
    n if n >= 500   -> "server error"; // guard
    _               -> "other";         // wildcard
;;
```

Loops:

```jolt
for i in 0..10 -> println(i); ;;     // 0 to 9
for x < 100 -> x = x * 2; ;;          // the "while" form
loop -> if done -> break; ;; step(); ;;   // infinite until break
```

---

## 7. Generics and contracts

A **generic** function works for many types — write the type parameter in `|…|`:

```jolt
@first |T| (xs: Array<T>) T? -> if len(xs) > 0 -> Some(xs[0]) else -> Nothing ;; ;;
```

But what if the function needs the type to *support* something — like comparison? That's what
**contracts** are for. A contract is a set of methods a type promises to provide (think
"interface"/"trait", but Jolt has no inheritance):

```jolt
@@Comparable ->
    @compare(self, other: Self) Int;                       // required method
    @max(self, other: Self) Self ->                        // default method (has a body)
        if self.compare(other) >= 0 -> self else -> other ;;
    ;;
;;
```

A type **adopts** a contract:

```jolt
Point::Comparable ->
    @compare(self, o: Self) Int -> ... ;;
    // max() comes for free from the default
;;
```

Now you can require it as a bound — only types that are `Comparable` may be passed:

```jolt
@largest |T: Comparable| (xs: Array<T>) T? ->
    if len(xs) == 0 -> return Nothing; ;;
    $$best = xs[0];
    for i in 1..len(xs) -> best = best.max(xs[i]); ;;
    Some(best)
;;
```

This is checked at compile time and costs nothing at runtime (the compiler generates a specialized
version per type). When you need a *mixed* collection at runtime, use `dyn`:

```jolt
$$shapes: Array<dyn Drawable> = [];   // different concrete types, one list
for s in shapes -> s.draw(); ;;
```

---

## 8. No null, and errors as values

Jolt has **no null**. A value that might be absent is an `Option`:

```jolt
@find(xs: Array<Int>, target: Int) Int? ->     // Int? = Option<Int>
    for i in 0..len(xs) -> if xs[i] == target -> return Some(i); ;; ;;
    Nothing
;;

$$idx = find(nums, 7) ?? -1;     // ?? supplies a fallback if it's Nothing
```

And **no exceptions**. A function that can fail returns `!T` — "a `T`, or an error":

```jolt
@read_config(path: String) !Config ->
    $f = open(path)?;              // the ? says: if this failed, return the error now
    $text = read_all(borrow(f))?;
    parse(text)                     // returns the Config on success
;;
```

The `?` operator is the magic: on success it unwraps; on failure it returns the error up the call
chain. No try/catch, no hidden control flow — errors are ordinary values you can see in the types.

For cleanup, `defer` runs when the scope ends (even on an error path):

```jolt
@process(path: String) !None ->
    $f = open(path)?;
    defer close(f);        // guaranteed to run, success or failure
    do_work(borrow(f))?;
;;
```

> Two operators, easy to keep straight: **`?`** is for errors (`!T`/`Result`), **`??`** is for
> optionals (`Option`). Mixing them is a friendly compile error.

---

## 9. Doing many things at once

Jolt's concurrency is **safe by the type system** — the Custodian rejects data races at compile
time. The everyday tool is **structured concurrency**: `scope` starts a region, `spawn` launches work
that's guaranteed to finish before the region ends.

```jolt
using Concurrent;

@sum_in_parallel(data: Array<Int>) Int ->
    $$total = Atomic::new(0);
    scope ->
        for chunk in data.chunks(1024) ->
            spawn -> total.add(sum(chunk)); ;;     // each chunk summed concurrently
        ;;
    ;;   // all spawned work has finished here
    total.load()
;;
```

For I/O you just write normal, blocking-looking code — Jolt runs it on **fibers** so it doesn't waste
a thread waiting. No `async`, no `await`, no two-colored functions:

```jolt
using Fiber;

@fetch(url: String) !Bytes ->
    $conn = connect(url)?;     // looks like it blocks; really the fiber steps aside
    conn.read_all()
;;
```

---

## 10. Tell the compiler what code is allowed to do

You can *prove* properties of a function with attributes the compiler enforces:

```jolt
[noalloc]   @hot_path() None -> ... ;;    // compile error if it ever heap-allocates
[pure]      @hash(x: Int) Int -> ... ;;   // no side effects at all
[constanttime] @compare_secret(a, b) Bool -> ... ;;   // no secret-dependent timing (crypto)
```

These are checked all the way down the call chain — a `[noalloc]` function can't call one that
allocates. It's how the same language serves a web server and a microcontroller interrupt handler.

And when you run a program, it gets **no permissions by default** (like Deno):

```
jolt run                              # pure compute only
jolt run --allow-read=./data          # may read these paths
jolt run --allow-net=api.example.com  # may talk to this host
```

A program that tries to do something it wasn't granted gets a clean `PermissionDenied` — not a
surprise at the worst moment.

---

## 11. Testing comes built in

No framework to install. Mark a function `[test]` and run `jolt test`:

```jolt
/// Adds two integers.
/// ```
/// assert_eq!(add(2, 3), 5);   // this example is RUN as a test
/// ```
@add(a: Int, b: Int) Int -> a + b ;;

[test]
@adds_positives() None -> assert_eq!(add(2, 3), 5); ;;
```

Assertions print *why* they failed (the values, a diff), and the doc example above is executed too,
so docs can't go stale. Jolt also has property testing, fuzzing, benchmarking, and even deterministic
simulation testing for concurrent systems — but `[test]` + `assert_eq!` is all you need to start.

---

## 12. A complete little program

Putting it together — a program that finds the closest point to the origin:

```jolt
using Std;

struct Point -> $$x: Float64; $$y: Float64; ;;

Point::dist(self) Float64 -> sqrt(self.x ^ 2 + self.y ^ 2) ;;

@closest(points: Array<Point>) Point? ->
    if len(points) == 0 -> return Nothing; ;;
    $$best = points[0];
    for i in 1..len(points) ->
        if points[i].dist() < best.dist() -> best = points[i]; ;;
    ;;
    Some(best)
;;

[public]
@main() !None ->
    $pts = [
        Point { x: 3.0, y: 4.0 },
        Point { x: 1.0, y: 1.0 },
        Point { x: 5.0, y: 5.0 },
    ];
    match closest(pts) ->
        Some(p) -> println("closest: {p.x}, {p.y}");
        Nothing -> println("no points");
    ;;
;;
```

```
jolt run
# closest: 1, 1
```

---

## Where to go next

- **Cheat sheet** (`jolt-cheatsheet.md`) — everything on one page once the ideas click.
- **Specification** (`jolt-spec-v0.4.md`) — the precise rules behind every feature here.
- **Standard library** (`jolt-stdlib-outline.md`) — what's available to build with.
- Topics this tour skipped: `comptime` (compile-time code), macros, value generics (`Array<T, N>`),
  the low-level layer (raw pointers, MMIO, inline `asm`), and FFI — all in the spec when you're ready.

### The five ideas to remember
1. **Immutable by default** (`$` vs `$$`).
2. **Ownership + the Custodian** — values move; borrow with `borrow`/`claim`; safety with no GC.
3. **Contracts, not inheritance** — `@@` defines capabilities a type can adopt.
4. **No null, no exceptions** — `Option` (`??`) and `!T` errors (`?`).
5. **Tell the compiler your intent** — attributes (`[noalloc]`, `[pure]`, …) and permissions are
   checked, not hoped for.

Welcome to Jolt.
