# Jolt — Toolchain Design (v0.1)

> "Huge toolchain to help users" was a founding goal. This designs the whole developer-facing
> surface: one driver command (`jolt`) fronting a compiler, build system, package manager,
> formatter, linter, language server, debugger, test/bench runner, doc generator, and target
> management. Design principles first, then each tool.

---

## Guiding principles

1. **One binary, many verbs.** Everything is `jolt <verb>` (like `cargo`/`go`/`zig` rolled into one).
   No separate install for formatter, package manager, or test runner.
2. **The compiler is a library.** The CLI, LSP, formatter, and linter all call the same
   `libjolt` front end — one source of truth for parsing/typing/the Custodian, so tools never drift
   from the compiler.
3. **Reproducible & incrementally cached by default.** Pinned dependencies, a content-addressed,
   shareable build cache, deterministic output. Same inputs → byte-identical artifacts. The compiler
   is query-based so rebuilds track *what changed*, not project size — full design in
   **`jolt-caching-system.md`**.
4. **Capability- and target-aware end to end.** `[noalloc]`/`[nostd]`/`[constanttime]` and the target
   triple flow through every stage — the build system, not just the compiler, understands them.
5. **Fast feedback.** Incremental compilation, query-based architecture, parallel by default.

---

## The `jolt` driver — command surface

```
jolt new <name>          scaffold a new package (program or library)
jolt build               compile the current package
jolt run                 build + run (programs)
jolt run --allow-net=…   run with Deno-style permission grants (see Security, §13)
jolt test                discover & run [test] functions + doctests
jolt bench               run [bench] functions with statistics
jolt check               type/Custodian check without codegen (fast)
jolt fmt                 format sources
jolt lint                static analysis / style / capability lints
jolt doc                 generate documentation (+ run doctests)
jolt add <dep>           add a dependency to jolt.toml (manifest)
jolt update              update the lockfile
jolt fetch               download dependencies into the cache
jolt clean               clear build artifacts
jolt repl                interactive REPL (comptime-backed) — see Extended Toolchain
jolt lsp                 start the language server (editors invoke this)
jolt debug               build with debug info + launch the debugger
jolt target <add|list>   manage cross-compilation targets
jolt asm / jolt ir       dump generated assembly / intermediate IR
jolt explain <code>      expand a diagnostic / Custodian / capability / DST error
jolt fix / jolt migrate  apply machine-applicable fixes / edition migration
jolt profile             CPU / allocation / fiber profiling (+ PGO data)
jolt coverage            line/branch/region coverage, gating, HTML/LCOV
jolt verify              capability/contract/bounds verification (optional SMT)
jolt tree / why / outdated / audit / licenses / sbom / vendor   supply-chain
jolt deps --capabilities aggregate permission footprint of the dependency tree
jolt bindgen             generate/consume C / WASM / FFI bindings
jolt graph / bloat / size / timings / deadcode   build & artifact inspection
jolt version / jolt help
```

> Verbs from `jolt fix` onward (plus REPL depth, IDE/LSP extensions, and telemetry hooks) are
> designed in **`jolt-toolchain-extended.md`**. Each reuses `libjolt` + existing language machinery
> rather than being a standalone tool.

---

## 1. Compiler pipeline (`libjolt`)

Query-based / incremental (demand-driven, like rustc/Roslyn): every stage is a cached query keyed by
content hash, so editing one function reanalyzes only what depends on it.

```
source (.jolt)
  │  Lexer            → tokens                        (// comments, ;/;; , sigils)
  │  Parser           → AST                           (-> ;; blocks, |…| generics, @(+) ops)
  │  Macro/comptime   → expanded AST                  (#macros, comptime eval, [attr] proc-macros)
  │  Resolver         → names, modules, imports       (package/library/program tiers)
  │  Type checker     → typed HIR                     (inference, contracts, |T| generics, dyn)
  │  CUSTODIAN        → ownership/borrow/move proof    (the safety pass; emits "custody violation")
  │  Capability check → [noalloc]/[noio]/… transitive (call-graph walk vs annotated stdlib)
  │  Monomorphizer    → concrete instances            (value generics, comptime guards resolved here)
  │  Lowering         → MIR (mid-level IR)             (desugars match, loops, defer/errdefer, dispose)
  │  Backend          → object code                   (see §1.1)
  ▼  link             → executable / library / firmware
```

Notable stage interactions already decided:
- **Comptime** runs inside the front end (an interpreter over MIR) and powers procedural macros and
  `[constfn]`.
- **The Custodian** is its own pass after typing — keeping safety analysis separate keeps error
  messages focused ("custody violation: `data` moved on line N").
- **Capability check** consumes the stdlib's attribute annotations; it runs before monomorphization
  so violations name source functions, not generated instances.
- **Comptime guards** (`{N > 0}`) are evaluated in the monomorphizer; failures become compile errors.

### 1.1 Backends
- **Primary: LLVM** — mature optimization + the widest target list (x86, ARM, RISC-V, WASM, GPU).
- **Secondary: a fast in-house backend** (Cranelift-style) for **debug builds** — near-instant
  codegen, huge win for `jolt check`/edit-compile loops.
- **`[nostd]`/freestanding:** emits raw object files + honors `[link_section]`/`[no_mangle]`/custom
  entry; integrates a user linker script.
- Backend selection is automatic (fast backend for debug, LLVM for release) and overridable.

### 1.2 Optimization levels
`-O0` (fast backend, debug), `-O1`, `-O2` (default release), `-O3`, `-Osize`, `-Ofast` (assumes
no UB-traps), plus `-Oembedded` (size + `[noalloc]`/`[nostd]` friendly). LTO and PGO supported via
the LLVM path.

---

## 2. Diagnostics

A first-class concern — the compiler's UX *is* the language's UX.

- **Structured, with codes:** every error has a stable code (`E0123`); `jolt explain E0123` prints a
  full write-up with examples.
- **Custodian errors are teaching moments:** show the move/borrow site, the conflicting use, and a
  suggested fix ("`data` was moved here; borrow it with `borrow(data)` instead").
- **Capability errors name the chain:** "`hot_path` is `[noalloc]` but calls `build_list` → `alloc`
  on line N."
- Spans, multi-line carets, "did you mean", machine-readable JSON output for tooling, and severity
  control (`deny`/`warn`/`allow`) per lint.

---

## 3. Build system & manifest

Two files, **separate concerns**: **`jolt.toml`** (the package manifest — identity, version,
dependencies, features, profiles; owned by the package manager) and an **optional `build.jolt`** (the
build logic — targets, steps, codegen; owned by the build system). The build system is
**declarative-first, with a `comptime` escape hatch, in Jolt's own syntax**, and **zero-config when
there's no `build.jolt`**. Three levels:
1. **`jolt.toml` only, no `build.jolt`** → the CLI infers targets from directory conventions
   (`src/main.jolt` → program, `src/lib.jolt` → library, `src/bin/*` → extra programs) and flags.
2. **Declarative `build.jolt`** → targets/steps as `->`/`;;` blocks (`program "x" -> root: …; ;;`),
   with `when <cond>` clauses; `needs`/`feature(...)` reference the manifest. No imperative graph code.
3. **+ `comptime`** → the language's own `comptime` for codegen/computed targets, only where needed.

`build.jolt` never restates metadata/deps — those live in `jolt.toml`. Full design, declaration
forms, and `jolt new` templates: **`jolt-build-system.md`**.

### 3.1 Manifest — `jolt.toml`
```toml
[package]
name = "myapp"
version = "0.1.0"
edition = "2026"
license = "MIT"

[dependencies]
http = "1.2"
json = { version = "0.4", features = ["derive"] }
sensor = { git = "https://…", rev = "abc123" }
hal = { path = "../hal" }

[features]
default = ["std"]
std = []
gui = []

[profile.release]
opt = "O2"
lto = true
```
`jolt add http@1.2` edits `jolt.toml`; the package manager resolves it into `jolt.lock`.

### 3.2 Build logic — `build.jolt` (optional)
```jolt
build ->
    program "myapp" -> root: "src/main.jolt"; needs: [http, json]; link_c: ["SDL2"] when feature("gui"); ;;
    library "core"  -> root: "src/core/lib.jolt"; ;;
;;
```
- Zero-config: `jolt run` builds `src/main.jolt` with no `build.jolt` at all; `jolt build --explain`
  prints the implied one.
- Same surface syntax as the language; type-checked, `jolt fmt`-formatted, LSP-understood.
- `comptime` blocks handle codegen/looping; `cap:` constraints are first-class.
- **Templates:** `jolt new -t <exe|lib|staticlib|dylib|freestanding|embedded|kernel|wasm|workspace|cffi|cli>`;
  default `exe` ships `jolt.toml` + sources and **no** `build.jolt`.
- **Content-addressed cache:** shared across projects; reproducible builds.

---

## 4. Package manager

- **Registry:** central index (`pkg.jolt.dev`-style) plus git/path/local sources.
- **`jolt.lock`:** fully pinned, committed; guarantees reproducible dependency graphs.
- **SemVer resolution** with a clear conflict resolver; `jolt update` re-resolves.
- **Security:** signed packages, checksum verification, `jolt audit` against an advisory database,
  and **capability transparency** — a package's declared capabilities (does it do I/O? unsafe?
  allocate?) are surfaced so you can vet dependencies. A `[nostd]`/`[noalloc]` project can *refuse*
  dependencies that violate those constraints at resolve time.
- **Vendoring** for air-gapped/embedded workflows.

---

## 5. Formatter — `jolt fmt`

- Canonical, near-zero-config style (one true format, like gofmt) so the `->`/`;;` block style is
  consistent everywhere. Small knobs (line width) only.
- Idempotent, fast, runs on save via the LSP.
- Format-on-save and a `--check` mode for CI.

---

## 6. Linter — `jolt lint`

Beyond type/Custodian errors: style, correctness, and performance lints, e.g.
- unused bindings/imports, shadowing, dead code;
- `[must_use]` results discarded;
- needless allocation in `[noalloc]`-eligible code; suggesting `Slice` over copy;
- redundant `claim` where `borrow` suffices; un-needed `clone`;
- capability tightening hints ("this fn could be `[pure]`").
Lints are configurable per-project (`deny`/`warn`/`allow`) and the rule set is pluggable (lints can
be written as comptime/proc-macro analyzers).

---

## 7. Language server — `jolt lsp`

Same `libjolt` front end, so completions/types/errors match the compiler exactly. Features:
- completions, hover types, go-to-def, find-refs, rename, signature help;
- inline Custodian/capability diagnostics as you type (incremental);
- inlay hints (inferred types, inferred lifetimes from §9.4, allocator in scope);
- code actions (apply suggested fixes, derive a contract, add `[must_use]`);
- format-on-save, semantic highlighting.
Editor-agnostic (LSP protocol): VS Code, Neovim, JetBrains, etc.

---

## 8. Debugger — `jolt debug`

- Emits DWARF/CodeView; works with GDB/LLDB and the VS Code debug protocol.
- **Jolt-aware pretty-printers** for stdlib types (`Array`, `Map`, `String`, `Option`, `Shared`).
- Visualizes **ownership/borrow state** at a breakpoint (who owns/claims what — the Custodian's view
  at runtime).
- Async/fiber-aware stack unwinding for Tier-2 fibers; per-task views.
- Embedded: remote debugging over JTAG/SWD/probe-rs-style backends; semihosting capture.

---

## 9. Test & benchmark runner

Extensive built-in testing — full design in **`jolt-testing.md`**.

- `jolt test`: discovers `[test]`/`[fuzz]`/`[simulation]` functions + doctests, runs them in parallel
  (isolated), with filtering (`--tag`/`--exclude`/`[only]`), `should_fail`, fixtures, and
  **macro-based assertions** with structured diffs + soft `expect_*`.
- **Property testing** (`for_all` + shrinking), **fuzzing** (`jolt fuzz`, crashes → regression tests),
  **snapshot** (`--update-snapshots`), **mutation testing** (`--mutate`, mutation score).
- **Deterministic Simulation Testing** (`[simulation]`): whole-system runs on a virtual clock with a
  seed-driven scheduler + fault injection; failures replay exactly (`--seed N --trace`).
- **Sanitizers** (`--sanitize=address,thread,…`) backstop `[unsafe]`/FFI code.
- **Cache-aware:** unchanged tests show "cached pass"; **permission-sandboxed** per test;
  `--coverage`, `--watch`, `--format json`/JUnit; `[nostd]` on-device/QEMU harness.
- `jolt bench`: statistical benchmarking with baseline regression tracking; integrates with PGO.

---

## 10. Documentation — `jolt doc`

- Generates browsable HTML from declarations + `///` doc comments (markdown).
- **Runs doctests** as part of `jolt test` so examples never rot.
- Renders contract conformances, capability attributes (so users see at a glance that a fn is
  `[pure]`/`[noalloc]`), and cross-links types.
- Doc coverage metric; `--check` fails CI on undocumented public items.

---

## 11. Targets & cross-compilation

- **Target triples** like `x86_64-linux-gnu`, `aarch64-macos`, `thumbv7em-none-eabi`,
  `riscv32-none-elf`, `wasm32-unknown`.
- `jolt target add <triple>` fetches the matching prebuilt stdlib (or builds it).
- **Cross-compile is first-class:** `jolt build --target thumbv7em-none-eabi` just works; no separate
  cross toolchain to assemble.
- **`[cfg: …]` resolution** keys off the target: `arch`, `os`, `endian`, `pointer_width`, `feature`,
  `has_std`, `has_alloc`. The stdlib ships in `full` / `nostd+alloc` / `nostd+noalloc` flavors and the
  right one is selected automatically.
- Built-in `jolt objcopy`/`jolt size`-style helpers for firmware images (bin/hex/elf).

---

## 12. Toolchain management

- `joltup` (a small bootstrap installer): installs/updates `jolt` releases, manages multiple
  versions, and per-project pinning via a `jolt-version` file (so a repo builds with the version it
  was written for).
- Stable / beta / nightly channels; `edition` in `jolt.toml` gates language changes so upgrades
  don't break old code.

---

## 13. Security & permissions (Deno-style)

**Deny-by-default, explicitly-granted** permissions spanning runtime, CLI, and the build sandbox —
unified with the language's capability attributes. Full design: **`jolt-security-model.md`**.

- **No grants = pure compute.** `jolt run` with no flags can compute/allocate but any `Fs`/`Net`/`Os`
  call returns `PermissionDenied` (a value, not a panic).
- **Grants:** `--allow-read[=paths]`, `--allow-write`, `--allow-net[=hosts]`, `--allow-env`,
  `--allow-run`, `--allow-ffi`, `--allow-sys`, `--allow-hrtime`, `--allow-all`; `--deny-*` overrides.
  `--prompt` opts into interactive prompting.
- **Static proof removes runtime checks:** a `[noio]`/`[nonet]` function is compiler-proven and
  policed for free — no flags needed, no checks inserted. Granting a permission a binary statically
  forbids is a no-op + warning.
- **Build sandbox:** `build.jolt`/`comptime` run with no io/net by default; codegen that fetches or
  reads outside the project needs `--allow-build-net`/`--allow-build-read` *and* a `[build_io]`
  attribute — closing the supply-chain hole.
- **Dependency policy:** `jolt.toml [permissions]` can cap the whole dependency tree (`deny = ["ffi",
  "run"]`); the compiler rejects a dep whose capability surface exceeds the policy. `jolt audit`
  surfaces capability creep.

---

## How it all hangs together

```
                ┌──────────── libjolt (front end: lex→parse→type→Custodian→capability) ────────────┐
                │                                                                                   │
  jolt fmt ─────┤   jolt lsp ─────┤   jolt lint ─────┤   jolt check ─────┐                          │
                │                                                        │                          │
                └────────────────────────────────────────┬─────────────┘                          │
                                                          ▼                                         │
                              monomorphize → MIR → backend (fast | LLVM) → link ────────────────────┘
                                                          │
   jolt build / run / test / bench / doc / debug  ◄───────┘     jolt (pkg mgr + build system + targets)
```

One front end feeds every tool; one driver exposes every workflow; the package manager, build system,
and target manager wrap the whole thing. The language's distinctive features — the Custodian,
capability attributes, the two I/O tiers, comptime, `[nostd]` — are not just compiler concerns; they
are surfaced and enforced at every layer of the toolchain.

---

## Open questions for the toolchain

1. **Backend commitment.** LLVM-primary + fast-debug-backend is the recommended split; confirm, or go
   LLVM-only first (simpler) / in-house-only (more control, much more work).
2. **comptime in `build.jolt`** — how sandboxed is build-time code (can it do arbitrary I/O)?
   Reproducibility argues for a restricted capability set (`[noio]`-ish) by default.
3. **Registry governance** — central vs federated; namespacing; who runs `pkg.jolt.dev`.
4. **Stable ABI?** Decide whether Jolt commits to a stable ABI for dynamic linking, or stays
   recompile-from-source (simpler, like Rust today). Affects plugins and OS distribution.
5. **Sanitizers vs the Custodian** — define exactly what runtime checking adds for `[unsafe]` code
   that the compile-time Custodian cannot cover.
