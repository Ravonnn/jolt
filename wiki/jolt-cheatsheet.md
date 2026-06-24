# Jolt — One-Page Cheat Sheet (v0.3)

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

### Memory  (move + borrow-checked, no implicit copy, no null)
```jolt
$b = a;            // MOVE (a now unusable)
$d = copy(big);    // explicit copy; auto only for Copy-contract types
$v = ref(data);    // shared borrow Ref<T>
$m = ref_mut(data);// mutable borrow RefMut<T> (source must be $$); shared XOR mutable
read(unref(v));    // deref (never frees); non-lexical borrows
[shared] $t = Node::new();        // counted shared ownership (non-atomic)
[shared, sync] $u = ...;          // atomic, cross-thread
child.parent = weak(t);           // [weak] ref breaks cycles
[alloc: arena] @f() !T -> ...     // explicit allocator (else implicit default)
[unsafe] @g() -> $p = ptr(x); ;;  // raw Pointer<T> only here
// lifetimes inferred; rare escape: |life L| ... Ref<T, L>
```

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
