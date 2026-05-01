# Agent-Ready Rust Checklist

Use this checklist as the operational supplement for "Minimum Semantic Surface." Mark items as `present`, `partial`, or `missing`; do not count a repository as agent-ready when setup, proof, or security gates are missing.

## Repository Shape

- Root `AGENTS.md` is short, command-oriented, and points to local docs instead of duplicating them.
- Cargo workspace membership, `default-members`, generated zones, and owner crates are documented.
- `agent-map.json` or equivalent repo map identifies crates, entrypoints, generated files, and owner paths.
- `test-map.json` or equivalent test map links source paths to fast, medium, deep, and security proofs.
- Ordinary feature work has a named owner crate or module and usually touches three to five owner files.

## Rust Contracts

- Domain identifiers and units use newtypes instead of raw strings or integers.
- Externally supplied values enter through validated constructors.
- Closed state machines use enums; open extension surfaces are explicit.
- Public structs protect invariants with private fields where possible.
- API, database, JSON, CLI, and TypeScript contracts are generated or compile-time checked where practical.

## Failure Surfaces

- Library errors are typed and stable.
- Application errors include context, owner/layer, remediation hint, and stable code where useful.
- Logs and traces use structured fields and spans.
- Test failures point to the violated invariant and next proof command.

## Verification

- Fast lane exists and completes quickly enough that agents will actually run it.
- Medium lane proves changed contracts and feature combinations.
- Deep lane covers unsafe, concurrency, public API, parser, state machine, and performance hotspots.
- Every fixed bug gets a regression test.
- External APIs and generated contracts have contract tests or snapshots.

## Security

- Security lane is explicit in root instructions and CI.
- Unsafe blocks and FFI boundaries have local invariant notes.
- Dependencies have owners and rationale.
- Secret handling, untrusted input, path traversal, SSRF, authz, and deserialization risks are tested where relevant.
- Destructive or external side-effect commands require human approval.

