# Jolt — One-Page Cheat Sheet (v0.4)

> **Tiny subset today:** `@main`, `$`/`$$`, `Int`/`Bool`/`String`/`None`, `if`/`loop`/`for`,
> `borrow`/`claim`/`deref`, `println`, `assert_eq`, `[test]`. See
> [Jolt 0.1 preview](../design/jolt-0.1-preview.md).

### Comments
```jolt
// line          /// doc (line, repeatable)
```

### Bindings  (immutable by default)
```jolt
$x = 10          // immutable        $$y = 10   // mutable
x = 11 // ERROR  y = 11 // ok
[const] $MAX = 64        $name: String = "Aoi"   // typed
$$slot? = 9.0            // mutable + re-typeable
$a, $b, _ = 1, 2, 3      // destructure, _ discards
```

### Naming  (enforced)
`snake_case` vars/fns · `PascalCase` types/contracts · `SCREAMING_SNAKE` consts · `PascalCase` packages

### Blocks
`->` opens · `;;` closes · `;` ends a statement

### Key types
`Int Uint Int8..128 Uint8..128 Byte Short Long Bool` · `Float16/32/64/128 Double`
`Char`(32-bit scalar) `String`(UTF-8) `Array Set Map Tuple Pair Raw`
`Complex<T> Rational<T>` = stdlib · `None`=absence, `Empty`=empty collection (no null)

### Operators
```
^ power (NOT xor)   // floor-div   %|=xor  ~=not  &|=and/or  ~& ~| ~%|=nand/nor/xnor
<< >> arith shift   <<< >>> logical shift   <<| >>| rotate
== != < <= > >=     && and / || or / not
?  = error propagate     ??  = Option unwrap/coalesce
.. range   ... spread     ÷ √ ≠ ≈ ≡  = unicode aliases
```

### Functions
```jolt
@double(x: Int) Int -> x * 2 ;;
@max |T: Comparable| (a: T, b: T) T -> ... ;;     // generics in |…|
$inc = |x: Int| Int -> x + 1 ;;                   // closure

// multiple dispatch (guards = runtime; identical sig = compile err; both true = runtime err)
@conv(m: Float64, {unit == "inch"}) Float64 -> m / 2.54 ;;
@conv(m: Float64, {unit != "inch"}) Float64 -> m * 2.54 ;;
```

### Control flow
```jolt
if c -> ... else -> ... ;;
if -> c1: a; c2: b; default: d; ;;        // cond-list
switch v -> case 1: a; default: d; ;;     // constants
match v -> Circle(r) -> ...; Rect(w,h) -> ...; ;;   // patterns + exhaustive
loop -> if done -> break; ;; ;;           // infinite
for i in 0..10 -> ... ;;                   // iterate
for x < 100 -> ... ;;                       // "while"
break · next · return
```

### Memory  (Custodian — move + borrow-checked, no null)

**v0.4 names (Tiny compiler):**

```jolt
$b = a;              // MOVE for String (a unusable after)
$v = borrow(data);    // shared borrow
$m = claim(data);     // exclusive claim ($$ source for mutation)
read(deref(v));      // read through borrow/claim handle
```

**Full language (spec):** `copy()`, `[shared]`, `[weak]`, `[alloc: …]`, `[unsafe]` — see
[spec v0.4](jolt-spec-v0.4.md). Legacy cheatsheet used `ref`/`ref_mut`/`unref`; v0.4 uses
`borrow`/`claim`/`deref`.

### Option & errors  (no exceptions)
```jolt
@find(...) Int? -> return Some(i); ... return None; ;;
$$i = find(..) ?? -1;                      // ?? unwraps Option

@read(p) !Config -> $f = open(p)?; ... ;;  // !T = open inferred error set
@load(p) Result<Data, LoadError> -> ...    // explicit = closed/exhaustive
error NotFound;                            // declare; Error = root contract
defer close(f);      errdefer free(buf);   // LIFO; errdefer only on error exit
```

### Structs / enums / contracts
```jolt
struct Person -> $name: String; $$age: Uint; ;;
enum Shape -> Circle(Float64); Rect(Float64, Float64); ;;
Person::greet(self) None -> println(self.name); ;;       // self = shared
Person::birthday($$self) None -> self.age = self.age+1; ;;// $$self = mutable

@@Comparable ->                            // contract = trait/interface/bound
    @compare(self, other: Self) Int;       // required (no data, no inheritance)
    @max(self, other: Self) Self -> ... ;; // default method
;;
Person::Comparable -> @compare(self, o: Self) Int -> self.age - o.age ;; ;;
```

### Visibility / modules
```jolt
[public] @api() -> ...        // private by default
using Math;                   // whole package
import Utils.Print;           // qualified
from Utils import Print as P;  // selective
export { Add as add };
package MyApp;  library MyApp.Core;  program -> ... ;;   // distribution tiers
```

### Meta / low-level
```jolt
#macro swap(a, b) -> $tmp = a; a = b; b = $tmp; ;;   // hygienic declarative macro
#swap(x, y);                  // macros are #-prefixed
comptime -> ... ;;            // compile-time eval (also backs proc-macros / user attrs)
asm -> ... ;;     c -> ... ;; // inline asm / C
len(x) println(x) typeinfo(x) help(x)   // prelude funcs, not keywords
```

### Attributes
`[public] [const] [unsafe] [shared] [shared, sync] [weak] [heap] [alloc: …] [{deprecated:"", since:""}]`
plus user-defined attributes (compile-time macros).
