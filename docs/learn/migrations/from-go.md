# Coming from Go

| Go | Jolt (Tiny) |
| -- | ----------- |
| `func main()` | `@main() None -> … ;;` |
| `:=` / `var` | `$` / `$$` |
| `for i := 0; i < n; i++` | `for x in n` (Tiny) |
| `if err != nil` | `!T` / `?` (future) |
| `go func()` | fibers (future) |

## Syntax feel

Jolt uses **tail expressions** instead of explicit `return` in many cases. Functions end with `;;`.

## Testing

```bash
jolt test tests/test/    # like go test
```

**Start:** [Tour](../tour/01-hello.md) · [Jolt by Example](../examples/index.md)
