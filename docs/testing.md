# humanlint Testing

Testing is routed proof. Agents should not guess which tests matter.

| Lane | Purpose |
| --- | --- |
| `fast` | deterministic local proof for most edits |
| `contract` | API/schema generation and drift checks |
| `db` | migrations, constraints, schema drift |
| `web` | TypeScript typecheck, component tests, rendered UX QA |
| `e2e` | Playwright critical product flows |
| `security` | secrets, dependencies, SBOM/SCA, workflow lint |
| `observability` | traces, request IDs, structured error payloads |
| `audit` | humanlint repo score and hard-rule findings |
| `full` | release/merge gate |

For this workspace:

- `just versions` checks version and artifact bindings through the Rust auditor.
- `just ux-qa` builds and tests the optional Playwright geometry runtime.
- `just fast` writes a deterministic audit snapshot under `target/humanlint/`.
- `just score` writes `agent/repo-score.json` and `agent/repo-score.md`.
- `just paper` builds `paper/humanlint.pdf`.
- `just check` runs version checks, audit, and paper build.
- `humanlint doctor` and `humanlint init` write receipts under `target/humanlint/receipts/` for handoff evidence.
- `humanlint prove` executes a proof-plan JSON. Commands must match `agent/proof-lanes.toml` and `agent/test-map.json` after whitespace normalization, unless `--allow-unsigned-commands` is passed together with `HUMANLINT_ALLOW_UNSIGNED_PROOF_COMMANDS=1` (emergency only; keep CI on the default allowlist).
- Proof run artifacts: `target/humanlint/proof-receipts/*.json`, `target/humanlint/logs/*.log`, and `target/humanlint/evidence-index.json`, each validated against the matching `schemas/*.schema.json` on write where applicable.
- `humanlint doctor` validates proof receipts, the evidence index, `target/humanlint/security/evidence.json`, and when present `target/humanlint/context-pack.json` / `target/humanlint/repair-plan.json`; it warns on stale proof `git_head`. It validates **`agent/boundaries.toml`** and, when present, **`agent/ux-qa.toml`** (`schemas/boundaries.schema.json`, `schemas/ux-qa-policy.schema.json`). Parse/schema failures are **medium** (use `--fail-on medium` to treat as blocking).
- `humanlint context-pack` and `humanlint repair-plan` emit JSON validated against `schemas/context-pack.schema.json` and `schemas/repair-plan.schema.json` on every file write (and validate before printing to stdout when `--out` is omitted).
- `humanlint security run` runs `tools/security-lane.sh` (override with `--script`) via `bash -lc`, writes a combined log under `target/humanlint/security/`, and emits evidence JSON validated against `schemas/security-evidence.schema.json`. Pass `--strict` to enforce `HUMANLINT_SECURITY_STRICT=1` in the child environment (required tools must be present; advisory tools may still be skipped by the script depending on its logic).
- Receipts should record the command, exit code, changed paths, artifacts, and the rerun command that the next agent should trust.
- Phase closeouts should cite the exact receipt path instead of relying on chat history.
- Prefer structured errors, telemetry, and repair receipts that tell the next agent where to rerun proof.

Rendered UX QA combines Storybook states, Playwright screenshots, ARIA snapshots, visual review, axe/WCAG checks, CLS checks, MSW/generated mocks, design tokens, and deterministic DOM geometry rules such as edge clearance, target size, overlap, clipping, wrapping, horizontal overflow, sticky obstruction, focus visibility, form labels, and nested scrollbars.

Critical UI proof must be artifact-backed. A useful receipt names the route or story, browser, viewport, action sequence, screenshot or crop path, ARIA snapshot path when available, rule IDs, selectors, owner, and merge decision. Deterministic rule violations block; visual diffs require baseline approval; AI/VLM opinions route to review unless backed by a deterministic rule.

Schema-first work should get a parse smoke test before command wiring lands. For new contract files under `schemas/`, add a Rust test that loads the JSON and checks the required fields or references the contract chain. Keep that proof under `cargo test -p humanlint` so the schema stays machine-readable while the CLI surface is still being planned.

The security lane is wrapper-aware: `tools/security-lane.sh` is the canonical shell entrypoint for secret scanning, dependency review, SBOM, and workflow lint checks.

Observability repairs should stay typed. The auditor now carries repair-hint surfaces in `crates/humanlint/src/audit/mod.rs` with purpose, reason, common fixes, `docs_url`, and `repair_hint` fields so the next rerun stays local.

The CLI defaults to `domcontentloaded`. Override with `--wait-for` and `--timeout-ms` when the preview server or app shell needs a different readiness contract.

Route-matrix and Storybook commands:

```bash
humanlint ux audit --config agent/ux-qa.toml --out target/humanlint/ux-qa.json
humanlint ux storybook --url http://localhost:6006 --config agent/ux-qa.toml
```

Artifacts in reports must be relative to the repo or configured output root.

The full target-stack test doctrine lives in `docs/agent-native-standard.md`
and the canonical TeX paper under `paper/tex/`.
