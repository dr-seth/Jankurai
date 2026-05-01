# Agent Instructions

Keep this file short. It is a router, not a tutorial.

## Canonical Commands

- Fast lane: `cargo check --workspace && cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings`
- Targeted tests: `cargo nextest run -p <crate> <filter>`
- Medium lane: `cargo nextest run --workspace && cargo test --workspace --doc`
- Feature lane: `cargo hack check --workspace --feature-powerset --no-dev-deps`
- Security lane: `cargo deny check && cargo geiger`

## Repository Map

- Domain rules live in `<domain-crate>`.
- Application use cases live in `<application-crate>`.
- External adapters live in `<adapter-crate>`.
- Generated files are marked with `@generated`; edit the source schema instead.
- See `agent-map.json` for crate owners and `test-map.json` for validation routing.

## Default Flow

1. Reproduce the failure or run the smallest relevant proof.
2. Read the owner module and nearby tests before patching.
3. Patch the narrowest owner surface.
4. Run the mapped fast lane.
5. Widen only when the failure proves the owner or contract boundary is broader.
6. Add or update regression, property, snapshot, compile-fail, or security tests when behavior changes.

## Guardrails

- Do not edit generated files unless the generator is the requested target.
- Do not add broad `utils`, `common`, or `helpers` modules without a narrow named abstraction.
- Do not erase typed errors into strings.
- Do not add unsafe code without an invariant note and a proof command.
- Ask for approval before destructive, external, financial, or security-sensitive actions.

