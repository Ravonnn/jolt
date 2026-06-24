# Jolt — Project Index

> **Jolt** is a general-purpose, statically-typed, low-level systems language. This is the front
> door to the design corpus under `docs/`.

---

## Canonical documents

| Doc | What it covers |
| --- | -------------- |
| [`spec/jolt-spec-v0.4.md`](spec/jolt-spec-v0.4.md) | **Language specification** (authoritative) |
| [`tutorial/jolt-tour.md`](tutorial/jolt-tour.md) | **A Tour of Jolt** — start here to learn |
| [`learn/hub.md`](learn/hub.md) | **Jolt Learn** — interactive tutorials (local execution) |
| [`spec/jolt-cheatsheet.md`](spec/jolt-cheatsheet.md) | One-page quick reference |
| [`spec/jolt-grammar.md`](spec/jolt-grammar.md) | Formal EBNF grammar |
| [`design/jolt-decisions.md`](design/jolt-decisions.md) | Design decisions log |
| [`design/jolt-stdlib-outline.md`](design/jolt-stdlib-outline.md) | Standard library outline |
| [`design/jolt-toolchain.md`](design/jolt-toolchain.md) | Core toolchain |
| [`design/jolt-toolchain-extended.md`](design/jolt-toolchain-extended.md) | Extended toolchain |
| [`design/jolt-build-system.md`](design/jolt-build-system.md) | `build.jolt` & `jolt.toml` |
| [`design/jolt-caching-system.md`](design/jolt-caching-system.md) | Incremental cache |
| [`design/jolt-testing.md`](design/jolt-testing.md) | Testing system |
| [`design/jolt-security-model.md`](design/jolt-security-model.md) | Permissions & security |
| [`design/jolt-safety-attributes.md`](design/jolt-safety-attributes.md) | Capability attributes |
| [`design/jolt-compiletime-safety.md`](design/jolt-compiletime-safety.md) | Compile-time safety |
| [`design/compiler-status.md`](design/compiler-status.md) | **Compiler build status** (living doc) |
| [`design/jolt-0.1-preview.md`](design/jolt-0.1-preview.md) | **0.1 preview** — what works / what doesn't |
| [`design/jolt-implementation-plan.md`](design/jolt-implementation-plan.md) | Compiler build plan |
| [`design/00-repo-structure.md`](design/00-repo-structure.md) | Repository layout |
| [`design/year-1.md`](design/year-1.md) … [`year-5.md`](design/year-5.md) | Phased roadmap |

---

## Reading paths

**Learn the language:** [`learn/hub.md`](learn/hub.md) → `tutorial/jolt-tour.md` → `spec/jolt-cheatsheet.md` → `spec/jolt-spec-v0.4.md`

**Build the compiler:** [`design/compiler-status.md`](design/compiler-status.md) →
`spec/jolt-grammar.md` → `spec/jolt-spec-v0.4.md` → `design/jolt-toolchain.md`
→ `design/jolt-caching-system.md`

**Implementation status:** Phase 0 complete; Phase 1a–1f complete; Phase 2a–2c complete; Phase 3 complete (**Jolt 0.1 preview**). See [`design/jolt-0.1-preview.md`](design/jolt-0.1-preview.md).
See [`design/compiler-status.md`](design/compiler-status.md) for details and
[`design/year-1.md`](design/year-1.md) for the next gate.

---

## Reading the docs locally (mdBook)

Install [mdBook](https://rust-lang.github.io/mdBook/) and preview the full corpus in a browser:

```bash
cargo install mdbook --locked   # once
mdbook serve docs --open        # http://localhost:3000
```

Or `cargo xtask docs serve`. See [`README.md`](README.md) in this directory.

## ADRs

New implementation decisions: [`adr/README.md`](adr/README.md)
