# Jolt — Build System (`build.jolt`) & Package Manifest (`jolt.toml`)

> Two files with **clean, separate responsibilities**:
>
> - **`jolt.toml`** — the **package manifest**: identity, version, edition, dependencies, features,
>   registry/package-manager metadata. Always present (even a one-liner). Owned by the package
>   manager.
> - **`build.jolt`** — the **build logic**: targets, build steps, codegen, target/optimize/capability
>   wiring. **Optional** — only needed when conventions + CLI flags aren't enough. Owned by the build
>   system.
>
> The build system is **declarative-first with a `comptime` escape hatch**, in Jolt's own syntax —
> not an imperative graph-builder. You *describe* targets as declaration blocks using the same
> `->`/`;;` blocks and attributes as the rest of the language; the driver derives the dependency
> graph. Real logic (codegen, conditional wiring) drops into a `comptime` block. And with **no
> `build.jolt` at all**, the CLI builds from conventions + flags with zero configuration.
>
> `build.jolt` never restates what `jolt.toml` already holds — it *reads* manifest values
> (dependencies, features, version) and focuses purely on *how* to build. A `jolt.lock` is generated
> for dependency pinning.

---

## Division of responsibility

| Concern | Lives in |
| ------- | -------- |
| package name, version, edition | `jolt.toml` |
| dependencies + their versions/sources/features | `jolt.toml` |
| feature flag declarations | `jolt.toml` |
| registry/publish metadata, license, authors | `jolt.toml` |
| build profiles (default opt levels, LTO) | `jolt.toml` |
| **what targets to produce** (program/library/firmware…) | `build.jolt` (or inferred) |
| **build steps** (run, flash, codegen, custom) | `build.jolt` |
| **target/optimize/capability/linker wiring** | `build.jolt` (or CLI flags) |
| dependency *pinning* (generated) | `jolt.lock` |

Rule of thumb: **`jolt.toml` = what the package *is*; `build.jolt` = how it's *built*.**

---

## The manifest — `jolt.toml`

```toml
[package]
name = "myapp"
version = "0.1.0"
edition = "2026"
license = "MIT"
authors = ["…"]

[dependencies]
http = "1.2"
json = { version = "0.4", features = ["derive"] }
sensor = { git = "https://…", rev = "abc123" }
hal = { path = "../hal" }

[target.thumbv7em-none-eabi.dependencies]   # per-target deps
cortex_m = "0.7"

[features]
default = ["std"]
std = []
gui = []

[profile.release]
opt = "O2"
lto = true
```

Dependencies declared here are resolved by the package manager and made available to both the
program's `using`/`import` and to `build.jolt` (via `manifest.dependency("name")`). `jolt add http@1.2`
edits `jolt.toml`, never `build.jolt`.

---

## Three levels of build configuration

Jolt scales the build to the project, so you never write more than you need:

1. **No `build.jolt` (zero-config).** `jolt build` infers targets from directory layout + CLI flags.
   Most small programs and libraries need only `jolt.toml`.
2. **Declarative `build.jolt`.** Targets, options, and steps as declaration blocks. Covers most
   real projects.
3. **Declarative + `comptime` logic.** Drop into `comptime` for codegen, conditional targets, or
   custom steps — full language, only where needed.

The distinguishing choice: Zig makes *everyone* write imperative graph code; Jolt makes the common
case **no build file**, the normal case **declarations**, and reserves imperative code for the hard
case — with package metadata cleanly separated into `jolt.toml` either way.

---

## Level 1 — Zero-config (only `jolt.toml`, no `build.jolt`)

With no `build.jolt`, the driver builds from **conventions**:

| Convention | Meaning |
| ---------- | ------- |
| `src/main.jolt` present | build a **program** named after the package |
| `src/lib.jolt` present (no `main`) | build a **library** named after the package |
| both present | build the library, and a program that uses it |
| `tests/` or `[test]` fns | discovered automatically by `jolt test` |
| `src/bin/*.jolt` | each is an additional program target |

Everything else comes from **CLI flags**, which are the same knobs a `build.jolt` would set:

```
jolt build                              # debug build of the inferred target
jolt build --release                    # optimize
jolt build --target wasm32-unknown      # cross-compile
jolt build --opt size                   # ReleaseSmall
jolt build --cap noalloc,nostd          # impose capability constraints repo-wide
jolt build --link-c SDL2                # link a C library
jolt build --feature gui                # enable a [cfg] feature
jolt build --emit bin,hex               # firmware artifacts
jolt run -- arg1 arg2                    # build + run with program args
jolt add http@1.2                        # adds a dep (writes a minimal build.jolt if none exists)
```

So a hello-world is just `jolt new myapp && cd myapp && jolt run` — no build file is generated until
you need one. `jolt build --explain` prints the equivalent `build.jolt` the conventions imply, so you
can "graduate" to Level 2 by copying it.

---

## Level 2 — Declarative `build.jolt`

A `build.jolt` is a normal Jolt file holding **build logic only** — it does **not** restate package
identity, version, or dependencies (those live in `jolt.toml`). You **declare targets** as blocks;
the driver evaluates them at comptime and derives the graph, pulling dependency/feature info from the
manifest.

```jolt
build ->                                  // no name/version here — that's jolt.toml's job
    program "myapp" ->                     // a target
        root: "src/main.jolt";
        uses: [core];                      // local libraries
        needs: [http, json];               // deps resolved from jolt.toml [dependencies]
        link_c: ["SDL2"] when feature("gui");   // feature declared in jolt.toml [features]
    ;;

    library "core" ->
        root: "src/core/lib.jolt";
    ;;
;;
```

### What the declaration forms are
- `build -> … ;;` — the build block; holds target declarations and steps. **No** metadata/deps.
- `program "<name>" -> … ;;` / `library …` / `staticlib …` / `dylib …` / `object …` / `firmware …`
  / `kernel …` — **target** declarations.
- Inside a target: `root`, `uses` (local libraries), `needs` (manifest dependencies to link),
  `link_c`, `target:` (triple), `optimize:`, `linker_script`, `emit`, `cap:` (capability
  constraints), `generate` (codegen, see Level 3).
- **`when <cond>`** — a declarative conditional clause on any field (`when feature("gui")`, where the
  feature itself is declared in `jolt.toml`; `when target.os == "linux"`).
- `step "<name>" -> … ;;` — custom CLI steps.

Features and dependencies are *referenced* by name; their definitions stay in `jolt.toml`.

### Targets, optimize, capabilities are declarative
```jolt
build ->
    firmware "blink" ->
        root: "src/main.jolt";
        target: "thumbv7em-none-eabi";       // or omit → use --target / default
        optimize: embedded;
        cap: [noalloc, nostd];               // whole-image capability constraint
        linker_script: "link/stm32.ld";
        emit: [elf, bin, hex];
        steps -> flash; size; ;;             // request built-in steps
    ;;
;;
```

### Custom named steps (still declarative)
```jolt
build ->
    program "myapp" -> root: "src/main.jolt"; ;;
    step "run"  -> runs: "myapp"; help: "build and run"; ;;
    step "test" -> tests: "src"; ;;
    step "lint" -> command: "jolt lint"; ;;
;;
```

`jolt build`, `jolt build run`, `jolt build flash`, etc. invoke these.

---

## Level 3 — Declarative + `comptime`

When a field needs computation, or a target must be generated, embed a `comptime` block. This is the
*same* `comptime` from the language spec — no special build DSL.

```jolt
build ->
    // generate a source file from a schema, at build time
    generate "tables" ->
        comptime ->
            $schema = read_input("schema.json");      // declared input → sandboxed read
            $code   = emit_tables(parse(schema));
            produce("src/tables.jolt", code);          // becomes a LazyPath input
        ;;
    ;;

    program "app" ->
        root: "src/main.jolt";
        includes: [generated("tables")];
    ;;

    // conditionally declare extra targets with a loop
    comptime ->
        for plat in ["x86_64-linux-gnu", "aarch64-macos", "wasm32-unknown"] ->
            declare_program("app-" + plat, root: "src/main.jolt", target: plat);
        ;;
    ;;
;;
```

The rule: **declarations for structure, `comptime` for computation.** You only reach for `comptime`
where a static declaration can't say what you mean. (Version, deps, and features are never here —
they're in `jolt.toml`.)

---

## How the driver uses it

1. Read `jolt.toml` (always) → package identity, dependencies, features, profiles.
2. If `build.jolt` is absent → infer targets from conventions + flags (Level 1).
3. If present → evaluate it at comptime to obtain **target/step declarations**, resolving `needs`/
   `feature(...)` references against the manifest, then derive the graph and run the requested step
   (default: build + install all top-level targets).
4. Steps run in parallel where the graph allows; the content-addressed cache skips unchanged work.
5. `comptime` build code is **capability-sandboxed** (`[noio]`-by-default; reads/writes only declared
   inputs/outputs) for reproducibility; network/extra-fs access needs a `[build_io]` attribute **and**
   a `--allow-build-*` grant — see the Deno-style permission model in `jolt-security-model.md`.

Because the declarations are evaluated by the language's own comptime engine, the build file is
type-checked, formatted by `jolt fmt`, and understood by the LSP just like any other Jolt source.

---

## Templates — `jolt new <name> [-t <template>]`

Templates scaffold a layout and a matching `build.jolt` (or none, for the simplest). Default is
`exe`. Each generates `src/`, a `.gitignore`, and a starter doc + `[test]`.

| `-t` | Produces | Generated build.jolt? |
| ---- | -------- | --------------------- |
| `exe` (default) | runnable program | none — relies on zero-config conventions |
| `lib` | reusable library | minimal (metadata + test/doc steps) |
| `staticlib` | C-linkable static lib | declares `staticlib` + `[extern: "C"]` exports |
| `dylib` | shared library | declares `dylib` |
| `freestanding` | `[nostd]` no-OS binary | `cap: [nostd]`, linker script, `embedded` optimize |
| `embedded` | MCU firmware | `firmware` target, `cap: [noalloc]`, `flash`/`size` steps |
| `kernel` | bootable kernel | `cap: [nostd, noalloc]`, boot asm, `iso`/`qemu` steps |
| `wasm` | wasm32 module | `target: wasm32`, exports |
| `workspace` | monorepo | top-level `build.jolt` listing sub-packages |
| `cffi` | binds a C library | `requires` + `generate` bindgen step |
| `cli` | CLI app | arg-parsing scaffold |

### `exe` — `jolt.toml` only, *no* build file
```
myapp/
├── jolt.toml
├── .gitignore
└── src/
    └── main.jolt
```
```toml
# jolt.toml
[package]
name = "myapp"
version = "0.1.0"
edition = "2026"
```
```jolt
// src/main.jolt
using Std;
[public]
@main() !None -> println("Hello from myapp!"); ;;
```
`jolt run` just works via conventions. Add a `build.jolt` only when you outgrow defaults
(`jolt build --explain` prints the implied one to start from).

### `embedded`
```jolt
// build.jolt   (package name/version/deps are in jolt.toml)
build ->
    firmware "firmware" ->
        root: "src/main.jolt";
        target: "thumbv7em-none-eabi";      // override with --target
        optimize: embedded;
        cap: [noalloc];
        linker_script: "link/stm32.ld";
        emit: [elf, bin];
        steps -> flash; size; ;;
    ;;
;;
```
```jolt
// src/main.jolt
using Embed;
[no_mangle]
@_start() None ->
    $gpio = Gpio::take(PORTA);
    loop -> gpio.toggle(PIN5); delay_cycles(1_000_000); ;;
;;
```

### `kernel`
```jolt
// build.jolt   (metadata in jolt.toml)
build ->
    kernel "kernel" ->
        root: "src/kmain.jolt";
        target: "x86_64-none-elf";
        cap: [nostd, noalloc];
        linker_script: "link/kernel.ld";
        asm: ["src/boot.s"];
        steps -> iso; qemu; ;;
    ;;
;;
```

### `workspace`
```jolt
// build.jolt (repo root) — members & their own jolt.toml carry metadata/deps
build ->
    members -> "packages/core"; "packages/cli"; "packages/service"; ;;
    step "test" -> tests: members; ;;
;;
```

Custom templates: `jolt new -t <git-url>` scaffolds from a repo; `jolt template list/add` manages them.

---

## Why this is Jolt's own style (vs Zig)

| | Zig (`build.zig`) | Jolt (`build.jolt`) |
| - | ------------------ | ------------------- |
| common case | write imperative graph code | **no file at all** (conventions + flags) |
| normal case | `b.addExecutable(...)` calls | **declarations** in `->`/`;;` blocks |
| conditionals | `if` in build code | **`when <cond>`** clauses on fields |
| logic | always imperative | **`comptime` only when needed** |
| metadata | `build.zig.zon` + `build.zig` | **`jolt.toml`** (always) + optional `build.jolt` |
| capabilities | n/a | `cap:` constraints are first-class |

Same surface syntax as the language (`@`, `->`/`;;`, attributes, `comptime`), capability-aware, and
declarative-first — it reads like Jolt, not like a build tool bolted on. Metadata and dependencies
stay cleanly in `jolt.toml`, owned by the package manager; `build.jolt` is purely build logic.

---

## Open questions

1. **Declaration vs comptime boundary** — exactly which fields accept a `when` clause vs requiring a
   `comptime` block? (Lean: all scalar fields accept `when`; only generation/looping needs `comptime`.)
2. **Conventions specificity** — confirm the zero-config inference rules (`src/main.jolt`, `src/bin/*`,
   `src/lib.jolt`) and how `--target`/`--cap` flags compose with a partial `build.jolt`.
3. **`needs` vs automatic linking** — does a target have to list `needs: [http]`, or are all
   `jolt.toml` dependencies linkable implicitly? (Lean: implicit availability, `needs` only to scope
   a dep to specific targets.)
4. **`members` cross-deps** — declarative path deps per member (in each member's `jolt.toml`), or a
   root-level wiring block?
5. **Sandbox default for build `comptime`** — confirm `[noio]`-by-default + `[build_io]` opt-in.
