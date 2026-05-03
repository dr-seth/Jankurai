# Phases feedback review status

This document records how `tips/phases_feedback/*` notes were considered relative to `docs/moonshot.md`, `agent/MASTER_PLAN.md`, and the live codebase. It is not a second roadmap; it is a reconciliation ledger.

Last reviewed: 2026-05-03

Full artifact/schema index: `docs/artifact-contracts.md`.

## Summary

| Area | Feedback files | Resolution |
| --- | --- | --- |
| Phase 00 | `00-phase/tip1`–`tip4` | **Aligned.** `tips/phases/00-phase-index.md` already implements the living router (status taxonomy, read-first, dependency graph, log contract). |
| Phase 01 | `01-standard/tip1`–`tip4` | **Schemas + doctor + exports + Finding contract.** `ArtifactSchema::Finding` → `finding.schema.json`; `report_compatibility_guard` validates **`findings[]`** and **`issues export --format jsonl`** lines. (Plus prior: Markdown headings, repair-queue JSONL, `just security` → `security run`, `docs/artifact-contracts.md`.) **Per-tool security steps:** bundled `tools/security-lane.sh` emits **`jankurai-security-step=`** JSON lines (needs **`python3`** on PATH for those lines); `jankurai security run` parses them into **`commands[]`**, else falls back to a single wrapper step (thin fixtures). |
| Phase 02 | `02-rule/tip1`–`tip4` | **Satisfied by current architecture / out of scope for one-shot apply.** Feedback scripts propose replacing `boundaries/` and audit wiring wholesale; the repo already has `audit/rules.rs`, finding builders, boundary checks, and registry tests per `tips/phases/02-rule-engine-semantic-oracle.md`. Large speculative rewrites were not merged; incremental rule/oracle work continues through normal audit PRs. |
| Phase 03 | `03-proof/tip1`–`tip4` | **Implemented in code.** `prove` exposes `--plan-out` / `--plan-md`, persists changed-mode plans, rejects ambiguous inputs and root-only changed paths, and populates `rules_covered` where deterministic; see `crates/jankurai/tests/proof_surface_smoke.rs` and `tips/phases/03-proof-router-evidence-ledger.md`. |
| Phase 04 | `04-init/tip1`–`tip4` | **Bundled profiles + `--profile-file`.** Seven bundled manifests; `crates/jankurai/src/init/profiles.rs` unit tests **`bundled_profile_contract`** require `templates/profiles/*.json` stems ↔ `BUNDLED_PROFILE_IDS` and manifest `id` ↔ bundle key. `docs/install.md` documents profiles. **Deferred:** deeper merge policy beyond markers. |

## Validation expectations

After material changes to schemas or audit output:

```bash
cargo test -p jankurai
just compat
just security
just fast
```

For report contract drift, `audit_smoke::audit_report_serializes_against_repo_score_schema` asserts emit JSON validates against `ArtifactSchema::RepoScore`. **`report_compatibility_guard`** additionally runs a full audit with sidecar exports on the jankurai repo and checks SARIF, JUnit XML, GitHub summary Markdown, repair-queue JSONL, **`findings[]` vs `finding.schema.json`**, **`issues export` JSONL** vs the same schema, and stable Markdown section headings in `repo-score.md`. For routing maps and policy manifests, `schema_contracts::agent_control_plane_schemas_parse_and_repo_fixtures_validate` asserts committed `agent/owner-map.json`, `agent/test-map.json`, `agent/generated-zones.toml`, `agent/proof-lanes.toml`, `agent/standard-version.toml`, and `agent/audit-policy.toml` validate against their schemas. Bundled init profiles: `profiles::bundled_profile_contract` (unit tests in `profiles.rs`).
