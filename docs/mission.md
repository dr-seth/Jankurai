# Mission: Jankurai

Paper title: "Jankurai: The Anti-Vibe Coding Standard"

Subtitle: "Continuous Repository Alignment for Proof-Carrying AI Pull Requests"

Public thesis line: "Find the vibe. Prove the merge. Repair the repo."

Jankurai exists to make AI-assisted merge auditable. AI makes plausible code cheap; Jankurai makes merge evidence explicit. A conformant repository can answer, at the current commit: what changed, who owns it, which proof lanes apply, which receipts exist, which evidence is missing, what repair is queued, and why the merge passed, reviewed, blocked, ratchet-failed, or release-failed.

In this project, "proof" means repository-local evidence receipts. It does not mean formal proof of full program semantics.

## Core First

Jankurai Core is the standard:

- version manifests
- owner maps
- test/proof maps
- generated-zone policy
- proof receipts
- merge witnesses
- repair queues
- waivers
- evidence indexes
- stable HLT rule IDs
- a closed decision enum: `pass`, `review`, `block`, `ratchet_fail`, `release_fail`

The Rust/TypeScript/React/Vite/PostgreSQL/bounded-Python layout is a reference profile. It is useful because it gives agents and CI a concrete implementation shape, but it is not Jankurai Core. Go, .NET, JVM, Rails, Python, Elixir, or other profiles can conform when they emit equivalent owner routes, proof receipts, generated-zone evidence, security/UX evidence where applicable, repair queues, and merge witnesses.

Score is posture. Witness is decision. Conformance is pass/fail.

## Current Paper Mission

The paper argues for continuous repository alignment: ownership, generated boundaries, proof lanes, security controls, UX evidence, waivers, and repair receipts stay synchronized with each merge. The central artifact is the merge witness, a versioned binding among changed paths, owner routes, required proof receipts, observed evidence, missing-evidence decisions, artifact digests, tool identity, and commit identity.

The non-normative profile sections should support the standard, not dominate it. Stack comparisons are policy assumptions and adoption guidance, not a reason to reject an otherwise conformant repository.

## Honest Project Status

The current conformance lane validates seed-suite inventory and expected JSON presence. It has 10 fixture directories and 12 expected JSON files. A per-fixture runner that emits observed witness decisions remains a project gap.

World-class open-source gaps to keep visible:

- deeper conformance runner with observed pass/block comparison
- accessible HTML or tagged PDF edition
- durable JEP/RFC governance documents
- independent implementation compatibility path
- public evidence registry and badge policy
- release checklist that binds schemas, rules, profiles, paper edition, and generated artifacts

## Adoption Mission

Jankurai should be useful before it is strict. Start with read-only observe/advisory reports. Add owner maps, test maps, generated zones, and version manifests. Require proof receipts and merge witnesses only after the team accepts a baseline. Ratchets and release gates should be opt-in, baseline-backed, and repair-oriented.

The goal is not ceremony. The goal is to make wrong code easier to reject, localize, prove, audit, and repair.
