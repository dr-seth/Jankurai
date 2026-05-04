# Changelog

All notable user-facing changes should be recorded here.

Jankurai is pre-1.0. Public CLI behavior, report schemas, generated scaffold paths, and agent-facing contracts should still receive compatibility notes when they change.

## Unreleased

### Added

### Changed

- Reframed the paper around Jankurai as a versioned agent-native repository standard and bumped the paper edition to `2026.05-ed6`.

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
