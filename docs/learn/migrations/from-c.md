# Coming from C / C++

| C / C++ | Jolt (Tiny) |
| ------- | ----------- |
| `int main()` | `@main() None -> … ;;` |
| Manual `malloc`/`free` | Custodian — compile-time ownership |
| Pointers | `borrow` / `claim` / `deref` (safe) |
| `printf` | `println` |
| Header files | `using` imports (future) |

## Safety

Jolt rejects use-after-move and conflicting borrows **at compile time** via the Custodian — no
AddressSanitizer required for those classes of bugs.

**Start:** [Custodian tour](../tour/04-custodian.md)
