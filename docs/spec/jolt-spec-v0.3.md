# Jolt — Language Specification (v0.3)

> A general-purpose, statically-typed, low-level systems language with modern memory safety,
> compile-time execution, and a large toolchain. This spec folds in every decision from the
> design process (see `jolt-decisions.md`) into one internally-consistent document. Examples are
> tutorial-style and assume no prior Jolt knowledge.
>
> **Reading note.** This version resolves every previously-open detail; there are no residual
> markers. The decisions log (`jolt-decisions.md`) records the rationale for each choice.

---

## 1. Hello, Jolt

```jolt
using Std;

[public]
@main() !None ->
    println("Hello, Jolt!");
    return None;
;;
```

Things to notice already:
- `using Std;` pulls in the standard prelude.
- `@` introduces a function. `@main` is the entry point.
- `[public]` is an attribute making `main` visible outside its library.
- Blocks open with `->` and close with `;;`. Statements end with `;`.
- `!None` is the return type: "returns `None`, or fails with an error" (see §12).
- Comments use `//`.

---

## 2. Comments

```jolt
// single-line comment

/// multi-line and documentation comment.
/// The doc generator collects these.
/// Spans consecutive lines.

@add(a: Int, b: Int) Int -> a + b ;;   // trailing comment
```

`//` is an ordinary comment. `///` is a doc comment consumed by the documentation generator.
There is no other comment form — `#` is reserved for macros (§17), and `/* */` is **not** used.

---

## 3. Lexical structure & naming

Jolt enforces naming conventions at compile time:

| Kind | Convention | Example |
| ---- | ---------- | ------- |
| Variables, functions, methods | `snake_case` (enforced) | `total_count`, `read_file` |
| Types, contracts, enums | `PascalCase` | `Buffer`, `Comparable` |
| Compile-time constants (`[const]`) | `SCREAMING_SNAKE_CASE` (linted) | `MAX_SIZE` |
| Packages, libraries | `PascalCase` | `Utils`, `Math` |

Identifiers may use Unicode (UTF-8):

```jolt
$δ = 0.00001;
$名前 = "Aoi";
```

Terminators and blocks:
- `;` ends a statement.
- `->` opens a block; `;;` closes it.
- `_` is the discard binding (ignore a value).

---

## 4. Variables & bindings

Jolt is **immutable by default**. The sigil marks a *new* binding; bare names reassign.

```jolt
$x = 10;          // new immutable binding (Int inferred)
// x = 11;        // COMPILE ERROR: x is immutable

$$y = 10;         // new MUTABLE binding ($$)
y = 11;           // ok

[const] $z = 64;  // compile-time constant (see §11 attributes)
```

Explicit types use a **colon**:

```jolt
$name: String = "Aoi";
$$count: Uint = 0;
```

Flexible (re-typeable) bindings use `?`, and only make sense on mutable bindings:

```jolt
$$slot? = 9.0;    // mutable, type may change later
slot = "now text"; // ok because of ?
```

Destructuring, with `_` to discard:

```jolt
$a, $b, _ = 1, 2, 3;   // a = 1, b = 2, third value ignored
```

> **Inference note.** A negative literal infers to a signed `Int`, never `Uint`:
> `$n = -3;  // Int`. A bare decimal infers to `Float64`: `$f = 1.0; // Float64`.

---

## 5. Types

### 5.1 Integers

| Type | Signed | Bits | Min | Max | Alias of |
| ---- | ------ | ---- | --- | --- | -------- |
| Int | ✓ | architecture | — | — | |
| Uint | | architecture | 0 | — | |
| Int8 | ✓ | 8 | -2^7 | 2^7-1 | |
| Uint8 | | 8 | 0 | 2^8-1 | |
| Int16 | ✓ | 16 | -2^15 | 2^15-1 | |
| Uint16 | | 16 | 0 | 2^16-1 | |
| Int32 | ✓ | 32 | -2^31 | 2^31-1 | |
| Uint32 | | 32 | 0 | 2^32-1 | |
| Int64 | ✓ | 64 | -2^63 | 2^63-1 | |
| Uint64 | | 64 | 0 | 2^64-1 | |
| Int128 | ✓ | 128 | -2^127 | 2^127-1 | |
| Uint128 | | 128 | 0 | 2^128-1 | |
| Byte | | 8 | 0 | 2^8-1 | Uint8 |
| Short | ✓ | 16 | -2^15 | 2^15-1 | Int16 |
| Long | ✓ | 64 | -2^63 | 2^63-1 | Int64 |
| Bool | N/A | 8 | false (0) | true (1) | |

### 5.2 Floating point

| Type | Precision | Bits |
| ---- | --------- | ---- |
| Float16 | half | 16 |
| Float32 | single | 32 |
| Float64 | double | 64 |
| Float128 | quadruple | 128 |
| Double | double | 64 | (alias of Float64) |

### 5.3 Text

- `Char` — a 32-bit Unicode scalar value (`'A'`, `'λ'`, `'🜨'`).
- `String` — UTF-8 encoded bytes. Multiline strings use backticks:

```jolt
$c: Char = 'λ';
$greeting = "hello";
$block = `line one
line two`;
```

### 5.4 Composite & built-in

`Raw` (binary blob), `Array`, `Set`, `Map`, `Tuple`, `Pair`, `Symbol`, `Enum`, `Struct`, `Union`,
`Error`, `None`. `Empty` denotes an empty collection only (it is **not** a null value — Jolt has
no null; absence is modeled with `Option`, §10).

```jolt
$nums: Array<Int> = [1, 2, 3];
$pair: Pair<String, Int> = ("age", 30);
$lookup: Map<String, Int> = {"a": 1, "b": 2};
```

`Complex` and `Rational` are **standard-library generics**, not primitives:

```jolt
using Math;
$z: Complex<Float64> = Complex::new(1.0, -2.0);
$q: Rational<Int64> = Rational::new(3, 4);
```

### 5.5 Memory types

- `Ref<T>` — a safe, borrow-checked reference (§9).
- `Pointer<T>` — a raw pointer, usable only inside `[unsafe]` (§9.6).

---

## 6. Operators

### 6.1 Precedence (high → low)

```
postfix     expr++  expr--  expr[]  expr?[]  expr.  expr?.  @expr  $expr  $$expr
prefix      -expr   !expr   ++expr  --expr
multiplicative  *  /  %  ^         // ^ is POWER
additive    +  -
shift       <<  >>  <<<  >>>  <<|  >>|
bitwise-and &
bitwise-not ~
bitwise-or  |
bitwise-xor %|
bitwise-nand ~&
bitwise-nor ~|
bitwise-xnor ~%|
relational  >=  >  <=  <  as
equality    ==  !=
logical-and &&   and
logical-or  ||   or
negation    not
ternary     c ? a : b
assignment  =  +=  -=  *=  /=  ^=  %=  //=  &=  |=  %|=  ~&=  ~|=  ~%|=  >>=  <<=  >>|=  <<|=  <<<=  >>>=
range       ..
spread      ...
```

> ⚠️ **`^` is exponentiation, not XOR.** XOR is `%|`. C/Python users: re-map your muscle memory.
> There is **no** `**` operator.

### 6.2 Arithmetic

| Op | Example | Meaning |
| -- | ------- | ------- |
| `+` | `x + y` | add |
| `-` | `x - y` | subtract |
| `-x` | `-x` | unary minus |
| `*` | `x * y` | multiply |
| `/` | `x / y` | divide |
| `//` | `x // y` | floor division |
| `%` | `x % y` | modulo |
| `^` | `x ^ y` | power |

```jolt
$a = 7 // 2;     // 3  (floor division)
$b = 2 ^ 10;     // 1024 (power)
$c = 7 % 3;      // 1
```

Increment/decrement: `++x` / `--x` (prefix), `x++` / `x--` (postfix).

### 6.3 Bitwise (symbols only)

```jolt
$m = a & b;      // and
$n = a | b;      // or
$o = a %| b;     // xor
$p = ~a;         // not
$q = a ~& b;     // nand
$r = a ~| b;     // nor
$s = a ~%| b;    // xnor
```

Shifts:

| Op | Meaning |
| -- | ------- |
| `<<` | arithmetic shift left |
| `>>` | arithmetic shift right |
| `<<<` | logical shift left |
| `>>>` | logical shift right |
| `<<\|` | rotate left |
| `>>\|` | rotate right |

### 6.4 Unicode math aliases

`÷` ≡ `/`, `√x` ≡ `sqrt(x)`, `≠` ≡ `!=`, `≈` ≡ approx-equal helper, `≡` ≡ identity helper. These are
pure conveniences that lower to the ASCII forms / stdlib calls.

```jolt
$d = 10 ÷ 2;     // same as 10 / 2
$root = √16.0;   // same as sqrt(16.0)
```

---

## 7. Functions

```jolt
// @name |generics| (params) ReturnType -> body ;;

@double(x: Int) Int -> x * 2 ;;

@greet(name: String) None ->
    println("Hi, " + name);
    return None;
;;
```

Generic functions put type parameters in `|…|`, with contract bounds after a colon:

```jolt
@max |T: Comparable| (a: T, b: T) T ->
    if a.compare(b) >= 0 -> return a; ;;
    return b;
;;
```

### 7.1 Multiple dispatch with guards

A function name can be defined multiple times with `{guard}` clauses. Guards are evaluated **at
runtime** and may read argument values. Two fully-identical signatures are a **compile error**; if
two different guards are both true for a call, that's a **runtime error** (ambiguous dispatch).

```jolt
@convert(m: Float64, {unit == "inch"}) Float64 -> m / 2.54 ;;
@convert(m: Float64, {unit != "inch"}) Float64 -> m * 2.54 ;;
```

> **Toolchain note.** Because guards are runtime, an ambiguous call only fails when that input
> occurs. The compiler emits a *warning* when it can statically prove two guards overlap, to surface
> the problem earlier.

### 7.2 Closures

Anonymous functions reuse the `|…|` bracket:

```jolt
$inc = |x: Int| Int -> x + 1 ;;
$doubled = map(nums, |x| -> x * 2 ;;);   // types inferred where possible
```

---

## 8. Control flow

### 8.1 if / else

```jolt
$color = "blue";

if color == "blue" ->
    println("cool");
else ->
    println("warm");
;;
```

Condition-list form (each arm is `condition: body`):

```jolt
if ->
    color == "blue":  println("blue");
    color == "green": println("green");
    default:          println("other");
;;
```

### 8.2 switch vs match

Jolt has **both**, with a clear division of labor:
- `switch` compares a value against constant cases.
- `match` does pattern matching with destructuring and (for closed types) exhaustiveness.

```jolt
// switch — constant comparison
switch status ->
    case 200: println("ok");
    case 404: println("not found");
    default:  println("other");
;;

// match — patterns + destructuring
enum Shape -> Circle(Float64); Rect(Float64, Float64); ;;

@area(s: Shape) Float64 ->
    match s ->
        Circle(r)  -> return 3.14159 * r ^ 2;
        Rect(w, h) -> return w * h;
    ;;   // exhaustive: every variant handled, no default needed
;;
```

### 8.3 Loops

Two forms. `loop` is infinite (exit with `break`); `for` iterates, and `for <cond>` doubles as a
while-loop.

```jolt
loop ->
    if done -> break; ;;
    step();
;;

for i in 0..10 ->        // 0 to 9
    println(i);
;;

for x < 100 ->           // "while" form
    x = x * 2;
;;
```

`break` exits a loop; `next` skips to the next iteration; `return` exits the function.

---

## 9. Memory & ownership

Jolt is **move-by-default with compile-time borrow checking** — Rust's safety model, tuned to be
lighter. There is **no implicit copy** and **no null**.

### 9.1 Moves

```jolt
$a = Buffer::new();
$b = a;            // a is MOVED into b
// use(a);         // COMPILE ERROR: a was moved
use(b);            // fine
```

### 9.2 Copy types

A type is copied implicitly **iff it conforms to the `Copy` contract**. Primitives are `Copy`;
everything else moves unless you copy explicitly.

```jolt
$x = 5; $y = x;    // Int is Copy → x still usable
$big = Buffer::new();
$dup = copy(big);  // explicit copy for non-Copy types
```

### 9.3 Borrows

There are two borrow forms: `ref(x)` is a **shared** (read-only) borrow of type `Ref<T>`;
`ref_mut(x)` is a **mutable** borrow of type `RefMut<T>` and requires `x` to be a mutable (`$$`)
binding. `unref()` dereferences either (it does **not** free). The borrow checker enforces
**shared XOR mutable**: any number of `ref`s, or exactly one `ref_mut`, never both at once.

```jolt
$$data = Buffer::new();

$view = ref(data);         // shared borrow: Ref<Buffer>
$view2 = ref(data);        // another shared borrow — fine
read(unref(view));

// A mutable borrow cannot coexist with shared borrows:
$editor = ref_mut(data);   // mutable borrow: RefMut<Buffer>
write(unref(editor), 42);
```

Borrows are **non-lexical** — a borrow ends at its last use, not at the end of the scope, which
makes common patterns just work without reordering code.

### 9.4 Lifetimes

Lifetimes are **inferred** — there is no annotation syntax in ordinary code. For the rare case the
compiler can't disambiguate (e.g. a function returning one of several input references), declare a
lifetime in the generic bracket with `life` and attach it as a **second type parameter** on the
reference type. No new sigil is introduced — it reuses generics:

```jolt
// returns a reference tied to the same borrow as `haystack`
@first |life L| (haystack: Ref<Array<Int>, L>, needle: Int) Ref<Int, L> -> ... ;;
```

In the common case you write none of this; `@first(xs, n) Ref<Int>` infers `L` automatically.

### 9.5 Shared ownership

When single ownership is too strict, opt in with the `[shared]` attribute, which gives counted
shared ownership (the runtime-counted escape hatch from the borrow-checked default).

```jolt
[shared] $tree = Node::new();
$also_tree = tree;   // shared: both keep access; freed when the last owner drops
```

Thread-safety is explicit: plain `[shared]` uses a **non-atomic** count (single-threaded, cheaper),
while `[shared, sync]` uses an **atomic** count safe to share across threads. Reference cycles are
**not** automatically collected — break them with a `[weak]` reference, which does not keep its
target alive:

```jolt
[shared] $parent = Node::new();
[shared] $child  = Node::new();
child.parent = weak(parent);   // [weak] ref breaks the parent⇄child cycle
```

### 9.6 Raw pointers & unsafe

`ptr()` and `Pointer<T>` exist only inside `[unsafe]` blocks/functions.

```jolt
[unsafe]
@poke() None ->
    $p: Pointer<Int> = ptr(some_int);
    // pointer arithmetic and raw deref allowed here
;;
```

`[unsafe]` unlocks exactly five powers, and **nothing else**:
1. creating and dereferencing raw `Pointer<T>` via `ptr()`;
2. pointer arithmetic;
3. calling other `[unsafe]` functions;
4. reading/writing `union` fields (§14);
5. inline `asm` (§19).

Crucially, `[unsafe]` does **not** switch off the borrow checker — safe `Ref`/`RefMut` borrows are
still fully checked inside an unsafe block. Unsafe widens what you *can* express; it does not weaken
the guarantees on safe references.

### 9.7 Allocators

There is an **implicit default allocator**, plus explicit allocators selected with the `[alloc: …]`
attribute. Containers remember the allocator that created them, so cleanup is correct.

```jolt
// uses the default allocator
@build_default() !Tree -> ... ;;

// uses an explicit allocator for everything allocated in this function
[alloc: arena]
@build_in_arena() !Tree -> ... ;;
```

The process-wide default is swapped by putting `[alloc: …]` on `@main`:

```jolt
[alloc: tracking_allocator]
@main() !None -> ... ;;     // every default allocation now goes through tracking_allocator
```

`defer` and `errdefer` (see §12.4) handle deterministic cleanup.

---

## 10. Option (absence, no null)

`T?` is sugar for `Option<T>` — either `Some(value)` or `None`. There is no null pointer.

```jolt
@find(xs: Array<Int>, target: Int) Int? ->
    for i in 0..len(xs) ->
        if xs[i] == target -> return Some(i); ;;
    ;;
    return None;
;;

// ?? unwraps an Option with a fallback (coalesce)
$$idx = find(nums, 7) ?? -1;

// pattern-match an Option
match find(nums, 7) ->
    Some(i) -> println("found at " + to_string(i));
    None    -> println("absent");
;;
```

`??` is the **Option** operator. Do not confuse it with `?` (error propagation, §12) — mixing them
is a compile error with a hint.

---

## 11. Attributes

Attributes appear in `[...]` before a declaration. Multiple attributes combine; some take fields.

```jolt
[public]
@api_fn() None -> ... ;;

[const] $MAX = 1024;

[{deprecated: "use parse_v2", since: "0.3"}, unsafe]
@old_parse() None -> ... ;;

[heap]
@big_alloc() None -> ... ;;
```

Built-in attributes seen so far: `public` (§13), `const` (§4), `unsafe` (§9.6), `shared` / `weak` /
`sync` (§9.5), `heap`, `alloc: …` (§9.7), and `deprecated{...}`. **User-defined attributes are
allowed**: a user attribute is a compile-time macro (§17/§18) that receives the annotated
declaration and may inspect or rewrite it. Built-in attributes are simply the ones the compiler
ships with.

---

## 12. Error handling

Errors are values. Jolt has **no exceptions** — control flow stays visible.

### 12.1 Two spellings, one model

- `!T` — "returns `T`, or fails." The error type is an **inferred, open error set** (Zig-style).
- `Result<T, E>` — explicit error type `E`. Use when you want a **closed**, exhaustively-matchable
  set. `!T` is sugar for `Result<T, E>` with an inferred open `E`, so both work with `?`.

```jolt
// open error set, inferred
@read_config(path: String) !Config ->
    $f = open(path)?;       // ? propagates the error, unioning it into this fn's set
    $text = read_all(f)?;
    return parse(text);
;;

// explicit, closed error set → exhaustive match possible
enum LoadError -> NotFound; Corrupt; ;;

@load(path: String) Result<Data, LoadError> ->
    match check(path) ->
        Ok(d)            -> return Ok(d);
        Err(NotFound)    -> return Err(NotFound);
        Err(Corrupt)     -> return Err(Corrupt);
    ;;   // exhaustive — all LoadError variants handled
;;
```

### 12.2 The `?` operator

`?` propagates errors: on `Ok`/success it unwraps; on error it returns early, merging the error into
the caller's set. It works on both `!T` and `Result<T, E>`. It is **error-only** — using `?` on an
`Option` is a compile error (use `??`).

### 12.3 Declaring errors

`error` declares an error value; `Error` is the root contract every error conforms to. An "open
error set" is conceptually a set of `Error`s.

```jolt
error NotFound;
error Timeout;
```

### 12.4 Cleanup: defer / errdefer

`defer` runs on scope exit (LIFO). `errdefer` runs **only** when the scope exits via an error.

```jolt
@process(path: String) !None ->
    $f = open(path)?;
    defer close(f);            // always runs at scope end
    $buf = alloc(1024)?;
    errdefer free(buf);        // runs only if a later step errors
    fill(buf)?;
    return None;
;;
```

---

## 13. Visibility

**Private by default.** Expose a declaration with `[public]`.

```jolt
@helper() None -> ... ;;        // private to its library

[public]
@api() None -> ... ;;           // visible to importers
```

---

## 14. Structs, enums, unions

```jolt
struct Person ->
    $name: String
    $$age: Uint        // mutable field
;;

enum Shape ->
    Circle(Float64);
    Rect(Float64, Float64);
;;

union Word ->
    as_int: Uint32
    as_bytes: Array<Byte>
;;
```

Methods are defined in a separate `Type::method` block. The receiver is an explicit first
parameter: `self` for a shared (read-only) receiver, `$$self` for a mutable one (which requires a
mutable borrow of the value at the call site).

```jolt
Person::greet(self) None ->          // shared receiver: reads only
    println("Hi, I'm " + self.name);
;;

Person::birthday($$self) None ->     // mutable receiver: may mutate fields
    self.age = self.age + 1;
;;
```

---

## 15. Contracts (the trait/interface system)

Contracts are Jolt's single abstraction mechanism — they serve as both interfaces and generic
bounds. They are **trait-style**: no inheritance, no runtime/subtype polymorphism, **no instance
data** — only required methods and optional default implementations. They are used as **static**
bounds and monomorphized (zero-cost), so there is no diamond problem.

```jolt
@@Comparable ->
    @compare(self, other: Self) Int;       // required method (no body)
    @max(self, other: Self) Self ->        // default method (has a body)
        if self.compare(other) >= 0 -> return self; ;;
        return other;
    ;;
;;
```

A type adopts a contract with a `Type::Contract` block:

```jolt
Person::Comparable ->
    @compare(self, other: Self) Int -> self.age - other.age ;;
    // max() inherited from the default
;;
```

Use as a generic bound:

```jolt
@largest |T: Comparable| (items: Array<T>) T? ->
    if len(items) == 0 -> return None; ;;
    $$best = items[0];
    for i in 1..len(items) ->
        best = best.max(items[i]);
    ;;
    return Some(best);
;;
```

The built-in `Copy` contract (§9.2) is an ordinary contract a type can conform to.

---

## 16. Modules, packages & imports

Import syntax is **path-based and reads left→right**. Three forms plus `export`:

```jolt
using Math;                  // bring in a whole package
import Utils.Print;          // import a qualified path
from Utils import Print;     // selective import
from Utils import Print as P;

export { Add as add, Sub as sub };
```

Visibility of exported items is governed by `[public]` (§13); there is no separate api-file split.

Jolt surfaces three organizational tiers:
- **`library`** — the unit of *reuse and import*. A folder of source files sharing a namespace.
- **`package`** — the unit of *distribution* (what you publish/depend on); contains one or more
  libraries.
- **`program`** — a compilation unit containing `@main`; compiles to an executable. (This replaces
  the old draft's loose `program` keyword.)

```jolt
package MyApp;            // distribution unit
library MyApp.Core;       // a library within it

program ->               // an executable unit
    using MyApp.Core;
    [public] @main() !None -> ... ;;
;;
```

---

## 17. Macros

Macros are `#`-prefixed (the `#` sigil is free because comments use `//`). Two tiers:

1. **Declarative pattern macros** — hygienic, for common substitutions.
2. **Procedural / `comptime` macros** — run code at compile time to emit code (same machinery as
   `comptime`, §18). User-defined attributes (§11) are procedural macros that receive a declaration.

A declarative macro is defined with `#macro` and matches a pattern to an expansion:

```jolt
// definition: pattern on the left of ->, expansion on the right
#macro swap(a, b) ->
    $tmp = a; a = b; b = $tmp;
;;

// invocation
#swap(x, y);
```

Macros are **hygienic**: identifiers a macro introduces (like `$tmp` above) cannot capture or
collide with names at the call site, and names from the call site resolve in the caller's scope.

```jolt
$r = #ref(person);        // declarative macro invocation
#(ref, person);           // alternate call form
```

---

## 18. Compile-time execution (`comptime`)

`comptime` evaluates values and arguments at compile time (Zig-style). It also backs procedural
macros (§17). Note: because contracts + `|T|` generics already provide static polymorphism,
`comptime` is for **value-level** compile-time work and codegen — it is *not* the generics
mechanism.

```jolt
comptime ->
    $$table = build_lookup_table();   // computed at compile time
;;

@pow_of_two(comptime n: Uint) Uint -> 1 << n ;;   // n must be known at compile time
```

---

## 19. Inline assembly & C

`asm` and `c` are plain keywords using arrow blocks (no `#` prefix).

```jolt
asm ->
    movq $60, %rax     // exit syscall (Linux)
    movq $2,  %rdi
    syscall
;;

c ->
    /* C source compiled and linked in */
;;
```

---

## 20. Built-in / prelude functions

`len`, `log`, `typeinfo`, `help`, `print`, `println` are ordinary standard-library functions in the
auto-imported prelude — **not** keywords — so they can be shadowed and are resolved through the
module system.

```jolt
$n = len(nums);
typeinfo(nums);          // reflection
println("done");
```

---

## 21. Keywords (reserved)

```
// packages
using  import  export  from  as  package  library  program  module  macro

// control flow
if  else  for  loop  match  switch  case  then  return  next  break  default

// memory & safety
ref  ref_mut  unref  ptr  weak  life  defer  errdefer  comptime  asm  c

// types & decls
struct  enum  union  error  type  in  implicit  empty

// literals
true  false  None  Some  Ok  Err  Self  self
```

The orphaned keywords from earlier drafts — `when`, `do`, `while`, `repeat`, `until`, `imply`,
`infer` — are **removed**: loops are `loop` / `for` (with `for <cond>` covering the while case), and
multi-branch dispatch is `match` / `switch`.

---

## Appendix A — A complete small program

```jolt
using Std;

/// A point in 2D space.
struct Point ->
    $$x: Float64
    $$y: Float64
;;

Point::distance_to(self, other: Self) Float64 ->
    $dx = self.x - other.x;
    $dy = self.y - other.y;
    return √(dx ^ 2 + dy ^ 2);
;;

@@Named ->
    @name(self) String;                 // required
    @describe(self) String ->           // default
        return "I am " + self.name();
    ;;
;;

Point::Named ->
    @name(self) String -> "a point" ;;
;;

[public]
@main() !None ->
    $a = Point { x: 0.0, y: 0.0 };
    $b = Point { x: 3.0, y: 4.0 };
    println("distance = " + to_string(a.distance_to(b)));   // 5.0
    println(a.describe());

    $$found = find_first([1, 2, 3, 4], |n| -> n % 2 == 0 ;;) ?? -1;
    println("first even = " + to_string(found));
    return None;
;;
```

---

*End of v0.3. All earlier residual items are now resolved; the decisions log records the rationale.*
