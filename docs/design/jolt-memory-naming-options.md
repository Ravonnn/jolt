# Jolt — Memory System Naming Options

> The memory system has ~6 things that need names that work *together*: the **checker** (the pass),
> the **shared borrow**, the **mutable borrow**, the **dereference** op, the **error state**, and
> the **move**. Below are complete, coherent schemes. Current = Grounder / `tap` / `tap_mut` /
> `deref` / "ungrounded". Pick a whole scheme, or mix one column from each.

---

## Full coherent schemes (electrical theme — fits "Jolt")

### Scheme E1 — Grounder (current)
| Slot | Name |
| ---- | ---- |
| checker | **the Grounder** |
| shared borrow | `tap(x)` → `Tap<T>` |
| mutable borrow | `tap_mut(x)` → `TapMut<T>` |
| dereference | `deref(t)` |
| error state | **ungrounded** |
| move | move |

Pro: cohesive, "ungrounded" is intuitive. Con: "tap" and "ground" are different sub-metaphors.

### Scheme E2 — Circuit (one metaphor end-to-end)
| Slot | Name |
| ---- | ---- |
| checker | **the Circuit** / circuit analysis |
| shared borrow | `wire(x)` → `Wire<T>` |
| mutable borrow | `wire_live(x)` → `LiveWire<T>` |
| dereference | `read(w)` / `drive(w)` |
| error state | **short** ("short circuit") |
| move | reroute |

Pro: single consistent metaphor (live wire = the one mutable holder; a short = conflict). Memorable.

### Scheme E3 — Charge
| Slot | Name |
| ---- | ---- |
| checker | **the Regulator** |
| shared borrow | `tap(x)` → `Tap<T>` |
| mutable borrow | `charge(x)` → `Charge<T>` |
| dereference | `read(t)` |
| error state | **overload** |
| move | discharge / transfer |

Pro: "overload" as the error is evocative. "Charge" = exclusive/energized = mutable.

---

## Non-electrical schemes (if you'd rather drop the theme)

### Scheme L — Lease (property/rental metaphor)
| Slot | Name |
| ---- | ---- |
| checker | **the Lease Checker** / Registrar |
| shared borrow | `lease(x)` → `Lease<T>` |
| mutable borrow | `lease_mut(x)` → `LeaseMut<T>` |
| dereference | `deref(l)` |
| error state | **lease conflict** |
| move | transfer / hand over |

Pro: ownership/lease is the most *literal* description of what's happening; very teachable.

### Scheme V — View/Hold
| Slot | Name |
| ---- | ---- |
| checker | **the Aliasing Checker** / Warden |
| shared borrow | `view(x)` → `View<T>` |
| mutable borrow | `hold(x)` → `Hold<T>` |
| dereference | `get(v)` / `deref(v)` |
| error state | **alias conflict** |
| move | move |

Pro: `view` (read) vs `hold` (exclusive) reads naturally; no `_mut` suffix needed.

### Scheme C — Custody (guardianship)
| Slot | Name |
| ---- | ---- |
| checker | **the Custodian** |
| shared borrow | `borrow(x)` → `Borrow<T>` |
| mutable borrow | `claim(x)` → `Claim<T>` |
| dereference | `deref(b)` |
| error state | **custody violation** |
| move | hand off |

Pro: "Custodian" as the checker is friendly; `borrow`/`claim` distinguishes shared vs exclusive.

### Scheme W — Watcher/Steward (minimal, neutral)
| Slot | Name |
| ---- | ---- |
| checker | **the Steward** / Sentinel / Warden |
| shared borrow | `ref(x)` → `Ref<T>` |
| mutable borrow | `ref_mut(x)` → `RefMut<T>` |
| dereference | `unref(r)` |
| error state | **safety violation** |
| move | move |

Pro: keeps the original v0.3 borrow ops; only names the *checker* distinctively.

---

## Just the checker (keep `tap`/`deref`, rename only the pass)

If you like the current borrow ops and only want the checker name swapped, candidates:

- **the Grounder** (current)
- **the Sentinel** — guards safety; neutral, strong
- **the Warden** — same energy, slightly sterner
- **the Steward** — emphasizes responsible management of resources
- **the Regulator** — electrical + "enforces rules" double meaning
- **the Custodian** — friendly, care-oriented
- **the Anchor** — ties values down (pairs with "anchored"/"unanchored" error state)
- **the Conductor** — electrical + "directs/orchestrates" pun

### Matching error-state words by checker name
| Checker | "Good" state | "Bad" state (the error) |
| ------- | ------------ | ----------------------- |
| Grounder | grounded | **ungrounded** |
| Anchor | anchored | **adrift / unanchored** |
| Sentinel/Warden | clear | **violation / breach** |
| Regulator | regulated | **overload** |
| Conductor | in tune | **clash** |

---

## My picks

1. **If keeping the electrical theme and one metaphor:** **Scheme E2 (Circuit / wire / live wire /
   short)** is the most cohesive — "the one live wire" naturally captures "exactly one mutable
   borrow," and "a short" is a perfect error word.
2. **If you want maximum teachability:** **Scheme L (Lease)** — it literally describes the model, so
   docs almost write themselves ("you can hand out many read leases, or one write lease").
3. **If you only want to rename the checker and keep `tap`/`deref`:** **the Anchor**, with
   grounded→**anchored** / error→**adrift**. "Adrift" is a vivid, friendly error state and pairs
   well with `tap`.

Tell me which scheme (or which column-mix) and I'll apply it across the spec, changelog, and
keyword list in one pass.
