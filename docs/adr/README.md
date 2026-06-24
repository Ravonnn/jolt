# Architecture Decision Records

ADRs capture **new** implementation decisions made during the build phase. They complement the
pre-design decisions log in [`../design/jolt-decisions.md`](../design/jolt-decisions.md).

## When to write an ADR

- Changing compiler architecture (e.g. query engine, IR shape)
- Deferring or revising a design doc default
- Choosing between implementation options (LLVM binding, hash function, etc.)

## Template

Copy [`0000-template.md`](0000-template.md) to `NNNN-short-title.md` and fill in each section.
