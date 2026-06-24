# Jolt — Standard Library Outline (v0.1)

> A rich, batteries-included stdlib. Organized as a module tree; each module lists its key types,
> functions, and the contracts it provides or relies on. Signatures use Jolt syntax (`@name(params)
> Ret`, `!T` = fallible, `T?` = optional). This is a *surface* outline — semantics are sketched, not
> fully specified.
>
> Conventions: everything in **`Core`** + **`Prelude`** is auto-imported. Other modules need
> `using`. Functions that can fail return `!T`; functions that may have no result return `T?`
> (`Some`/`Nothing`). Memory follows the Custodian (`borrow`/`claim`/`deref`); allocation respects
> the in-scope allocator.

---

## Module tree

```
Core          intrinsics, primitive ops, the foundational contracts   (auto)
Prelude       the names every program gets for free                   (auto)
Mem           allocators, raw memory, smart pointers
Collections   Array, Map, Set, List, Deque, Heap, ...
Text          String, Char, formatting, parsing, regex, unicode
Math          numeric functions, Complex, Rational, BigInt, random, linear algebra
Io            Tier-1 completion-based I/O (zero runtime)
Fiber         Tier-2 green-threaded I/O + runtime
Concurrent    threads, scopes, channels, locks, atomics
Time          instants, durations, clocks, calendars
Fs            files, directories, paths, metadata
Net           TCP/UDP/TLS, sockets, addresses
Os            process, env, args, signals, syscalls
Encoding      JSON, bytes, base64, hex, CSV, serialization framework
Crypto        hashing, AEAD, signatures, constant-time primitives
Iter          iterator adapters & helpers
Test          test harness, assertions, benchmarking, property testing
Log           structured logging
Reflect       comptime + runtime reflection on top of `typeinfo`

# ── low-level / systems layer (see §"Low-level programming") ──
Volatile      volatile + MMIO register access
Atomic        (in Concurrent) lock-free primitives, fences, memory ordering
Bits          bit fields, bitsets, packing, endianness
Layout        struct layout control, alignment, padding, repr
Ptr           typed/untyped raw pointers, pointer arithmetic, spans
Abi           C ABI, calling conventions, extern, varargs, struct-by-value
Intrin        CPU intrinsics, SIMD, prefetch, cache control
Arch          per-architecture: registers, special instructions, CPUID
Interrupt     IRQ handlers, NMI, exception frames, masking
Boot          entry points, linker sections, no-std runtime shims
Simd          portable SIMD vector types and ops
Embed         embedded helpers: GPIO/SPI/I2C/UART traits, no-alloc patterns
Inline        inline asm DSL (typed operands) beyond raw `asm`
```

---

## Core (auto-imported)

> **Low-level affordances throughout.** Most application-level modules below carry a **"Low-level
> extensions"** note: the same module that gives you ergonomic high-level APIs also exposes the
> systems-grade surface (no-alloc/fixed-capacity variants, `Span`/raw access, `[noalloc]`/`[noblock]`
> paths, allocator-explicit constructors, hardware acceleration). You do not switch libraries to go
> low-level — you reach further down the *same* module. The dedicated systems modules (`Layout`,
> `Ptr`, `Volatile`, …) cover what has no high-level equivalent.

Foundational contracts (auto-derived where possible):

| Contract | Method(s) | Meaning |
| -------- | --------- | ------- |
| `Copy` | — (marker) | value is implicitly copied, not moved |
| `Dispose` | `@dispose($$self)` | deterministic destruction (RAII) |
| `Clone` | `@clone(self) Self` | explicit deep copy for non-`Copy` types |
| `Equals` | `@(==)(self, o: Self) Bool` | equality |
| `Comparable` | `@compare(self, o: Self) Int` | ordering (`<,>,<=,>=` derive from this) |
| `Hash` | `@hash(self, h: $$Hasher) None` | hashable (for `Map`/`Set` keys) |
| `Display` | `@show(self) String` | human-readable; powers interpolation & `to_string` |
| `Debug` | `@debug(self) String` | developer-facing representation |
| `Default` | `@default() Self` | a sensible zero/empty value |
| `Plus`/`Minus`/`Times`/`Over`/`Mod`/`Power` | `@(+)` … | arithmetic operator overloading |
| `Iterator` | `@next($$self) Item?` | produces values until `Nothing` |
| `Iterable` | `@iter(self) dyn Iterator` | can be looped with `for` |
| `Sendable` / `Shareable` | — (marker) | thread-transfer / thread-share safety |
| `From<U>` / `Into<U>` | `@from(u: U) Self` / `@into(self) U` | conversions |

Intrinsics: numeric overflow helpers (`wrapping_add`, `checked_add`→`T?`, `saturating_add`),
`size_of |T|()`, `align_of |T|()`, `swap($$a, $$b)`, `move(x)`, `copy(x)` (for `Copy`), `drop(x)`.

---

## Prelude (auto-imported names)

`print`, `println`, `eprintln`, `format("{}…", …) String`, `to_string(x) String`,
`assert(cond, msg?)`, `panic(msg)`, `todo()`, `len(c)`, `range(a, b)`, `min`, `max`, `abs`,
`clamp`, `Some`/`Nothing`/`Ok`/`Err`, and the Core contracts above.

---

## Mem

- **Allocators:** `Allocator` contract (`@alloc`, `@free`, `@realloc`); concrete: `System`,
  `Arena`, `Pool`, `Bump`, `Tracking`, `Null` (for `[noalloc]` testing).
- **Smart pointers / ownership:** `Box<T>` (heap-owned), `Shared<T>` / `SharedSync<T>` (counted,
  from `[shared]`), `Weak<T>`, `Cell<T>` / `RefCell<T>` (interior mutability, runtime-checked).
- **Raw:** `Pointer<T>`, `Slice<T>`, `Raw`; `memcpy`, `memset`, `memcmp` (in `[unsafe]`).

```jolt
using Mem;
$arena = Arena::new();
[alloc: arena] @build() !Tree -> ... ;;     // everything here uses the arena
```

**Low-level extensions.** `Allocator` gains `@alloc_aligned(size, align)`, `@alloc_zeroed`, and a
`@grow_in_place` hook for realloc-free growth. Fallible variants `try_alloc → Pointer<None>?` for
`[nopanic]` paths (no trap on OOM). `FixedBuffer(static_region)` and `StackAllocator` let `[noalloc]`
code allocate from a caller-supplied byte region. `Box::new_in(a, v)` / `Shared::new_in(a, v)` place
smart pointers in a chosen allocator. `forget(x)` suppresses `Dispose`; `take($$slot) T` /
`replace($$slot, v) T` move without dropping. `assume_init|T|()` and `MaybeUninit<T>` support
manual, staged initialization for `[unsafe]` hot paths.

---

## Collections

Generic, allocator-aware containers.

| Type | Notes |
| ---- | ----- |
| `Array<T>` / `Array<T, N>` | growable / fixed-size; `push`, `pop`, `get`, `[]`, `slice`, `sort`, `map` |
| `List<T>` | doubly-linked list |
| `Deque<T>` | double-ended queue (ring buffer) |
| `Map<K: Hash, V>` | hash map; `insert`, `get`→`V?`, `remove`, `entry` |
| `OrderedMap<K: Comparable, V>` | tree/btree map, sorted iteration |
| `Set<K: Hash>` / `OrderedSet<K>` | hash / sorted set |
| `Heap<T: Comparable>` | binary heap / priority queue |
| `Stack<T>` / `Queue<T>` | thin wrappers over `Array`/`Deque` |

Common surface (each conforms to `Iterable`, `Clone` if `T: Clone`, `Dispose`):
`len`, `is_empty`, `contains`, `clear`, `iter`, `map`, `filter`, `fold`, `reduce`.

```jolt
using Collections;
$$scores: Map<String, Int> = Map::new();
scores.insert("a", 10);
match scores.get("a") -> Some(v) -> println("{v}"); Nothing -> println("none"); ;;
```

**Low-level extensions.**
- **No-alloc / fixed-capacity variants** (mirror the `Embed` containers, usable anywhere):
  `StaticVec<T, N>`, `StaticMap<K, V, N>`, `RingBuffer<T, N>`, `InlineString<N>` — all stack/static,
  `[noalloc]`-safe, never grow.
- **Capacity control on growable types:** `with_capacity(n)`, `reserve(n)`, `shrink_to_fit`,
  `capacity()`, and fallible `try_push → !None` (returns an error instead of trapping on OOM, for
  `[nopanic]`).
- **Raw access:** `as_span() Span<T>` / `as_span_mut()` to hand contiguous storage to FFI/DMA;
  `from_raw_parts(ptr, len, cap)` (`[unsafe]`); `set_len(n)` (`[unsafe]`) for manual init.
- **Layout guarantees:** `Array<T>` is documented as pointer+len+cap with `[repr: C]`-compatible
  layout so it can cross the ABI; `Array<T, N>` is exactly `N * size_of<T>()` contiguous bytes.
- **Allocator-explicit constructors:** `Array::new_in(a)`, `Map::new_in(a)` alongside the
  in-scope-allocator defaults.

---

## Text

- `String` (UTF-8, owned), `StringView` (borrowed slice), `Char` (32-bit scalar).
- Methods: `len_bytes`, `len_chars`, `chars()`, `bytes()`, `split`, `trim`, `replace`,
  `starts_with`, `to_upper`, `to_lower`, `find`→`Uint?`, `[a..b]`.
- **Formatting:** `format`, `Display`/`Debug`; interpolation `"{x}"`, `"{x:hex}"`, `"{f:.2}"`
  (format specifiers) — escaping `{{`/`}}`.
- **Parsing:** `parse |T| (s: String) !T` (e.g. `parse<Int>("42")`).
- **Regex:** `Regex::compile(pat) !Regex`, `match`, `find_all`, `replace`, `captures`.
- **Unicode:** normalization, grapheme iteration, case folding, width.

**Low-level extensions.** `InlineString<N>` (stack-allocated, `[noalloc]`); `as_bytes() Span<Byte>`
and `from_utf8(span) !String` / `from_utf8_unchecked` (`[unsafe]`) for zero-copy byte↔text;
`CStr`/`CString` interop (→ `Abi`) for NUL-terminated C strings; ASCII fast paths
(`as_ascii`, `eq_ignore_ascii_case`) that skip Unicode machinery; a `[noalloc]` `format_into(buf, …)`
that writes into a caller `Span<Byte>` instead of allocating; byte-level `find`/`split` on raw
`Span<Byte>` for parsers that predate decoding.

---

## Math

- Functions: `sqrt`, `cbrt`, `pow`, `exp`, `log`, `log2`, `log10`, `sin`/`cos`/`tan` (+inverse/hyper),
  `floor`, `ceil`, `round`, `trunc`, `hypot`, `gcd`, `lcm`.
- Constants: `PI`, `E`, `TAU`, `INF`, `NAN`, per-type `MIN`/`MAX`/`EPSILON`.
- Types: `Complex<T>`, `Rational<T>`, `BigInt`, `BigFloat`, `Decimal` (base-10 fixed).
- `Random`: PRNG (`next`, `range`, `shuffle`, `choice`), seedable, plus `SecureRandom` (→ `Crypto`).
- `LinAlg` (submodule): `Vec<T, N>`, `Mat<T, R, C>` (value-generic dims), dot/cross/transpose/inverse.

**Low-level extensions.** Fixed-point types `Fixed<Bits, Frac>` for FPU-less targets; saturating /
wrapping / checked arithmetic surfaced as methods (`a.sat_add(b)`, `a.wrap_mul(b)`,
`a.checked_div(b) → T?`); bit-exact float helpers (`to_bits`/`from_bits`, `next_after`, `is_nan`
without FP ops); soft-float fallbacks for `[nostd]` no-FPU builds; `LinAlg` vectors map onto `Simd`
hardware vectors where available; all core functions have `[pure]` (and FPU-free ones `[noalloc]`)
variants so they're callable from realtime/embedded paths.

---

## Io (Tier 1 — completion-based, zero runtime)

```jolt
using Io;
$$ring = Ring::new(entries: 64);
$op = ring.submit(read(fd, claim(buf)));
$n  = ring.wait(op)?;
```
- `Ring` (submission/completion queues), `Op` handles, `read`/`write`/`accept`/`connect`/`fsync`
  ops, `Completion`. Backends: io_uring (Linux), IOCP (Windows), kqueue (BSD/macOS).
- Low-level `Reader`/`Writer` contracts shared with `Fiber`.

## Fiber (Tier 2 — green threads, ergonomic default)

```jolt
using Fiber;
@fetch(url: String) !Bytes -> $c = connect(url)?; c.read_all() ;;   // blocking-style, parks fiber
```
- `Runtime` (executor over OS threads, built on `Io`), `spawn` integration with `Concurrent::scope`.
- Blocking-style `Reader`/`Writer`, `BufReader`/`BufWriter`, `Stdin`/`Stdout`/`Stderr`.
- `[noblock]` functions are statically guaranteed never to park.

**Low-level extensions (both tiers).** Scatter/gather `readv`/`writev` over `Span` arrays;
zero-copy `sendfile`/`splice`; registered fixed buffers and pre-registered FDs on `Io.Ring` for
syscall-free hot loops; raw FD/handle access (`as_raw_fd`, `from_raw_fd`); `[noalloc]` buffered
readers/writers backed by a caller `Span<Byte>`; direct/unbuffered and `O_DIRECT`-style modes for
block-device work.

---

## Concurrent

- **Structured (Model A):** `scope -> spawn -> … ;; ;;` (tasks joined at scope exit).
- **Raw (Model B):** `Thread::spawn(closure) Thread`, `join`, `Thread::current`, `yield`, `sleep`.
- **Channels:** `Channel<T>` (bounded/unbounded), `send`/`recv`/`try_recv`, `select` over channels.
- **Locks/sync:** `Mutex<T>` (guard via `Dispose`), `RwLock<T>`, `Once`, `Barrier`, `Semaphore`,
  `CondVar`.
- **Atomics:** `Atomic<T>` for integer/bool/pointer; `load`, `store`, `add`, `compare_swap`.
- All gated by `Sendable`/`Shareable`; the Custodian rejects races at compile time.

**Low-level extensions.** Full memory-ordering control on every atomic op (`Ordering` =
`Relaxed`…`SeqCst`) and standalone `fence`/`compiler_fence`; `SpinLock<T>` and lock-free
`AtomicQueue`/`AtomicStack` for IRQ/no-OS contexts; thread affinity/pinning, `park`/`unpark`,
priority hints; `CachePadded<T>` to avoid false sharing; a `[noblock]` `try_lock → guard?` on every
lock for realtime paths.

---

## Time

- `Instant` (monotonic), `SystemTime` (wall clock), `Duration` (`from_secs`, `from_millis`, `+`/`-`).
- `Clock` contract; `Stopwatch`; `sleep(Duration)`.
- `Calendar` submodule: `Date`, `DateTime`, `TimeZone`, parsing/formatting (ISO 8601, strftime-like).

**Low-level extensions.** Raw cycle counters (`rdtsc`/`cntvct` via `Intrin`) and `Instant` backed by
a monotonic hardware timer; `[noalloc]`/`[noblock]` `Duration` arithmetic (pure integer math); a
`Timer`/`Countdown` abstraction over hardware timers for embedded; busy-wait `delay_cycles(n)` for
sub-tick precision where no clock exists.

---

## Fs

- `File` (`open`/`create`/`read`/`write`/`seek`, conforms to `Dispose`), `OpenOptions`.
- `Path` / `PathBuf` (join, parent, extension, normalize), `Dir` (read entries, walk).
- `metadata`, `exists`, `copy`, `rename`, `remove`, `create_dir_all`, `read_to_string`,
  `read_bytes`, `write`. Works over both `Io` and `Fiber`.

**Low-level extensions.** Raw `open`/`read`/`write` against integer FDs; `mmap`/`munmap` returning a
`Span<Byte>` view; positional `pread`/`pwrite`; `O_DIRECT`/unbuffered + `fsync`/`fdatasync`;
`ioctl`/`fcntl` escape hatches (`[unsafe]`); block-device helpers for filesystems and storage drivers.

---

## Net

- Addresses: `IpAddr`, `SocketAddr`, DNS `resolve`.
- TCP: `TcpListener`, `TcpStream`; UDP: `UdpSocket`.
- TLS: `TlsConnector`/`TlsAcceptor` (on `Crypto`).
- Higher level: `http` submodule (client + minimal server), `url` parsing.

**Low-level extensions.** Raw socket FD access and `setsockopt`/`getsockopt`; `AF_PACKET`/raw sockets
for custom protocols; zero-copy `sendmmsg`/`recvmmsg` over `Span` batches; manual buffer management
(no implicit allocation per packet); a `[nostd]`-capable minimal TCP/IP path for embedded NICs.

---

## Os

- `args() Array<String>`, `env` (`get`/`set`/`vars`), `exit(code)`, `Process`/`Command`
  (spawn subprocesses, pipes), `Signal` handling, `cwd`/`chdir`, `hostname`, raw `syscall` (unsafe).

**Low-level extensions.** Direct `syscall(n, args…)` with per-arch numbers (→ `Arch`); `mmap`/`mprotect`/
`brk` memory syscalls; raw signal handlers with `[interrupt]`-style frames; `clone`/`fork`/`execve`
primitives under the high-level `Process`; CPU affinity, `nice`/scheduler class, `rlimit`; for
freestanding targets, `Os` degrades gracefully — unavailable calls are absent under `[nostd]` rather
than stubbed.

---

## Encoding

- **Serialization framework:** `Serialize` / `Deserialize` contracts, derivable via comptime
  (`#derive(Serialize)`).
- Formats: `json`, `toml`, `yaml`, `csv`, `msgpack`, `bincode`.
- Byte utils: `base64`, `hex`, `Bytes`/`BytesMut`, endian read/write.

```jolt
using Encoding;
#derive(Serialize, Deserialize)
struct Config -> $name: String; $$port: Uint; ;;
$txt = json.to_string(borrow(cfg))?;
$cfg2 = json.parse<Config>(txt)?;
```

**Low-level extensions.** Zero-copy parse/emit over `Span<Byte>` with no intermediate allocation
(`[noalloc]`-friendly); streaming/incremental decoders for fixed-size buffers; raw endian-aware
struct (de)serialization driven by `Layout`/`[repr: C]` for binary wire formats and on-disk
structures; `cobs`/`varint`/length-prefix framing for embedded links.

---

## Crypto

- Hashing: `sha256`, `sha3`, `blake3`, `Hasher` contract.
- AEAD: `aes_gcm`, `chacha20poly1305`.
- Signatures/KX: `ed25519`, `x25519`.
- `SecureRandom`, `constant_time_eq` ( `[constanttime]` ), key types are `[zeroize]` + `[secret]`.

**Low-level extensions.** Hardware-accelerated paths (AES-NI, SHA extensions, ARM crypto) via
`Intrin`, with `[constanttime]` scalar fallbacks; `[noalloc]` one-shot APIs that write into a caller
`Span`; incremental hashers for streaming over fixed buffers; access to a hardware RNG/TRNG where the
target exposes one.

---

## Iter

Lazy adapters returning new `Iterator`s: `map`, `filter`, `take`, `skip`, `zip`, `enumerate`,
`chain`, `flatten`, `flat_map`, `window`, `chunks`, `step_by`, `rev`. Consumers: `collect |C|()`,
`fold`, `reduce`, `sum`, `product`, `count`, `any`, `all`, `find`→`T?`, `min`/`max`, `for_each`.

```jolt
using Iter;
$top3 = scores.iter().filter(|s| -> s.value > 0 ;;).sorted().rev().take(3).collect<Array>();
```

**Low-level extensions.** All adapters are `[noalloc]` (they're lazy state machines, no heap); a
`collect_into(span)` consumer fills a caller buffer instead of allocating; SIMD-accelerated bulk
operations (`sum`, `map`, `filter`) over contiguous `Slice`/`Span` via `Simd`; chunked iteration
designed to vectorize cleanly.

---

## Test

A first-class, extensive testing library — full design in **`jolt-testing.md`**.

- **Macro assertions** (capture expression source): `assert!`, `assert_eq!`, `assert_ne!`,
  `assert_near!`, `assert_throws!`, `assert_matches!`, `assert_snapshot!`; soft `expect_*` family.
- **Attributes:** `[test]` (+ `should_fail`/`skip`/`only`/`tags`/`cases`), `[bench]`, `[fuzz]`,
  `[simulation]`, `[setup]`/`[teardown]`.
- **Property testing:** `for_all(gen, property)` with shrinking; `Generator`/`Arbitrary` +
  `#derive(Arbitrary)`; deterministic, seed-reproducible.
- **Fuzzing:** coverage-guided `[fuzz]` targets, crashes auto-saved as regression tests.
- **Sanitizers** (test mode): address/UB/thread/leak — backstop for `[unsafe]`/FFI.
- **Snapshot** (`assert_snapshot!`) and **mutation testing** (`--mutate`, mutation score).
- **Deterministic Simulation Testing (`[simulation]` + `Sim`):** run a whole concurrent/distributed
  system on a virtual clock with a controlled scheduler and seed-driven fault injection
  (crash/partition/delay/reorder); any failure replays exactly from its seed. Backed by substituting
  the fiber runtime + completion-`Io` with a deterministic simulator.
- **Mocking:** contract-conforming doubles + `#derive(Mock)`.
- **Benchmarking:** statistical, with regression tracking vs a baseline.
- **Doctests:** runnable `///` examples, run under `jolt test`.
- **Runner:** parallel/isolated, cache-aware ("cached pass"), permission-sandboxed per test,
  coverage, `--watch`, `[nostd]`/QEMU harness. (See toolchain §9.)

---

## Log

- Levels `trace`/`debug`/`info`/`warn`/`error`; structured fields; pluggable sinks
  (`ConsoleSink`, `FileSink`, `JsonSink`); `[secret]` fields are redacted automatically.

---

## Reflect

- Comptime: `typeinfo(T)` → fields, methods, conformed contracts, size/align; `emit_*` codegen
  helpers for proc macros.
- Optional runtime type info (opt-in per type via `[reflect]`) for dynamic scenarios.

---

# Low-level programming

The systems layer for kernels, drivers, firmware, bootloaders, and embedded targets. Everything here
works under `[nostd]`, most under `[noalloc]`, and the unsafe-required pieces are confined to
`[unsafe]`. The Custodian still runs on safe references throughout — going low-level does not turn
off memory safety, it only widens what you can express.

## Layout — data layout control

Control how types are laid out in memory (essential for ABI, hardware structs, wire formats).

- Type attributes: `[repr: C]` (C-compatible), `[repr: packed]` (no padding), `[repr: transparent]`
  (single-field newtype, same layout as inner), `[repr: align(N)]`, `[repr: u8/u16/…]` (enum tag size).
- Queries (comptime): `size_of|T|()`, `align_of|T|()`, `offset_of|T|(field)`, `is_zst|T|()`.

```jolt
[repr: C, packed]
struct Header -> $magic: Uint32; $version: Uint16; $flags: Uint16; ;;

comptime -> assert(size_of<Header>() == 8); ;;
```

## Ptr — raw pointers & spans

Beyond safe `Borrow`/`Claim`: raw addressing for MMIO, FFI, and allocators. All deref/arithmetic is
`[unsafe]`.

- `Pointer<T>` (mutable), `ConstPtr<T>` (read-only), `Address` (a `Uint` newtype for raw addresses).
- Ops (`[unsafe]`): `read(p)`, `write(p, v)`, `offset(p, n)`, `add`/`sub`, `align_up`,
  `read_volatile`/`write_volatile` (or use `Volatile`), `cast|U|(p)`.
- Non-null/aligned wrappers: `NonNull<T>`, `Aligned<T, N>`.
- `Span<T>` / `SpanMut<T>` — a raw pointer + length with **bounds-checked** indexing (the unsafe
  cousin of `Slice<T>`, used when you can't borrow-check the source, e.g. DMA buffers).

```jolt
using Ptr;
[unsafe]
@poke(addr: Address, value: Uint32) None ->
    $p = addr.as_ptr<Uint32>();
    write_volatile(p, value);
;;
```

## Volatile — memory-mapped I/O

Typed volatile access so the compiler never elides, reorders, or coalesces hardware reads/writes.

- `Volatile<T>` (read+write), `ReadOnly<T>`, `WriteOnly<T>`.
- `Mmio<T>` — maps a register block at a fixed address; field access compiles to volatile ops.
- `@register` helper to declare register maps with bit-field accessors.

```jolt
using Volatile;
struct UartRegs -> $data: Volatile<Uint32>; $status: ReadOnly<Uint32>; ;;

[unsafe]
@uart_putc(regs: Mmio<UartRegs>, c: Byte) None ->
    for regs.status.read() & TX_FULL != 0 -> () ;;     // spin
    regs.data.write(c as Uint32);
;;
```

## Bits — bit manipulation & packing

- `Bitset<N>` (fixed) / `Bitvec` (dynamic): `set`, `clear`, `toggle`, `test`, `count_ones`,
  `first_set`, iteration.
- Bit fields on structs: `[bitfield]` types with `@field` accessors that pack into a backing integer.
- Helpers: `rotate_left`/`right`, `leading_zeros`, `trailing_zeros`, `popcount`, `reverse_bits`,
  `byte_swap`; endian: `to_le`/`to_be`/`from_le`/`from_be`, `Endian` reader/writer over `Span`.

```jolt
[bitfield: Uint32]
struct PageEntry ->
    @present: Bool @ 0;          // bit 0
    @writable: Bool @ 1;         // bit 1
    @addr: Uint @ 12..32;        // bits 12-31
;;
```

## Concurrent.Atomic — lock-free & memory ordering

(Lives in `Concurrent`, highlighted here for systems use.)

- `Atomic<T>` for `Int*/Uint*/Bool/Pointer`: `load`, `store`, `swap`, `compare_swap`,
  `fetch_add`/`sub`/`and`/`or`/`xor`.
- Explicit memory ordering: `Ordering` = `Relaxed | Acquire | Release | AcqRel | SeqCst`.
- Fences: `fence(ordering)`, `compiler_fence(ordering)`.
- Used to build the higher-level `Mutex`/`Channel`; available raw for lock-free structures.

```jolt
$flag = Atomic::new(false);
flag.store(true, Ordering.Release);
for !flag.load(Ordering.Acquire) -> cpu_relax() ;;
```

## Abi — C ABI & foreign interface

- `extern` declarations + `[extern: "C"]`, `[extern: "stdcall"]`, etc. (see spec §20).
- `[repr: C]` structs/enums/unions for passing by value across the boundary.
- `CStr`/`CString` (NUL-terminated), `c_char`/`c_int`/`c_void` aliases, `varargs` support.
- Callback support: pass a Jolt fn as a C function pointer (`[no_capture]` closures only).

```jolt
[extern: "C"] @qsort(base: Pointer<None>, n: Uint, sz: Uint, cmp: ExternFn) None;
```

## Intrin / Simd — CPU intrinsics & vectors

- `Simd`: portable vectors `Vec<T, N>` (`f32x4`, `i32x8`, …): lane-wise arithmetic, shuffle, reduce,
  masks, gather/scatter; auto-lowers to SSE/AVX/NEON or scalar fallback.
- `Intrin`: `prefetch`, `pause`/`cpu_relax`, `likely`/`unlikely` hints, `unreachable()`,
  `black_box`, `bswap`, `clz`/`ctz`, `rdtsc`; architecture-gated via `[arch: …]`.

```jolt
using Simd;
@add4(a: f32x4, b: f32x4) f32x4 -> a + b ;;     // one SIMD instruction
```

## Arch — architecture-specific

- Submodules `Arch.X86_64`, `Arch.Aarch64`, `Arch.Riscv`, `Arch.Wasm`: register read/write,
  privileged instructions, `cpuid`/feature detection, port I/O (`inb`/`outb` on x86), MSRs, TLB/cache
  ops, barriers.
- Compile-time target queries: `target.arch`, `target.os`, `target.endian`, `target.pointer_width`,
  `cfg`-style conditional compilation via `[cfg: …]` attributes.

```jolt
[cfg: arch == "x86_64"]
[unsafe] @halt() None -> asm -> hlt; ;; ;;
```

## Interrupt — handlers & exceptions

- `[interrupt]` / `[interrupt: "irq"]` calling-convention attribute (saves/restores full frame).
- `InterruptFrame` type, `without_interrupts(|| -> … ;;)` critical sections, `mask`/`unmask`,
  IDT/vector-table helpers (arch-gated).

```jolt
[interrupt]
@timer_isr(frame: InterruptFrame) None -> ticks.fetch_add(1, Ordering.Relaxed); ;;
```

## Boot — freestanding entry & sections

- `[no_std]`, `[no_main]`, custom `@_start` entry, `[link_section: ".text.boot"]`,
  `[link_section: ".bss"]`, `[used]`/`[no_mangle]` for symbols the linker/hardware needs.
- Minimal runtime shims: `panic_handler` hook, `global_allocator` selection, `lang_item`-style hooks.
- Linker-script integration helpers and `extern` symbols for section bounds (`__bss_start`, …).

```jolt
[no_mangle, link_section: ".text.boot"]
@_start() None -> clear_bss(); kmain(); ;;
```

## Embed — embedded patterns

- HAL contracts: `Gpio`, `Spi`, `I2c`, `Uart`, `Pwm`, `Adc` (driver-agnostic interfaces).
- No-alloc building blocks: `StaticVec<T, N>`, `RingBuffer<T, N>`, `Pool<T, N>`, `heapless` maps.
- `[noalloc]`/`[noblock]`-friendly throughout; fits `bounded_stack` and `nopanic` work.
- `singleton!` macro for safe one-time peripheral ownership; `critical_section` integration.

## Inline — typed inline assembly

A structured layer over raw `asm` (§19): typed operand bindings, clobbers, and output constraints,
so inline asm participates in the type system instead of being an opaque block.

```jolt
[unsafe]
@read_tsc() Uint64 ->
    inline ->
        out("eax") $lo: Uint32,
        out("edx") $hi: Uint32,
        clobber("memory"),
        "rdtsc";
    ;;
    (hi as Uint64 << 32) | (lo as Uint64)
;;
```

---

## Low-level guarantees recap

| Concern | Mechanism |
| ------- | --------- |
| no heap | `[noalloc]` + `Embed` static containers; `Null` allocator |
| no runtime | `[nostd]`, `Boot`, Tier-1 `Io` (completion-based) |
| no panics | `[nopanic]` + custom `panic_handler` |
| bounded stack | `[bounded_stack: N]`, `[norecurse]` |
| exact layout | `Layout` (`[repr: …]`), `Bits` (`[bitfield]`) |
| hardware access | `Volatile`/`Mmio`, `Arch` port I/O, `Ptr` raw addressing |
| determinism | `Concurrent.Atomic` orderings, `Intrin` hints, `[constanttime]` |
| safety preserved | Custodian still checks all safe `Borrow`/`Claim`; unsafe is opt-in & scoped |

---

## Open design questions for the stdlib

1. **Error taxonomy.** With open error sets (`!T`), do stdlib modules each declare their own error
   values (`Fs.NotFound`, `Net.Timeout`) that compose into a caller's open set? (Recommended yes.)
2. **`String` vs `StringView` ergonomics.** Which APIs take owned vs borrowed strings by default —
   lean on borrows (`borrow`) to avoid copies?
3. **Allocator threading.** Do all `Collections` take an explicit allocator parameter, default to
   the in-scope one, or both (`Array::new()` vs `Array::with_allocator(a)`)? (Recommended: default +
   explicit override.)
4. **`Io`/`Fiber` API unification.** How much surface (`Reader`/`Writer`) is shared so code written
   against one tier ports to the other with minimal change?
5. **`Display` vs `Debug` for interpolation.** Does `"{x}"` use `Display` and `"{x:?}"` use `Debug`?
6. **Sync vs async naming collision** — none, since there's no async; but `Fiber` blocking calls and
   `Io` completion calls may want parallel names (`read` in both). Namespacing handles it.

7. **Low-level register/bitfield syntax.** The `@field: T @ bit` form (in `[bitfield]` and
   `@register`) reuses `@` and a bit-position `@` — confirm this parses cleanly or pick a distinct
   bit-range syntax.
8. **`Span` vs `Slice` boundary.** `Slice<T>` is borrow-checked; `Span<T>` is raw+length for cases
   the Custodian can't track (DMA, MMIO). Define exactly when each is required.
9. **Target configuration.** `[cfg: …]` conditional compilation and `target.*` comptime queries need
   a defined set of keys (arch/os/endian/pointer_width/features) and a resolution model.
10. **Allocator under `[noalloc]`.** Confirm that `[noalloc]` code may still use stack/static
    containers (`StaticVec`, `Pool`) and that the `Null` allocator makes accidental heap use a
    link-time/compile-time error.

## Suggested build order
`Core`/`Prelude` → `Mem` → `Collections` + `Text` + `Iter` → `Math` → `Io`/`Fiber` → `Concurrent`
→ `Fs`/`Net`/`Os`/`Time` → `Encoding`/`Crypto` → `Test`/`Log`/`Reflect`.
Low-level layer can be built in parallel from early on: `Layout`/`Ptr`/`Volatile`/`Bits` →
`Concurrent.Atomic`/`Abi` → `Intrin`/`Simd`/`Arch` → `Interrupt`/`Boot`/`Embed`/`Inline`.
