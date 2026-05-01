# AGENTS.md

This repository optimizes for narrow, provable Rust changes.

## Mission

- Keep domain invariants in `crates/domain`.
- Keep application orchestration in `crates/application`.
- Keep external systems and side effects in `crates/adapters`.
- Keep HTTP/RPC boundaries in `crates/api`.
- Keep operator commands in `crates/cli`.

## First Commands

1. Run `cargo metadata --no-deps --format-version 1` when ownership is unclear.
2. Run `cargo check --workspace --all-targets --message-format=json` before broad validation.
3. Run the lane in `proof-lanes.toml` that matches the changed owner.

## Ownership Map

- `crates/domain`: pure domain rules, type invariants, closed state machines.
- `crates/application`: use cases, transactions, coordination.
- `crates/adapters`: database, network, filesystem, queues, third-party APIs.
- `crates/api`: request/response contracts, route handlers, boundary schemas.
- `crates/cli`: command-line operator surface.

## Forbidden Or Escalated Edits

- Do not edit generated files directly.
- Do not add dependencies without rationale.
- Do not modify unsafe or FFI code without updating the unsafe ledger.
- Do not widen a patch across owners without stating why.

## Proof And Security Rules

- Use the smallest proof lane that matches the changed surface.
- Dependency, secret, CI/CD, authz, unsafe, and FFI edits require the security lane.
- Preserve raw command output paths and exit codes for compressed summaries.
- Never summarize a security failure as green.

## Definition Of Done

- Changed owner has regression proof.
- Public contract changes have generated diff or API proof.
- Security-sensitive edits include security-lane evidence.
- Review packet states owners changed, commands run, and raw-output locations.
