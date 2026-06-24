# Jolt toolchain (stage-0)

Rust-hosted tools until self-host (Phase 14). Phase 0 includes stub binaries for `jolt-fmt` and
`jolt-test`; other tools are listed for layout alignment with `00-repo-structure.md`.

| Tool | Status |
| ---- | ------ |
| `jolt-fmt-bin` | `jolt-fmt` binary (format via `libjolt`) |
| `jolt-test` | Stub workspace crate |
| `jolt-lsp` | Planned (Phase 4+) |
| `jolt-doc` | Planned |
| `jolt-lint` | Planned |
| `jolt-pkg` | Planned (Phase 10) |
| `jolt-profile` | Planned |
| `jolt-coverage` | Planned |
| `jolt-verify` | Planned |
| `jolt-bindgen` | Planned |
| `joltup` | Planned |

All tools must depend on `libjolt` only.
