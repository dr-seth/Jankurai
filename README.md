<img src="assets/jankurai_header.png" alt="Jankurai" width="100%">

# Jankurai

Jankurai is the control plane for agent-native repositories. It makes ownership, proof, generated zones, and repair evidence machine-readable so wrong code is easier to reject than to merge.

The moonshot is simple:

```text
intent -> bounded agents -> proof lanes -> evidence -> expired exceptions -> reusable primitives
```

That is the operating loop behind the repo, the paper, and the audit CLI. Humans set intent and risk tolerance. Jankurai routes the change, proves the result, records the evidence, and forces temporary exceptions to age out instead of becoming architecture.

## What Lives Here

- `crates/jankurai/` - Rust audit CLI and proof/router logic
- `packages/ux-qa/` - rendered-UX geometry and accessibility checks
- `agent/` - owner map, test map, generated-zone manifest, proof lanes, version bindings
- `docs/` - mission, standard, release, testing, and architecture notes
- `paper/` - the paper source and generated PDF
- `tips/` - phase notes and source material
- `reference/` - read-only source material

## Fast Start

```bash
cargo install --path crates/jankurai --locked
jankurai audit . --json agent/repo-score.json --md agent/repo-score.md
```

From source:

```bash
just fast
just score
just paper
```

The canonical audit lane is:

```bash
cargo run -p jankurai -- . --json agent/repo-score.json --md agent/repo-score.md
```

## Standard Files

These files define the control plane and should stay in sync:

- `AGENTS.md`
- `agent/JANKURAI_STANDARD.md`
- `agent/owner-map.json`
- `agent/test-map.json`
- `agent/generated-zones.toml`
- `agent/proof-lanes.toml`
- `agent/standard-version.toml`

## Validation

The usual local checks are:

```bash
just fast
just score
just ux-qa
just paper
just check
```

Generated outputs stay in declared generated zones. `reference/` stays read-only. The repo should remain clean enough that a fresh agent can locate ownership, choose the smallest proof lane, and leave a verifiable repair trail.
