# Jolt — Grammar (EBNF, v0.4)

> A complete grammar for the Jolt surface syntax, consistent with the v0.4 spec, stdlib, and
> `build.jolt`. Notation: `|` alternation, `( )` grouping, `[ ]` optional, `{ }` zero-or-more,
> `" "` literal terminals, `UPPER` lexical tokens (§1). Whitespace is insignificant except inside
> strings and as a token separator. This pass also fixes the last open item — **string
> interpolation** (§8.3).
>
> Two grammars are given: the **language** (§2–§10) and the **`build.jolt` declarative DSL** (§11),
> which is a restricted dialect sharing the language's lexer and expression grammar.

---

## 1. Lexical grammar

```ebnf
(* --- whitespace & comments (skipped) --- *)
ws          = { " " | "\t" | "\r" | "\n" } ;
comment     = line_comment | doc_comment ;
line_comment = "//" , { ANY - "\n" } , "\n" ;
doc_comment  = "///" , { ANY - "\n" } , "\n" ;       (* collected, not skipped, by the doc tool *)

(* --- identifiers (case enforced semantically, not lexically) --- *)
ident       = ( letter | "_" ) , { letter | digit | "_" } ;
letter      = "a".."z" | "A".."Z" | unicode_letter ;
digit       = "0".."9" ;
TYPE_IDENT  = ident ;   (* PascalCase, enforced post-parse *)
VAR_IDENT   = ident ;   (* snake_case, enforced post-parse *)

(* --- literals --- *)
int_lit     = dec_int | hex_int | oct_int | bin_int ;
dec_int     = digit , { digit | "_" } ;
hex_int     = "0x" , hexdigit , { hexdigit | "_" } ;
oct_int     = "0o" , octdigit , { octdigit | "_" } ;
bin_int     = "0b" , ( "0" | "1" ) , { "0" | "1" | "_" } ;
float_lit   = dec_int , "." , dec_int , [ exponent ] | dec_int , exponent ;
exponent    = ( "e" | "E" ) , [ "+" | "-" ] , dec_int ;
char_lit    = "'" , ( char_esc | ANY - "'" ) , "'" ;
bool_lit    = "true" | "false" ;

(* --- strings (see §8.3 for interpolation expansion) --- *)
string_lit   = '"' , { str_part } , '"' ;
mstring_lit  = "`" , { ANY - "`" } , "`" ;          (* raw multiline, no interpolation *)
str_part     = str_text | str_interp | str_escape ;
str_text     = { ANY - ( '"' | "{" | "\\" ) } ;
str_escape   = "\\" , ( '"' | "\\" | "n" | "t" | "r" | "0" | "{" | "}" | "u" , "{" , hexdigit+ , "}" ) ;
str_interp   = "{" , expr , [ ":" , format_spec ] , "}"
             | "{{" | "}}" ;                          (* literal braces *)
format_spec  = { ANY - "}" } ;                        (* e.g. hex, x, ".2", "08" *)
```

---

## 2. Program structure

```ebnf
file        = { item } ;

item        = use_decl
            | import_decl
            | export_decl
            | package_decl
            | library_decl
            | program_decl
            | fn_decl
            | struct_decl
            | enum_decl
            | union_decl
            | contract_decl
            | impl_block
            | macro_decl
            | comptime_block
            | extern_decl
            | const_binding
            | binding ;            (* top-level bindings allowed *)

use_decl     = "using" , path , ";" ;
import_decl  = "import" , path , ";"
             | "from" , path , "import" , import_list , ";" ;
import_list  = import_one , { "," , import_one } ;
import_one   = TYPE_IDENT , [ "as" , TYPE_IDENT ]
             | VAR_IDENT  , [ "as" , VAR_IDENT ] ;
export_decl  = "export" , "{" , import_list , "}" , [ ";" ] ;
package_decl = "package" , path , ";" ;
library_decl = "library" , path , ";" ;
program_decl = "program" , block ;

path        = ident , { "." , ident } ;
```

---

## 3. Attributes

```ebnf
attr_group  = "[" , attr , { "," , attr } , "]" ;
attr        = ident
            | ident , ":" , attr_value
            | "{" , attr_field , { "," , attr_field } , "}" ;
attr_field  = ident , ":" , attr_value ;
attr_value  = string_lit | int_lit | ident | "[" , [ attr_value , { "," , attr_value } ] , "]" ;

attrs       = { attr_group } ;     (* zero or more groups precede a declaration *)
```
Examples: `[public]`, `[const]`, `[alloc: arena]`, `[shared, sync]`,
`[{deprecated: "msg", since: "0.3"}]`, `[noalloc, nopanic, constanttime]`.

---

## 4. Bindings

```ebnf
binding        = attrs , sigil , VAR_IDENT , [ flex ] , [ ":" , type ] , "=" , expr , ";" ;
const_binding  = "[" , "const" , "]" , "$" , VAR_IDENT , [ ":" , type ] , "=" , expr , ";" ;
sigil          = "$$" | "$" ;          (* $$ = mutable, $ = immutable *)
flex           = "?" ;                  (* re-typeable; only valid with $$ *)

assign         = lvalue , assign_op , expr , ";" ;
assign_op      = "=" | "+=" | "-=" | "*=" | "/=" | "^=" | "%=" | "//="
               | "&=" | "|=" | "%|=" | "~&=" | "~|=" | "~%|="
               | ">>=" | "<<=" | ">>|=" | "<<|=" | "<<<=" | ">>>=" ;
lvalue         = VAR_IDENT | postfix_expr ;   (* a.b, a[i], deref targets *)

destructure    = bind_target , { "," , bind_target } , "=" , expr , ";" ;
bind_target    = sigil , VAR_IDENT | "_" ;
```

---

## 5. Functions, generics, closures

```ebnf
fn_decl     = attrs , [ "async" ] , "@" , fn_name , [ generics ] , params , [ type ] , fn_body ;
fn_name     = VAR_IDENT | "(" , operator , ")" ;        (* @(+) operator method *)
operator    = "+" | "-" | "*" | "/" | "//" | "%" | "^"
            | "==" | "!=" | "<" | ">" | "<=" | ">="
            | "&" | "|" | "%|" | "~&" | "~|" | "~%|" | "~"
            | "<<" | ">>" | "<<<" | ">>>" | "<<|" | ">>|"
            | "[]" ;                                      (* index operator *)

generics    = "|" , generic_param , { "," , generic_param } , "|" ;
generic_param = [ "comptime" ] , TYPE_IDENT , [ ":" , bound ]
              | "comptime" , VAR_IDENT , ":" , type        (* value generic: comptime N: Uint *)
              | "life" , TYPE_IDENT ;                       (* lifetime parameter *)
bound       = TYPE_IDENT , { "+" , TYPE_IDENT } ;           (* contract bounds *)

params      = "(" , [ param , { "," , param } ] , ")" ;
param       = [ "comptime" ] , recv_or_param ;
recv_or_param = "self" | "$$" , "self"                       (* method receivers *)
              | VAR_IDENT , ":" , type , [ "=" , expr ]      (* default arg *)
              | guard ;                                      (* dispatch guard *)
guard       = "{" , expr , "}" ;

fn_body     = block | "->" , expr , ";;" ;                  (* block, or single-expr body *)

closure     = "|" , [ closure_params ] , "|" , [ type ] , block
            | "|" , [ closure_params ] , "|" , "->" , expr , ";;" ;
closure_params = closure_p , { "," , closure_p } ;
closure_p   = VAR_IDENT , [ ":" , type ] ;
```

---

## 6. Blocks & statements

```ebnf
block       = "->" , { statement } , [ expr ] , ";;" ;   (* trailing expr (no ;) = block value *)

statement   = binding
            | const_binding
            | destructure
            | assign
            | use_decl | import_decl
            | if_stmt          (* when used in statement position *)
            | match_stmt
            | loop_stmt
            | for_stmt
            | defer_stmt
            | scope_stmt
            | comptime_block
            | unsafe_block
            | return_stmt
            | break_stmt
            | next_stmt
            | expr , ";" ;     (* expression statement *)

return_stmt = "return" , [ expr ] , ";" ;
break_stmt  = "break" , [ ident ] , ";" ;     (* optional loop label *)
next_stmt   = "next" , [ ident ] , ";" ;
defer_stmt  = ( "defer" | "errdefer" ) , ( expr , ";" | block ) ;
unsafe_block = "[" , "unsafe" , "]" , block ;
comptime_block = "comptime" , block ;
scope_stmt  = "scope" , block ;
```

---

## 7. Control flow

```ebnf
(* if — both single-condition and condition-list forms; usable as expr or stmt *)
if_expr     = "if" , expr , block , [ "else" , ( block | if_expr ) ]
            | "if" , "->" , { cond_arm } , [ default_arm ] , ";;" ;
cond_arm    = expr , ":" , ( expr , ";" | { statement } ) ;
default_arm = "default" , ( ":" , ( expr , ";" | { statement } ) | block ) ;
if_stmt     = if_expr ;

(* match — pattern matching; usable as expr or stmt *)
match_expr  = "match" , expr , "->" , { match_arm } , ";;" ;
match_arm   = pattern , [ "if" , expr ] , "->" , ( expr , ";" | block_inline ) ;
block_inline = { statement } , [ expr ] , ";" ;

(* loops *)
loop_stmt   = [ label ] , "loop" , block ;
for_stmt    = [ label ] , "for" , ( for_in | expr ) , block ;
for_in      = pattern , "in" , expr ;        (* iterate *)
                                              (* `for <expr>` = while form *)
label       = "[" , "label" , ":" , ident , "]" ;
```

---

## 8. Patterns (full grammar)

```ebnf
pattern     = or_pattern ;
or_pattern  = primary_pattern , { "|" , primary_pattern } ;     (* or-patterns *)
primary_pattern
            = capture_pattern
            | literal_pattern
            | range_pattern
            | enum_pattern
            | tuple_pattern
            | struct_pattern
            | wildcard
            | rest ;

capture_pattern = VAR_IDENT , ":=" , primary_pattern ;          (* whole-value capture, reuses := *)
literal_pattern = int_lit | float_lit | char_lit | string_lit | bool_lit
                | "Some" , "(" , pattern , ")"
                | "Nothing" | "None"
                | "Ok" , "(" , pattern , ")" | "Err" , "(" , pattern , ")" ;
range_pattern   = literal_pattern , ".." , literal_pattern ;
enum_pattern    = TYPE_IDENT , [ "(" , [ pattern , { "," , pattern } ] , ")" ] ;
tuple_pattern   = "(" , pattern , { "," , pattern } , ")" ;
struct_pattern  = TYPE_IDENT , "{" , field_pat , { "," , field_pat } , [ "," , rest ] , "}" ;
field_pat       = VAR_IDENT , [ ":" , pattern ] ;               (* shorthand binds same name *)
wildcard        = "_" ;
rest            = ".." ;
```

---

## 9. Expressions (with precedence)

Lowest-to-highest precedence is encoded by the rule chain (each rule binds tighter than the one above).

```ebnf
expr        = ternary_expr ;
ternary_expr = or_expr , [ "?" , expr , ":" , expr ] ;
or_expr     = and_expr , { ( "||" | "or" ) , and_expr } ;
and_expr    = not_expr , { ( "&&" | "and" ) , not_expr } ;
not_expr    = [ "not" ] , eq_expr ;
eq_expr     = rel_expr , { ( "==" | "!=" ) , rel_expr } ;
rel_expr    = bitxnor_expr , { ( ">=" | ">" | "<=" | "<" | "as" ) , bitxnor_expr } ;
bitxnor_expr= bitnor_expr  , { "~%|" , bitnor_expr } ;
bitnor_expr = bitnand_expr , { "~|"  , bitnand_expr } ;
bitnand_expr= bitxor_expr  , { "~&"  , bitxor_expr } ;
bitxor_expr = bitor_expr   , { "%|"  , bitor_expr } ;
bitor_expr  = bitnot_expr  , { "|"   , bitnot_expr } ;
bitnot_expr = bitand_expr  , { "~"   , bitand_expr } ;   (* binary ~ here; unary ~ in prefix *)
bitand_expr = shift_expr   , { "&"   , shift_expr } ;
shift_expr  = add_expr , { ( "<<" | ">>" | "<<<" | ">>>" | "<<|" | ">>|" ) , add_expr } ;
add_expr    = mul_expr , { ( "+" | "-" ) , mul_expr } ;
mul_expr    = prefix_expr , { ( "*" | "/" | "//" | "%" | "^" ) , prefix_expr } ;
prefix_expr = ( "-" | "!" | "++" | "--" | "~" ) , prefix_expr | range_expr ;
range_expr  = postfix_expr , [ ( ".." | "..." ) , postfix_expr ] ;
postfix_expr= primary_expr , { postfix_op } ;
postfix_op  = "++" | "--"
            | "[" , expr , "]" | "?[" , expr , "]"
            | "." , ( VAR_IDENT | int_lit ) | "?." , VAR_IDENT
            | "?" | "??"                      (* error-propagate / option-coalesce *)
            | call_args ;
call_args   = "(" , [ arg , { "," , arg } ] , ")" ;
arg         = [ VAR_IDENT , ":" ] , expr ;    (* named argument *)

primary_expr= int_lit | float_lit | char_lit | string_lit | mstring_lit | bool_lit
            | "None" | "Some" "(" expr ")" | "Nothing"
            | "Ok" "(" expr ")" | "Err" "(" expr ")"
            | VAR_IDENT
            | qualified                         (* Type::member / Type::method *)
            | array_lit | map_lit | tuple_lit
            | struct_lit
            | closure
            | if_expr | match_expr
            | block                             (* block as expression *)
            | "(" , expr , ")"
            | "await" , expr                    (* reserved; not in v0.4 I/O model *)
            | "@" , fn_name , [ generics ] , call_args ;   (* call by function value, rare *)

qualified   = TYPE_IDENT , "::" , ( VAR_IDENT | TYPE_IDENT ) , [ call_args ] ;
array_lit   = "[" , [ expr , { "," , expr } ] , "]" ;
map_lit     = "{" , [ map_entry , { "," , map_entry } ] , "}" ;
map_entry   = expr , ":" , expr ;
tuple_lit   = "(" , expr , "," , [ expr , { "," , expr } ] , ")" ;
struct_lit  = TYPE_IDENT , "{" , field_init , { "," , field_init } , "}" ;
field_init  = VAR_IDENT , ":" , expr | VAR_IDENT ;        (* shorthand *)
```

> Note: `?` and `??` are postfix (error-propagate / option-coalesce); the ternary `? :` is the only
> infix use of `?`. The parser disambiguates by context (a `?` immediately followed by an expression
> and a later `:` at the same level is ternary; otherwise postfix).

---

## 10. Types

```ebnf
type        = ref_type | dyn_type | fn_type | base_type ;
base_type   = TYPE_IDENT , [ type_args ] | "Self" ;
type_args   = "<" , type_arg , { "," , type_arg } , ">" ;
type_arg    = type | int_lit | VAR_IDENT ;          (* value generics: Array<Int, 16> *)
ref_type    = "Borrow" , "<" , type , [ "," , TYPE_IDENT ] , ">"   (* optional lifetime param *)
            | "Claim" , "<" , type , [ "," , TYPE_IDENT ] , ">"
            | "Pointer" , "<" , type , ">" ;
dyn_type    = "dyn" , TYPE_IDENT ;
fn_type     = "@" , params , [ type ] ;             (* function type *)
opt_type    = type , "?" ;                           (* sugar for Option<T> *)
err_type    = "!" , type ;                           (* sugar for Result<T, openset> *)
```
(`opt_type` and `err_type` appear wherever `type` does, as suffix/prefix forms.)

---

## 11. Declarations: struct / enum / union / contract / impl

```ebnf
struct_decl  = attrs , "struct" , TYPE_IDENT , [ generics ] , "->" , { field_decl } , ";;" ;
field_decl   = sigil , VAR_IDENT , ":" , type , [ ";" ] ;
enum_decl    = attrs , "enum" , TYPE_IDENT , [ generics ] , "->" , { variant_decl } , ";;" ;
variant_decl = TYPE_IDENT , [ "(" , type , { "," , type } , ")" ] , ";" ;
union_decl   = attrs , "union" , TYPE_IDENT , "->" , { field_decl } , ";;" ;

contract_decl = attrs , "@@" , TYPE_IDENT , [ generics ] , "->" , { contract_member } , ";;" ;
contract_member = "@" , fn_name , [ generics ] , params , [ type ] , ( ";" | fn_body ) ;
                                                    (* `;` = required, body = default method *)

impl_block   = TYPE_IDENT , "::" , method_or_contract ;
method_or_contract
             = ( VAR_IDENT | "(" operator ")" ) , [ generics ] , params , [ type ] , fn_body  (* method *)
             | TYPE_IDENT , "->" , { contract_member } , ";;" ;                                (* adopt a contract *)

extern_decl  = attrs , "extern" , string_lit? , "@" , VAR_IDENT , params , [ type ] , ";" ;
macro_decl   = "#" , "macro" , VAR_IDENT , params_pat , "->" , macro_body , ";;" ;
macro_invoke = "#" , VAR_IDENT , call_args | "#" , "(" , expr , { "," , expr } , ")" ;
```

---

## 12. The `build.jolt` declarative DSL

A restricted dialect: shares the lexer (§1), expressions (§9), and `comptime` blocks, but the
top level is declarations, not arbitrary statements.

```ebnf
build_file  = { build_top } ;
build_top   = "build" , "->" , { build_item } , ";;"
            | comptime_block ;

build_item  = target_decl | step_decl | generate_decl | members_decl | comptime_block ;

target_decl = target_kind , string_lit , "->" , { target_field } , ";;" ;
target_kind = "program" | "library" | "staticlib" | "dylib"
            | "object" | "firmware" | "kernel" ;
target_field= field_name , ":" , field_value , [ "when" , expr ] , ";"
            | "steps" , "->" , { ident , ";" } , ";;" ;
field_name  = "root" | "uses" | "needs" | "link_c" | "target" | "optimize"
            | "linker_script" | "asm" | "emit" | "cap" | "includes" ;
field_value = string_lit | ident | array_lit | call_expr ;
call_expr   = ident , "(" , [ arg , { "," , arg } ] , ")" ;   (* git(...), path(...), generated(...) *)

step_decl   = "step" , string_lit , "->" , { step_field } , ";;" ;
step_field  = ( "runs" | "tests" | "command" | "help" ) , ":" , field_value , ";"
            | "depends_on" , ":" , field_value , ";" ;

generate_decl = "generate" , string_lit , "->" , comptime_block , ";;" ;
members_decl  = "members" , "->" , { string_lit , ";" } , ";;" ;
```

Build conditionals use the same `when expr` clause; `feature("x")`, `target.os`, etc. are ordinary
expressions evaluated at build comptime.

---

## 13. Operator precedence summary (for implementers)

From loosest to tightest binding:

```
1  ternary        ? :
2  logical-or     ||  or
3  logical-and    &&  and
4  negation       not                (prefix)
5  equality       ==  !=
6  relational     >= > <= <  as
7  bitwise        ~%|  ~|  ~&  %|  |  ~  &     (xnor..and, in this order)
8  shift          << >> <<< >>> <<| >>|
9  additive       + -
10 multiplicative * / // % ^
11 prefix unary   - ! ++ -- ~
12 range          .. ...
13 postfix        ++ -- [] ?[] . ?. ? ?? call
14 primary        literals, idents, (), Type::x, struct/array/map lits, closures, blocks
```

Notes for the parser:
- `^` is **right-associative** (power); all other binary operators are left-associative.
- Unary `~` (bitwise not) is prefix (level 11); binary `~`... is *not* an operator — bitwise NOT is
  unary only, so `~` only appears prefixed. (The §9 chain lists `~` defensively; treat as unary.)
- `as` is a binary type-test/cast at relational level, RHS is a `type`.
- Generic `|T|` brackets vs bitwise `|`: `|` opens a generic list only immediately after `@name`,
  `struct/enum/contract Name`, or in a closure head — elsewhere it's bitwise-or. The parser keys off
  position, avoiding the C++ `< >` ambiguity entirely.
- `;` separates statements; `;;` closes a block. A block's last item, if it omits `;`, is the block's
  value (expression position).

---

## 14. Resolved by this grammar

- **String interpolation** (last open question): `"{ expr [: format_spec] }"`, `{{`/`}}` escape — §1,
  §8.3 lexer rules. `format_spec` is an opaque token consumed by `Display`/format machinery.
- **`@(+)` operator methods**: `fn_name = "(" operator ")"` (§5, §11) — unambiguous.
- **Value generics × patterns/guards**: `type_arg` allows `int_lit`/`VAR_IDENT` (§10); `guard` is a
  param-list `{ expr }` (§5).
- **`|T|` vs `|` (or) vs closures**: disambiguated positionally (§13).
- **`?` overloading** (ternary vs postfix error/option): positional disambiguation (§9 note).
- **`build.jolt`** forms: §12.

## Remaining for a real parser
- Precise tokenizer max-munch rules (e.g. `>>>` vs `>>` vs `>`), and whether `<<<`/`>>>` conflict with
  generic close `>` (they don't — generics use `<`/`>` only in `type_args`, never `<<`).
- Error-recovery productions (for the LSP) — omitted here; this is the accepting grammar.
- Doc-comment / attribute association rules (which item a `///` or `[attr]` binds to — the next item).
