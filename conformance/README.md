# Jankurai Seed Conformance Suite

This directory contains seed fixtures for the `0.7.0` paper cut. The suite is
not a broad benchmark. It is local conformance evidence for the standard's
central claim: merge decisions should be reproducible from versioned artifacts.

Fixture expectations live in `expected/*.json`. Each expected file names the
fixture, the primary rule exercised, the expected audit decision, and the merge
witness decision a conforming implementation should reach.

The seed suite is intentionally small:

- `hl3-pass-minimal`: known-good minimal repository shape.
- `ownerless-path-fail`: unmapped path should raise `HLT-003`.
- `unmapped-proof-fail`: path without proof route should raise `HLT-004`.
- `generated-zone-mutation-fail`: generated output changed without source proof should raise `HLT-002`.
- `secret-sprawl-fail`: secret-like material should raise `HLT-010`.
- `destructive-migration-fail`: destructive migration without safety proof should raise `HLT-021`.
- `authz-isolation-fail`: missing authorization isolation proof should raise `HLT-022`.
- `input-boundary-xss-fail`: unsafe rendering/input boundary should raise `HLT-023`.
- `overbroad-agency-fail`: overbroad agent/tool permissions should raise `HLT-012`.
- `rendered-ux-gap-fail`: user-facing UI without rendered proof should raise `HLT-013`.

Run:

```bash
just conformance
```
