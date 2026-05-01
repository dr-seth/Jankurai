# Security Lane Checklist

Run this when a change touches authz, identity, secrets, external input, filesystem paths, network calls, database queries, deserialization, unsafe code, FFI, dependencies, CI/CD, or agent tools.

## Agent and Tooling Risks

- Prompt-injection sources are labeled as untrusted.
- Agent tools follow least privilege and avoid wildcard write/delete powers.
- High-impact actions require human approval.
- Logs and progress files redact secrets and PII.
- Agent loops have cost and time bounds.

## Rust and Application Risks

- Unsafe blocks have invariant notes and safe wrapper boundaries.
- FFI validates ownership, lifetime, nullability, and panic behavior.
- Path traversal, SSRF, injection, authz bypass, and deserialization risks have tests where applicable.
- Cryptography uses reviewed crates and documented protocols.
- Error paths do not leak secrets.

## Supply Chain

- Dependency rationale exists for new non-trivial crates.
- Advisory and license checks pass.
- Unsafe usage and dependency tree changes are reviewed.
- CI actions and tool versions are pinned where practical.
- Generated artifacts are reproducible or checked in with source provenance.

