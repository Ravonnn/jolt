# Security Policy

## Reporting a Vulnerability

If you discover a security issue in the Jolt compiler, toolchain, or repository infrastructure,
please report it responsibly:

1. **Do not** open a public GitHub issue for undisclosed vulnerabilities.
2. Email the maintainers with a description, reproduction steps, and impact assessment.
3. Allow reasonable time for a fix before public disclosure.

We will acknowledge receipt and work toward a remediation plan.

## Scope

- Stage-0 Rust compiler and `jolt` CLI
- CI workflows and release artifacts
- Package registry (when present)

Language semantics and user program safety are tracked separately via the Custodian and security
model in [docs/design/jolt-security-model.md](docs/design/jolt-security-model.md).
