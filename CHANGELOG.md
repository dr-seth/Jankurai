# Changelog

All notable user-facing changes should be recorded here.

Jankurai is pre-1.0. Public CLI behavior, report schemas, generated scaffold paths, and agent-facing contracts should still receive compatibility notes when they change.

## Unreleased

No user-facing changes yet.

## 0.8.12 - 2026-05-07

### Added

- Added certified reuse-registry cells for periodic cron jobs and billing subscriptions, including example Rust boundaries, OpenAPI contracts, migration/constraint evidence, docs, ops notes, UX route notes, schema coverage, and smoke tests.
- Added `jankurai version`, `jankurai versions`, `jankurai upgrade --score`, and update receipt schema coverage for version-aware local upgrades.

### Changed

- Bumped the auditor/action package release to `0.8.12`; standard compatibility remains `0.8.0` and report schema is `1.6.1`.
- Updated the release docs and version manifests to reflect `jankurai version`, `jankurai versions`, and `jankurai upgrade --score` behavior.
- Retagged the GitHub Action reference to `v0.8.12`.
- Replaced Tuiwright bitmap rendering with rusttype plus bundled JetBrains Mono for anti-aliased screenshots.

### Fixed

- Fixed Tuiwright missing Unicode box drawing glyphs in rendered output.
- Made line-based scaffold merges recipe-aware so `Justfile` updates do not append commands from already-existing recipes as orphan lines.
- Updated scaffold merge behavior so `agent/standard-version.toml` refreshes canonical version keys instead of keeping stale auditor/schema metadata.

## 0.8.11 - 2026-05-06

### Added

- Added reference-profile structure audit output and migration steering for detected canonical cells.
- Added `HLT-039-WEB-SECURITY-BAD-BEHAVIOR` with high-confidence detectors for exposed Vite dev servers, client-exposed Vite secrets, browser token storage, and credentialed wildcard CORS.
- Added `HLT-040-REPO-ROT-BAD-BEHAVIOR` with active-source old/backup/copy/archive path checks plus soft review signals for commented-out code blocks and hard-disabled branches.
- Added focused coverage for risky and safe web-security and repo-rot cases, including false-positive guards for docs, tips, reference, tests, generated output, API versions, and DB migrations.

### Changed

- Hardened `jankurai upgrade` for source-checkout upgrades: `--source auto` now prefers a newer local `crates/jankurai` package over registry lookup and reinstalls into the current Cargo root instead of a nested `bin` path.
- Bumped the auditor/action package release to `0.8.11`; standard compatibility remains `0.8.0` and report schema is `1.6.0`.

## 0.8.10 - 2026-05-06

### Added

- Added default audit inventory exclusion for `tips/`, plus user-configurable `[scan] excluded_paths` entries in `agent/audit-policy.toml`.
- Added bounded score history commands: `jankurai history latest/export/compact/restore`, plus bounded audit retention and optional mirror sink support.
- Added May 6 public-repository paper evidence, score tables, and a README score table for the `v0.8.8` Marketplace action release.
- Added accepted-baseline ratchet scaffolding and strict scoring-integrity smoke tests for fail-closed audit decisions.

### Changed

- Routed `jankurai score trend` through the shared score-history loader and added stable score-history entry/export schemas.
- Bumped the auditor/action package release to `0.8.10`; standard compatibility remains `0.8.0` and report schema remains `1.5.0`.
- Hardened CI scoring order, required proof/security evidence, SHA-pinned Actions usage, SARIF upload, and badge source routing for release readiness.
- Fixed the isolated empty-repository ratchet regression so `decision.ratchet.score_delta` is always emitted, including `--no-score-history` runs.
- Prepared the `v0.8.10` GitHub Marketplace action release for the hardened scoring-integrity lane.
- Scoped crates.io publication out of this Marketplace release until the proof crates are published first.

## 0.8.0 - 2026-05-05

### Added

- Added the GitTools bad-behavior policy surface, research note, detector family, fixtures, and stable `HLT-036-GITTOOLS-BAD-BEHAVIOR` rule.
- Added the `gittools-bad-behavior` hard cap for high-confidence hook-manager and Git tooling hazards.

### Changed

- Bumped the standard and auditor release to `0.8.0` and the paper edition to `2026.05-ed8`; report schema remains `1.5.0`.
- Reframed the paper around Jankurai as a versioned agent-native repository standard and bumped the paper edition to `2026.05-ed6`.
- Fixed generated adapter templates so every generated adapter satisfies the startup update marker verification and shows a valid client-start command.
- Fixed Marketplace action packaging so external consumers install the CLI from the action checkout, and documented `v0.8.0` GitHub Action usage, inputs, artifacts, and local runner behavior.

## 0.6.1 - 2026-05-04

### Changed

- Hardened vibe coverage taxonomy with reviewed canonical groups, detector/evidence status fields, and `0` uncovered source rows.
- Downgraded broad `absolute` claims to `partial` unless backed by detector and audit evidence.
- Strengthened `jankurai vibe validate` for title matching, duplicate/missing row checks, known rule/tool/lane references, reviewed rows, and absolute-evidence requirements.
- Added semantic coverage fixtures and HLT-022 through HLT-027 detector fixtures.
- Regenerated the paper coverage table with short rule labels and a separate legend.

## 0.6.0 - 2026-05-04

### Added

- Vibe coverage registry in `agent/vibe-coverage.toml` mapping all 260 `tips/vibe_coding` source rows.
- `jankurai vibe validate` and `jankurai vibe coverage` for JSON, Markdown, and generated TeX coverage reports.
- Optional repo-score `vibe_coverage` summary and stable `## Vibe Coding Coverage` Markdown section.
- Generated paper appendix table with green/yellow/red coverage status.
- Conditional MASTER_PLAN/phase adapter routing for explicit phase work only.

- v0.6.0 trustworthy-merge surface: `jankurai witness`, `jankurai score diff`, `jankurai score trend`, `jankurai rules export`, and `jankurai rules verify`.
- Merge witness, score diff/trend, rule registry, and rule-verify schemas.
- Token-budgeted context packs with source-trust labels and included/excluded file receipts.
- Baseline-required ratchet CI and audit behavior.
- Public `init --bootstrap-commit` and `--bootstrap-message` flags; hidden `--yolo` aliases remain for compatibility with deprecation warnings.
- Public open-source README structure with install, safe trial, adoption, update, AI-agent risk, support, security, license, and citation sections.
- Community health files for contributing, security, conduct, support, changelog, pull requests, and issues.
- Cargo package metadata for repository, homepage, README, keywords, and categories.
