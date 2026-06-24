# Repository Structure

> Set this up in Phase 0. A single monorepo: Rust stage-0 compiler + tools, the Jolt-written stdlib,
> the design docs, and test corpora. Layout mirrors the `libjolt` architecture
> (`jolt-implementation-plan.md` §1) so crates map 1:1 to compiler stages.

```
jolt/
├── README.md                      # project intro, build instructions
├── LICENSE
├── CONTRIBUTING.md                # contributor guide, coding standards
├── CODE_OF_CONDUCT.md
├── SECURITY.md                    # vulnerability disclosure policy
├── CHANGELOG.md
├── rust-toolchain.toml            # pinned Rust version for stage-0
├── Cargo.toml                     # Rust workspace root
├── .gitignore
├── .github/
│   └── workflows/
│       ├── ci.yml                 # build + test on every PR
│       ├── nightly.yml            # nightly artifacts
│       └── release.yml
│
├── docs/                          # the design corpus (current .md files) + generated docs
│   ├── index.md
│   ├── spec/                      # jolt-spec, grammar, cheatsheet
│   ├── design/                    # decisions, security, caching, testing, toolchain, ...
│   ├── tutorial/                  # the tour, guides
│   └── adr/                       # Architecture Decision Records (new decisions during build)
│
├── compiler/                      # STAGE-0 compiler — the `libjolt` crates (Rust)
│   ├── jolt-source/               # SourceMap, FileId, Span, content hashing
│   ├── jolt-lexer/                # tokens; max-munch rules
│   ├── jolt-ast/                  # AST types + visitor
│   ├── jolt-parser/               # AST construction; error-recovery productions
│   ├── jolt-query/                # query engine: memoization, dep-tracking, red/green
│   ├── jolt-resolve/              # names, modules, imports, visibility
│   ├── jolt-types/                # type representation, inference, contract resolution
│   ├── jolt-custody/              # THE CUSTODIAN — ownership/move/borrow analysis
│   ├── jolt-capability/           # [noalloc]/[noio]/... transitive checking
│   ├── jolt-comptime/             # MIR interpreter (comptime + runtime interp + REPL)
│   ├── jolt-mono/                 # monomorphization; value-generic + comptime-guard resolution
│   ├── jolt-mir/                  # mid-level IR + lowering
│   ├── jolt-backend-interp/       # MIR interpreter backend (first)
│   ├── jolt-backend-llvm/         # LLVM codegen (second)
│   ├── jolt-backend-fast/         # in-house fast debug backend (later)
│   ├── jolt-diagnostics/          # structured errors, codes, suggestions, JSON
│   ├── jolt-cache/                # content-addressed store + query-graph persistence
│   └── libjolt/                   # umbrella crate exposing the front-end/back-end API
│
├── driver/
│   └── jolt-cli/                  # the `jolt` binary: build/run/test/fmt/... verbs
│
├── tools/                         # toolchain (Rust until self-host)
│   ├── jolt-fmt/
│   ├── jolt-lsp/
│   ├── jolt-test/                 # test runner
│   ├── jolt-doc/
│   ├── jolt-lint/
│   ├── jolt-pkg/                  # package manager + resolver
│   ├── jolt-profile/
│   ├── jolt-coverage/
│   ├── jolt-verify/
│   ├── jolt-bindgen/
│   └── joltup/                    # toolchain version manager (bootstrap installer)
│
├── stdlib/                        # the standard library, WRITTEN IN JOLT
│   ├── core/                      # foundational contracts + intrinsics surface
│   ├── std/                       # prelude
│   ├── mem/  collections/  text/  math/  iter/
│   ├── io/  fiber/  concurrent/  time/  fs/  net/  os/
│   ├── encoding/  crypto/  test/  log/  reflect/
│   └── lowlevel/                  # layout, ptr, volatile, bits, abi, intrin, simd, arch,
│                                  #   interrupt, boot, embed, inline
│
├── selfhost/                      # the Jolt-written compiler (Phase 14+); empty until then
│
├── tests/                         # compiler/toolchain test corpora
│   ├── ui/                        # parse/type/resolve: source + expected diagnostics (snapshot)
│   ├── custody/                   # Custodian conformance: should-accept / should-reject
│   ├── run/                       # programs + expected stdout (differential interp vs native)
│   ├── capability/                # capability/permission enforcement cases
│   ├── fuzz/                      # fuzz targets + corpora
│   ├── property/                  # property tests (fmt idempotence, parse/print roundtrip)
│   ├── sim/                       # deterministic-simulation tests of concurrent code
│   └── bench/                     # benchmarks with baselines
│
├── examples/                      # example Jolt programs (also smoke-tested)
├── registry/                      # package-registry server + index (Phase 10)
├── playground/                    # WASM-based web playground (Phase 15)
└── xtask/                         # build automation (bootstrap, three-stage, release)
```

### Conventions
- **One crate per compiler stage** so dependencies are explicit and stages are independently testable.
- **`libjolt` is the only public surface** the CLI/tools depend on — never reach into a stage crate
  directly. This is what keeps every tool consistent with the compiler.
- **Stdlib is Jolt source from the start** (only a minimal intrinsic surface lives in Rust), so the
  stdlib dogfoods the language continuously and ports cleanly at self-host.
- **ADRs** (`docs/adr/`) record any *new* decision made during implementation, complementing the
  pre-existing design decisions log.
- **Tests live beside the corpus they validate** and are the gate currency for every phase.
