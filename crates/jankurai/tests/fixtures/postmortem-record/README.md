# Verb #4 — `jankurai postmortem record` fixtures

Verb: validate + persist a postmortem TOML according to the schema (proposal §Contribution #4). Provides a feedback loop into slice-risk via `slice-risk --use-postmortems`.

## Schema enums (gating)

```
outcome      ∈ {blocked, rolled_back, shipped_with_caveats}
failure_mode ∈ {aspirational-spec, env-prerequisite, interop-runtime,
                equivalence-gap, cutover-rollback, perf-regression}
severity     ∈ {low, medium, high, critical}
```

## Fixtures

| File | Expected verdict | Why |
|------|------------------|-----|
| `positive/atlas-2026-05.toml` | accept; write to `.jankurai/postmortems/2026-05-phase-3d-atlas.toml` | Real Atlas postmortem (see `docs/postmortems/atlas-rust-rollback-2026-05.md`). All required fields present, all enums valid. |
| `adversarial_missing_failure_mode.toml` | reject — schema error | `failure_mode` field absent |
| `adversarial_invalid_failure_mode.toml` | reject — enum violation | `failure_mode = "totally-made-up-mode"` |
| `adversarial_outcome_typo.toml` | reject — enum violation | `outcome = "blocked-ish"` (typo) |
| `adversarial_missing_lessons.toml` | reject — required block absent | `[lessons]` section empty |

## Feedback-loop test (verb #3 interaction)

After `postmortem record positive/atlas-2026-05.toml` writes the postmortem, running:
```
jankurai migrate slice-risk ../slice-risk/positive_env_blocker_slice.toml --use-postmortems
```
MUST emit a cross-reference line (see `../slice-risk/expected_output.txt`).
