# Jolt — Language Specification (v0.2, consolidated draft)

> **Status:** This document merges the three source drafts + the inline notes into one
> internally-consistent base. Where the drafts disagreed, I picked a default and marked it
> `[DECISION]` so you can confirm or override. Net-new proposals for missing pieces are
> marked `[PROPOSAL]`. Things that need your input are marked `[OPEN]`.

Jolt is a general-purpose, statically-typed, low-level systems language with modern,
Rust-style memory safety and a large toolchain.

---

## 0. Design pillars (assumed — confirm)

- Statically typed, type inference where unambiguous.
- Low-level / systems target (native, no GC by default).
- Memory safety via ownership + references, with explicit `unsafe` escape hatch.
- Compile-time execution (`comptime`).
- Inline `asm` and `c` interop.
- AOT compiled, native binaries, desktop + mobile.

---

## 1. Comments

`[DECISION]` Drafts split between C-style `//` (doc 1) and `#`-style (everything newer).
**Canonical = `#`-style.** `//` is dropped.

```jolt
# single-line comment

#> enclosed single-line comment <#

#>
    multi-line comment
<#

#? single-line doc comment ?#

#?>
    multi-line doc comment
<?#
```

Doc comments feed the documentation generator.

> ⚠️ **Conflict to resolve:** `#` now starts a comment, but draft 1 also used `#` for
> macros (`#ref(any)`) and `#asm` / `#c`. Those collide. See §13 (Macros) for the proposed fix.

---

## 2. Lexical & naming rules

- Variables: **snake_case**, enforced by the compiler. Names carry no semantic meaning.
- `[PROPOSAL]` Types: **PascalCase**. Functions/methods: **snake_case**. Constants (`$$`): **SCREAMING_SNAKE_CASE** or snake_case — `[OPEN]`.
- Unicode (UTF-8) identifiers allowed (`$δ = 0.00001`, `$안녕하세요 = "Hello"`).
- `_` is the discard binding (ignore a value).

Statement terminators:
- `;`  — end of a single statement / line.
- `;;` — end of a logical block (closes a `->` block).
- Block opener is `->`.

---

## 3. Variables & bindings

`[DECISION]` Draft 1 used `:=` for declaration vs `=` for assignment, *and* a `$` sigil.
That's redundant — the `$` sigil already marks "new binding." Adopting draft 3's cleaner model
and **dropping `:=`**:

```jolt
$x = 10                 # new mutable binding, type inferred (Int)
$x: String = "hi"       # new binding, explicit type   [DECISION] colon, not space
x = 20                  # reassignment (no sigil)
$$PI = 3.14159          # constant binding (cannot reassign)
$y? = 9.0               # "flex" binding: may be reassigned to a different type later
y  = "now a string"     # legal only because y was declared with ?
```

`[DECISION]` Type annotation uses a **colon** (`$x: Int`), not space-separation
(`$x Int`). Draft 1 used space; docs 2/3 used colon; colon wins for being unambiguous next
to expressions.

Destructuring:

```jolt
$x, $y, _ = 1, 2, 3     # [DECISION] each new name takes its own $; _ discards
```

> 🐛 **Bug in draft 3 to fix:** the example `$y = -3  →  -1 (Uint)` is wrong twice over:
> a negative literal can't infer to `Uint`, and the printed value `-1` doesn't match `-3`.
> Negative literals should infer to `Int`.

---

## 4. Types

### 4.1 Integers

`[DECISION]` Including the 128-bit types from draft 3. `Byte`/`Short`/`Long` are aliases.

| Type     | Signed | Bits         | Min     | Max      | Alias of |
| -------- | ------ | ------------ | ------- | -------- | -------- |
| Int      | ✓      | architecture | —       | —        |          |
| Uint     |        | architecture | 0       | —        |          |
| Int8     | ✓      | 8            | -2^7    | 2^7-1    |          |
| Uint8    |        | 8            | 0       | 2^8-1    |          |
| Int16    | ✓      | 16           | -2^15   | 2^15-1   |          |
| Uint16   |        | 16           | 0       | 2^16-1   |          |
| Int32    | ✓      | 32           | -2^31   | 2^31-1   |          |
| Uint32   |        | 32           | 0       | 2^32-1   |          |
| Int64    | ✓      | 64           | -2^63   | 2^63-1   |          |
| Uint64   |        | 64           | 0       | 2^64-1   |          |
| Int128   | ✓      | 128          | -2^127  | 2^127-1  |          |
| Uint128  |        | 128          | 0       | 2^128-1  |          |
| Byte     |        | 8            | 0       | 2^8-1    | Uint8    |
| Short    | ✓      | 16           | -2^15   | 2^15-1   | Int16    |
| Long     | ✓      | 64           | -2^63   | 2^63-1   | Int64    |
| Bool     | N/A    | 8            | false(0)| true(1)  |          |

> 🐛 **Fix:** draft 1/2 gave `Byte` the signed range `-2^7..2^7-1`, which is `Int8`, not a byte.
> A `Byte` is conventionally unsigned `0..255`. Corrected above — confirm.

`[OPEN]` `Complex`, `Rational` — sizes/semantics undefined in every draft. Are these core, or
stdlib types? If core, define their bit layout.

### 4.2 Floating point

| Type             | Precision | Bits |
| ---------------- | --------- | ---- |
| Float16          | half      | 16   |
| Float32          | single    | 32   |
| Float64          | double    | 64   |
| Float128         | quadruple | 128  |
| Double           | double    | 64   | (alias of Float64)

### 4.3 Char & String

| Type   | Bits | Notes |
| ------ | ---- | ----- |
| Char   | 16   | single character `[OPEN]` UTF-16 unit? Or a full Unicode scalar (would need 32)? |
| String | —    | `"..."`; multiline with backticks `` `...` `` |

> `[OPEN]` `Char` is 16-bit in the tables but you allow UTF-8 identifiers and want low-level
> control. 16-bit Char can't hold every Unicode scalar (emoji etc.). Recommend either making
> `Char` a 32-bit Unicode scalar value, or being explicit that `String` is UTF-8 bytes and
> `Char` is a grapheme/scalar accessed separately.

### 4.4 Other / built-in

`Raw` (binary data), `Array`, `Set`, `Map`, `Tuple`, `Pair`, `Symbol`, `Enum`, `Struct`,
`Union`, `Error`, `None`, `Empty`.

`[OPEN]` `None` vs `Empty` vs `?` (option): you have three "absence" concepts. Pick a model:
- `None` = the null-ish value, `Empty` = empty collection, `?` = optional type wrapper? Or
- collapse into one. Right now their relationship is undefined.

### 4.5 Memory types

- `Ref<T>` — a safe reference (borrow).
- `Pointer<T>` — a raw pointer (likely `unsafe`).

`[OPEN]` **This is the biggest undefined area** for a "Rust-like" language. See §9.

---

## 5. Operators

### 5.1 Precedence (high → low)

```
postfix   expr++  expr--  expr[]  expr?[]  expr.  expr?.  @expr  $expr  $$expr
prefix    -expr   !expr   ++expr  --expr
mul       *  /  %  ^                      # ^ is POWER, not xor
add       +  -
shift     <<  >>  <<<  >>>  <<|  >>|
bit-and   &   AND
bit-not   ~   NOT
bit-or    |   OR
bit-xor   %|  XOR
bit-nand  ~&  NAND
bit-nor   ~|  NOR
bit-xnor  ~%| XNOR
relational >=  >  <=  <  as
equality  ==  !=
log-and   &&  and
log-or    ||  or
negation  not
ternary   c ? a : b
assign    =  +=  -=  *=  /=  ^=  %=  //=  &=  |=  %|=  ~&=  ~|=  ~%|=  >>=  <<=  >>|=  <<|=  <<<=  >>>=
range     ..        # sequence
spread    ...
```

> ⚠️ `^` means **power** here (not XOR as in C). XOR is `%|`. Worth a prominent note in user docs —
> this trips up everyone coming from C.

> `[OPEN]` Precedence above is a *guess* at ordering. In particular: should bitwise `&`/`|` really
> bind tighter than comparisons (C's famous footgun)? Recommend putting them looser than shift but
> the relative order vs comparison should be deliberate.

### 5.2 Shift family (clarified from draft 3)

| Op    | Meaning                |
| ----- | ---------------------- |
| `<<`  | arithmetic shift left  |
| `>>`  | arithmetic shift right |
| `<<<` | logical shift left     |
| `>>>` | logical shift right    |
| `<<|` | rotate left            |
| `>>|` | rotate right           |

### 5.3 Arithmetic

| Op    | Example | Meaning      |
| ----- | ------- | ------------ |
| `+`   | `x + y` | add          |
| `-`   | `x - y` | subtract     |
| `-x`  | `-x`    | unary minus  |
| `*`   | `x * y` | multiply     |
| `/`   | `x / y` | divide       |
| `%`   | `x % y` | modulo       |
| `^`   | `x ^ y` | power        |
| `//`  | `x // y`| `[OPEN]` floor-div? listed "NEED TO BE IMPLEMENTED" |
| `**`  | `x ** y`| `[OPEN]` listed "NEED TO BE IMPLEMENTED" — but `^` is already power. Redundant? |

`++x`/`--x` (prefix), `x++`/`x--` (postfix).

### 5.4 Math symbol literals (from draft 3)

`÷` (`\div`), `√` (`\sqrt`), `≠` (`\ineq`), `≈` (`\aproxeq`), `≡` (`\identity`).
`[OPEN]` Are these surface syntax for existing operators, or distinct operators? e.g. is `÷` just `/`?

---

## 6. Functions

```jolt
# @name |GenericParams| (params) ReturnType -> body ;;

@convert |A| (m: A, inch: Int) Int ->
    m / 2.54;
;;
```

`[DECISION]` Normalized to colon-annotated params (`m: A`) for consistency with §3, and the
generic list uses `|A|` (draft 1).

Argument guards / overloading (draft 1's `{type != "inch"}`):

```jolt
@convert (m: Int, value: Int, {type != "inch"}) Int ->
    if type == "inch" -> return m / 2.54;
    else -> return m * 2.54;
    ;;
;;
```

`[OPEN]` What is `{type != "inch"}` exactly — a *guard* selecting between overloads, a
*where/constraint* clause, or a *default-argument predicate*? It's the only example and its
meaning is undefined. This matters a lot for the type system.

> ⚠️ **Block-syntax conflict:** functions use `-> ... ;;`, but the Attributes section writes
> `@main() { ... }` with braces. **Pick one.** This spec assumes `-> ... ;;` everywhere; rewrite
> the attribute examples accordingly (see §11). `[DECISION]`

`[PROPOSAL]` Still undefined: closures / lambdas / function values. Suggested:

```jolt
$f = |x: Int| Int -> x * 2 ;;     # anonymous function value
map(xs, |x| -> x + 1 ;;)
```

---

## 7. Control flow

### 7.1 if / else (single condition)

```jolt
if color == "blue" ->
    debug::log(color);
else ->
    debug::log("Color", color);
;;
```

> 🐛 Draft 1 used a `default` clause inside `if`, which conflates `if` with `match`.
> Normalized to `else` above. `default` should only exist in `match`.

### 7.2 if (condition list — replaces "elif")

```jolt
if ->
    color == "blue":  D::log("Blue");
    color == "green": D::log("Green");
    default:          D::log("Color", color);
;;
```

This is a cond/guard form. `[OPEN]` It overlaps heavily with `match` (§7.3). Do you want both?
See "redundancy" note at the end.

### 7.3 match

```jolt
match color ->
    "green"  -> Debug::log("green");
    "red"    -> Debug::log("red");
    default  -> Debug::log("other");
;;
```

`[PROPOSAL]` For a safe systems language, `match` should support destructuring + exhaustiveness
checking on enums:

```jolt
match shape ->
    Circle(r)       -> area = 3.14 * r ^ 2;
    Rect(w, h)      -> area = w * h;
;;   # compiler errors if a variant is unhandled and no default
```

### 7.4 Loops

You currently have **`while`, `for`, `loop`, `when`, `do`, `repeat`/`until`** — six forms.
`[OPEN]` Recommend consolidating to three:
- `loop -> ... ;;` (infinite, exit via `break`)
- `while cond -> ... ;;`
- `for x in iterable -> ... ;;`

and folding `repeat/until` into `do { } while`-style if you want a post-condition loop.
What are `when` and `do` meant to do that the others don't?

`break` / `next` (continue) / `return` as expected.

---

## 8. Structs, Enums, Unions, Classes

```jolt
struct Person ->
    $name: String
    $age: Uint
;;

enum Shape ->
    Circle(Float64)
    Rect(Float64, Float64)
;;

union Bits -> ... ;;
```

Classes (draft 1):

```jolt
@@DebugClass ->
    $log: Bool
    $verbose: Bool
;;

DebugClass::print() ->
    ...
;;
```

> `[OPEN]` **Big design question:** you have *both* `struct` and `@@class`. In Rust there are no
> classes — structs + traits cover it. Either:
> 1. Drop `@@class`, give `struct` methods (via `impl`/trait blocks), or
> 2. Keep both and define precisely how they differ (inheritance? vtables? heap-by-default?).
> Right now their roles overlap and `@@` adds a third "definition sigil" alongside `@` and `$`.

`[PROPOSAL]` Method syntax: define methods in the type body or in a separate `Type::method`
block (draft 1 uses the latter). Pick one and use it for struct + enum + class uniformly.

---

## 9. Memory & references  `[OPEN — needs a real design pass]`

The drafts disagree on spelling and don't define semantics:

| Concept        | draft 1            | draft 2                       |
| -------------- | ------------------ | ----------------------------- |
| make reference | `ref(person)`      | `Ref<Int>(person)` / `Ref(p)` |
| dereference    | `p::unref()`       | `Unref(p)`                    |
| ref operator   | `:&` decl, `=&` set| —                             |

`[DECISION]` (proposed normalization — confirm):

```jolt
using Memory

$person = Person::new("name")
$p: Ref<Person>     = ref(person)     # safe borrow
$d: Person          = unref(p)        # deref
$raw: Pointer<Person> = ptr(person)   # raw pointer (unsafe context)
```

Open questions that define the whole language:
- Is there **ownership + move semantics**? When `$a = b`, is `b` moved, copied, or aliased?
- **Borrowing rules**: one mutable XOR many shared? Lifetimes — inferred or annotated?
- What exactly distinguishes `Ref<T>` (safe) from `Pointer<T>` (raw)? Is `Pointer` only usable
  in `[unsafe]`?
- What do `:&` / `=&` (draft 1) add over `ref()` — are they kept or dropped?
- Heap vs stack: the `[heap]` attribute (§11) suggests opt-in heap allocation — define the default.

This section is the heart of "Rust-like memory safety" and currently has almost no semantics.
Worth a dedicated design doc.

---

## 10. Error handling  `[PROPOSAL — currently missing]`

You have an `error` keyword and an `Error` type, and `?` is listed as "option," but there's no
defined propagation story. Suggested (Rust-flavored):

```jolt
@read_config() Result<Config, Error> ->
    $f = open("cfg")?;        # ? propagates the error
    return Ok(parse(f));
;;
```

`[OPEN]` Decide: exceptions, `Result`-style values, or both? And reconcile `?` (option),
`??`, `error`, `Error`, `None`, `Empty` into one coherent error/absence model.

---

## 11. Attributes

```jolt
[heap]
@main() ->
    ...
;;

[{deprecated: "", since: ""}, heap, unsafe]
@test() ->
    ...
;;
```

`[DECISION]` Rewritten with `-> ;;` blocks (drafts used `{ }` here — see §6 conflict).
Known attributes: `heap`, `unsafe`, `deprecated{...}`. `[OPEN]` define each one's effect and
whether user-defined attributes are allowed.

---

## 12. Generics & constraints  `[PROPOSAL — barely specified]`

Only `@func |A| (...)` exists. For a static systems language you'll want bounds:

```jolt
@max |T: Ord| (a: T, b: T) T -> if a > b -> return a; ;; return b; ;;
```

`[OPEN]` This requires a **trait / interface system** (e.g. `Ord`, `Eq`, `Display`), which no
draft mentions. That's probably the single biggest missing feature for the "Rust-like" goal.
How do you want to express "this type supports operation X"?

---

## 13. Macros  `[OPEN]`

Conflicting designs:
- draft 1: `#ref(any)`, `#(ref, person)` — `#` prefix (collides with comments!)
- draft 3: macro = keyword + `{ }`, params in `( )` inside.

`[PROPOSAL]` Since `#` is now the comment char, move macros to a non-colliding sigil. Options:
`@!name(...)`, `name!(...)` (Rust-style), or keyword-block form. Pick one and define hygiene
(do macros see/leak surrounding bindings?).

---

## 14. Modules, packages, imports

```jolt
using Math                       # bring in a library/package
import Print module Utils        # import symbol from a module
import Utils.Print package Test  # import from a package
export Test as t, Test2 as t2
export {
    Test  as t,
    Test2 as t2,
}
```

`[OPEN]` Issues:
- `import Print module Utils` reads backwards from most languages (`import <what> module <where>`).
  Consider `import Utils.Print` / `from Utils import Print`.
- Draft 1 also shows Elm-style `module Main exposing (...)`. Mixing `exposing` with
  `export`/`using`/`import` is two paradigms — pick one.
- Draft 3 has a `program` keyword distinct from `module`. What's the difference? (Entry-point
  compilation unit vs library?)
- Define `using` vs `import` precisely: package-level vs symbol-level?

---

## 15. Compile-time & low-level interop

```jolt
comptime ->            # [PROPOSAL] block form; semantics [OPEN]
    ...
;;

asm ->
    movq $60, %rax     # exit syscall (Linux)
    movq $2,  %rdi
    syscall
;;

c ->
    /* C source */
;;
```

`[DECISION]` `asm` / `c` are plain keywords (drop draft 1's `#asm` / `#c`, which collide with
comments). `[OPEN]` Define `comptime` semantics (Zig-style? const-eval? compile-time codegen?).

---

## 16. Built-in functions / reflection

`typeinfo`, `log`, `len`, `help`. `[OPEN]` draft 3 also had `printd`. Standardize the set and
decide which are keywords vs stdlib functions (keywords for these is unusual — most languages
make `len`/`log` ordinary functions).

---

## Summary of redundancies worth collapsing

1. **Comment vs macro vs asm/c** all wanting `#` → give macros & asm/c their own syntax.
2. **`match` vs `switch`/`case` vs `if ->` cond-list** — three multi-branch constructs. Keep `match` + `if/else`; drop `switch`/`case`.
3. **Six loop forms** (`while`/`for`/`loop`/`when`/`do`/`repeat-until`) → three.
4. **`struct` vs `@@class`** → unify or sharply differentiate.
5. **`^` power vs `**` power** → one power operator.
6. **`None`/`Empty`/`?`/`??`/`error`/`Error`** → one absence model + one error model.
7. **Named (`AND`/`XOR`/…) and symbolic (`&`/`%|`/…) bitwise ops** both exist → keep both only if you have a reason (readability?), else drop the words.
8. **`:=` vs `=`** → dropped `:=`.
