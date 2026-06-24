# Jolt — Security & Permissions Model

> Deno-style **deny-by-default, explicitly-granted** permissions, unified with Jolt's existing
> capability attributes. One model spans four layers: the **compiler** (static `[noio]`-style
> capabilities), the **runtime** (granted permissions enforced at syscall boundaries), the **CLI**
> (`--allow-*` grants), and the **build/`comptime` sandbox** (the same grants for build-time code).
>
> The insight: Jolt already has compile-time capability attributes (`[noio]`, `[noalloc]`, …) and
> package capability transparency. This adds the *runtime* and *CLI* half so that what a program is
> *allowed* to do is governed end-to-end, and a program proven `[noio]` at compile time needs no
> runtime check at all.

---

## Principles

1. **Deny by default.** A program gets no I/O, network, filesystem, env, FFI, or subprocess access
   unless granted. Like Deno, nothing ambient is trusted.
2. **Two enforcement tiers, one vocabulary.**
   - **Static** (compile time): capability attributes (`[noio]`, `[nonet]`, …) *prove* code can't do
     a thing → zero runtime cost, can't be bypassed.
   - **Dynamic** (runtime): granted permissions checked at the syscall boundary for code that *isn't*
     statically constrained.
3. **Compile-time proof removes runtime checks.** If `@f` is `[noio]`, the runtime never checks I/O
   for it — the attribute already guarantees it. Static capabilities and runtime permissions are the
   same concepts at different times.
4. **Permissions are values, scoped and revocable.** A grant can be narrowed (a path prefix, a host)
   and dropped; child fibers/threads/subprocesses inherit a subset, never a superset.
5. **The build system is a program too.** `build.jolt`/`comptime` code runs under the *same* model —
   build scripts can't silently phone home or read your SSH keys.

---

## Permission domains

Mirrors Deno's surface, mapped to Jolt's stdlib modules and capability attributes:

| Permission | Grants access to | Static attribute (denies it) | Stdlib gated |
| ---------- | ---------------- | ---------------------------- | ------------ |
| `read[=paths]` | filesystem reads | `[noread]` | `Fs` |
| `write[=paths]` | filesystem writes | `[nowrite]` | `Fs` |
| `net[=hosts]` | network / sockets | `[nonet]` | `Net` |
| `env[=names]` | environment vars | `[noenv]` | `Os.env` |
| `run[=cmds]` | subprocesses | `[norun]` | `Os.Process` |
| `ffi[=libs]` | C/foreign calls, `dyn` lib loading | `[noffi]` | `Abi` |
| `sys[=apis]` | system info (hostname, cpu, …) | `[nosys]` | `Os` |
| `hrtime` | high-resolution timers (side-channel risk) | `[nohrtime]` | `Time` |
| `unsafe` | raw pointers / `[unsafe]` at all | (compile-gated already) | `Ptr`, `Arch` |

`io` (the umbrella) = `read + write + net`. `all` grants everything (discouraged). `[noio]` (existing)
denies the umbrella statically.

---

## CLI grants (Deno-style)

```
jolt run                                   # NO permissions — pure compute only
jolt run --allow-read                      # all reads
jolt run --allow-read=./data,./cfg         # scoped to paths
jolt run --allow-net=api.example.com:443   # scoped to a host:port
jolt run --allow-env=HOME,PATH             # scoped env vars
jolt run --allow-run=git,ls                # scoped subprocesses
jolt run --allow-ffi=./libfoo.so           # scoped FFI
jolt run --allow-all                       # everything (prints a warning)
jolt run --deny-net                        # explicit deny overrides any allow
```

- **Prompting (interactive, opt-in):** the default is **hard-fail** — an ungranted access returns
  `PermissionDenied` (safe for CI). Passing `--prompt` opts into Deno-style runtime prompts:
  "myapp wants to read ./secrets — allow? [y/n/always]".
- **`--deny-*` beats `--allow-*`** for hard exclusions.
- **No flags = no permissions:** a Jolt program with no grants can still compute, allocate, and use
  pure stdlib, but any `Fs`/`Net`/`Os` call traps with a clear `PermissionDenied` error (a value, per
  the error model — not a panic).

---

## Static + dynamic interplay

A function's declared capabilities *shrink* what the runtime even has to police:

```jolt
[noio]                              // proven: no read/write/net anywhere in here or its callees
@checksum(data: Slice<Byte>) Uint64 -> ... ;;   // runs with zero permission checks, always

[nonet, noread]                     // may write logs, but never reads files or touches the network
@render(out: Fs.File) None -> ... ;;
```

- A `[noio]` program needs **no `--allow-*` flags at all** and the runtime inserts **no checks** —
  the compiler already proved it safe. This is the big win over pure-runtime models like Deno: much
  of the enforcement is *free and unbypassable*.
- For code that *isn't* statically constrained, the runtime checks each gated stdlib call against the
  active grant set and returns `PermissionDenied` (or prompts) if missing.
- **Capability attributes and CLI grants must be consistent:** you can't grant `--allow-net` to a
  binary whose `@main` is `[nonet]` — the compiler already forbade network code, so the flag is a
  no-op (a warning is emitted).

---

## Runtime permission API

Permissions are first-class values so programs can introspect, narrow, and drop them.

```jolt
using Permission;

// query
if Permission.query(net("api.example.com")) == Granted -> ... ;;

// voluntarily drop (defense in depth): after setup, a server can shed read/write
Permission.revoke(read());
Permission.revoke(write());

// run a closure with a narrowed subset (child can't exceed this)
Permission.scoped([net("api.example.com:443")], || ->
    fetch("https://api.example.com/data")      // ok
    // fetch("https://evil.com")               // PermissionDenied
;;);
```

Grants are **monotonically narrowing** everywhere — a project's `jolt.toml [permissions]` sets the
ceiling, CLI flags may only tighten it (never widen), and at runtime a program can drop or scope down
permissions but never *raise* them. Spawned fibers/threads inherit the current (possibly narrowed)
set; subprocesses inherit only what's explicitly passed.

---

## Build-system & `comptime` sandbox

The most distinctive part: **build-time code is governed by the same model**, closing the supply-chain
hole where a dependency's build script exfiltrates data or fetches arbitrary payloads.

```
jolt build                                 # build.jolt + comptime run with NO io/net by default
jolt build --allow-build-net=registry.jolt.dev   # e.g. a codegen tool that fetches a schema
jolt build --allow-build-read=./schema     # declared codegen inputs
```

- `build.jolt` and `comptime` blocks default to **`[noio]`** (the §"build sandbox" from the build
  doc). They may read/write only paths declared as build inputs/outputs.
- Anything more (network fetch in a `generate` step, reading outside the project) needs an explicit
  `--allow-build-*` grant **and** a `[build_io]` attribute on the offending build code — so it's
  visible in the source *and* requires operator consent.
- **Reproducibility benefit:** a default build can't depend on hidden network state.

---

## Package-manager integration (capability transparency)

Ties into the package manager's existing capability transparency:

- Every package declares (and the compiler verifies) the capabilities it needs:
  `[requires: net, read]` in its manifest, derived from its code's actual capability surface.
- `jolt add foo` shows foo's capability footprint before adding; `jolt audit` flags dependencies that
  request more than the project allows.
- A project can **cap its whole dependency tree**: `jolt.toml` →
  ```toml
  [permissions]
  deny = ["ffi", "run"]            # no dependency may use FFI or spawn processes
  allow-net = ["api.example.com"]  # transitive net limited to this host
  ```
  The compiler rejects a dependency whose capability surface exceeds the project's policy — a
  supply-chain guard enforced at build time, not just runtime.
- A `[noalloc]`/`[nostd]` project already refuses allocating/std-using deps (existing behavior); this
  extends the same mechanism to io/net/ffi.

---

## Threat model & limits

- **Covers:** accidental over-reach, supply-chain (build + dependency capability creep), least-
  privilege deployment, side-channel-sensitive code (`[constanttime]`, `hrtime` gating).
- **Static guarantees are unbypassable** (they're compiler-proven); **runtime grants are
  enforcement, not sandboxing** — they stop gated stdlib calls but a program with `unsafe` + raw
  syscalls (granted `--allow-ffi`/`unsafe`) can do anything. Hard isolation still needs OS sandboxing
  (namespaces, seccomp) — Jolt makes that *easier* by minimizing what needs granting.
- **`[unsafe]` is itself a permission:** raw syscalls/pointers require the `unsafe` grant (or a
  compile-time `[unsafe]` audit), so a "safe" deployment can forbid them entirely.

---

## How it unifies with what already exists

| Existing piece | Role in the security model |
| -------------- | -------------------------- |
| capability attributes (`[noio]`, `[nonet]`, …) | **static** half — compile-time proof, zero-cost |
| compile-time type validation / monomorphized generics / typed config | eliminates runtime type-confusion as attack surface (see `jolt-compiletime-safety.md`) |
| errors-as-values (`!T`) | `PermissionDenied` is a normal error, not a panic |
| `Dispose`/`scoped` | permission scoping/auto-revocation reuse the same RAII machinery |
| build `comptime` sandbox | build-time enforcement point |
| package capability transparency | dependency-tree policy enforcement |
| `[constanttime]`/`[secret]` | side-channel & data-flow guards complement permission gating |

Nothing here is a bolt-on: it's the runtime + CLI projection of capabilities Jolt already tracks at
compile time.

---

## Resolved decisions

1. **Prompt vs hard-fail default** — ✅ **hard-fail by default** (denied access returns
   `PermissionDenied`); interactive prompting is opt-in via `--prompt`. CI and non-interactive runs
   are safe by default; humans can opt into the Deno-style prompt flow.
2. **Granularity of `net`/`read` scoping** — ✅ Deno-style: `net` scopes by **host** and optional
   **`host:port`**; `read`/`write` scope by **path prefix** (a granted directory covers its subtree).
   Both **DNS names and IPs** are honored for `net` (a hostname grant also matches its resolved IPs at
   connect time, checked against the original grant). Prefix matching is path-segment-aware (no
   partial-segment matches); explicit globs allowed (`--allow-read=./data/**`).
3. **Static↔grant consistency** — ✅ **warning, not error.** Granting `--allow-net` to a `[nonet]`
   binary compiles and runs; the grant is a no-op and the CLI emits a warning ("`--allow-net` ignored:
   target is statically `[nonet]`"). Keeps scripts/CI robust when flags are passed broadly.
4. **Build-grant inheritance** — ✅ **per-package, deny-by-default.** `--allow-build-*` grants apply
   only to the **root** package's build code; each dependency's build script must be granted
   explicitly (e.g. `--allow-build-net=pkg:foo`), so a transitive dependency can't inherit the root's
   build permissions. Contains supply-chain risk at the build layer.
5. **Permission manifest vs flags** — ✅ **yes, bake defaults into `jolt.toml`**; CLI flags may only
   **narrow** them, never widen. A project declares its default grant set in `[permissions]` so
   `jolt run` needs no flags, and `--deny-*`/scoped `--allow-*` flags can only tighten that set at run
   time — a user can never grant more than the manifest permits without editing the manifest.

```toml
# jolt.toml — baked-in default grants (flags may only narrow these)
[permissions]
allow-read  = ["./data"]
allow-net   = ["api.example.com:443"]
deny        = ["ffi", "run"]      # also caps the dependency tree
```
