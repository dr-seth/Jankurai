# Supplementary Runtime Companion Note

Status: supplement artifact for the MSS paper. This document preserves the engineering taxonomy and local corpus notes behind the main-paper section on residual runtime waste.

## Purpose

The main paper keeps only a short carrier-centric runtime bridge plus five paper-facing stages:

1. `Suppress`
2. `Compress`
3. `Reuse`
4. `Steer`
5. `Guard`

This artifact preserves the fuller engineering breakdown, the hook surfaces in the WarpOS proxy pipeline, the local waste fingerprint for the observed agents plus the Quorp clean control, the removed chart-derivation notes, and the role of CrateAtlas as localization substrate.

## Main-Paper Mapping

| Main-paper stage | Engineering buckets merged | Meaning |
| --- | --- | --- |
| `Suppress` | `A + D + I` | Remove or short-circuit unnecessary, duplicated, wrong-routed, or inert requests. |
| `Compress` | `B + C.1 + C.2 + B.4` | Preserve decisive evidence while shrinking tool output, tool schemas, and repeated environment context. |
| `Reuse` | `C.3` plus `cargo-obligation-cache` | Avoid paying again for equivalent forward passes or proof obligations when state is conservatively unchanged. |
| `Steer` | `E + G + B.2` | Change the next action before the agent wastes a turn. |
| `Guard` | `F + H` | Prevent destructive or false-grounded actions even when they are token-cheap. |

## Full Engineering Taxonomy

| Bucket | Name | What it covers |
| --- | --- | --- |
| `A` | Request/response deduplication and catalog caching | in-flight coalescing, TTL-safe read caching, plugin catalog stubbing, retry dampening |
| `B` | Structural compression of tool results | Cargo packets, nextest routing, shell packets, SSE compaction |
| `C` | Content-addressed delta compression | schema dedup, environment-context deltas, semantic checkpoints |
| `D` | Protocol-side suppression | inert bootstrap filtering, analytics suppression, control-plane short-circuiting |
| `E` | Task-conditioned context steering | slice certificates, negative-context guidance, widen-lease synthesis |
| `F` | Tool-call firewall | destructive argv interrupts, path-scope guards, dirty-worktree checks, stall breakers |
| `G` | Hazard-gated online steering | per-turn hazard scoring, patch-proposal nudges, validation suppression |
| `H` | Ground-truth consistency checking | hallucination checks, fake-test-pass detection, diff-apply ambiguity checks |
| `I` | Wrong-route short-circuit | model / endpoint / auth consistency checks |

## Hook Points in the Proxy Pipeline

These are the concrete surfaces named in the main paper.

| Hook point | Pipeline location | Typical stages |
| --- | --- | --- |
| pre-dial request middleware | before upstream request leaves the proxy | `Suppress`, `Reuse` |
| request/body parse | normalized prompt and request-shape parsing | `Compress`, `Steer` |
| response/body parse | decoded response after transport normalization | `Suppress`, `Compress` |
| SSE / tool-call / tool-result intercept | per-event stream rewrite or analysis | `Compress`, `Steer`, `Guard` |
| exec boundary | before local command or file mutation runs | `Guard` |
| end-of-turn scoring | after a turn finishes and telemetry is assembled | `Reuse`, `Steer` |

## Per-Agent Waste Fingerprint

The main paper keeps only the grouped opportunity chart. This table preserves the local corpus fingerprint behind it.

Sources:

- `paper/four-agent-telemetry-study/generated/2026-04-18-five-agent-quorp-paper/data/agent_overview.csv`
- `.../savings_opportunity_summary.csv`
- `.../prompt_envelope_summary.csv`
- `.../path_tokens.csv`

| Agent | Raw tokens | Task-lane share | Dominant waste class | Key observed anchors |
| --- | --- | --- | --- | --- |
| Claude | 302,950 | 10.8% | platform bootstrap + catalog tax | `/api/event_logging/v2/batch` 151,216; `/mcp-registry/v0/servers` 84,416; tool schema replay across 19 task calls |
| Codex | 1,891,748 | 20.0% sealed / 80.0% broader scope | plugin catalog + response/control tax | `/backend-api/codex/responses` 1,513,470; `/connectors/directory/list` 185,392; `/otlp/v1/metrics` 64,323; six identical 107,943-byte POSTs within 11 s |
| Cursor | 334,840 | 0.1% | analytics telemetry chatter (BootstrapStatsig-dominated, shadow-mode caveat) | `BootstrapStatsig` 331,857; TrackEvents 1,544; SubmitLogs 612 |
| Antigravity | 133,357 | 90.0% | task-adjacent control + context replay | `/v1internal:streamGenerateContent` 120,026; `/listExperiments` 8,772; opening repo-tree dump repeatedly reattached |
| Quorp (off-chart control) | 15,992 | 97.4% | clean control | mostly `/quorp/v1/chat/completions`; little removable overhead |

## Residual Opportunity Derivation

Earlier drafts used a per-agent opportunity chart. The main paper no longer carries that figure, but the local derivation notes remain here for the supplementary runtime record.

- `task-core / irreducible task lane`
- `Suppress` (deterministic, request/response-side)
- `Compress` (deterministic, content-side)
- `Projected` (conditional, hatched)

### Interpretation

- `task-core` is the residual work expected to remain after the full runtime cascade.
- `Suppress` is the deterministic band attributable to request/response dedup, catalog caching, bootstrap filtering, retry damping, and wrong-route short-circuiting (buckets A + D + I in the engineering taxonomy).
- `Compress` is the deterministic band attributable to structural tool-output packets, schema dedup, environment-context delta, and SSE compaction (buckets B + C.1 + C.2 + B.4).
- `conditional projected opportunity` covers stages that require stronger assumptions:
  - conservative state reuse (C.3)
  - hazard-gated steering (E + G)
  - learned or policy-shaped guards beyond deterministic interrupts (F + H)

### Figure values used in the main paper (four-segment decomposition)

| Agent | Task-core | Suppress | Compress | Projected | Cumulative saving |
| --- | --- | --- | --- | --- | --- |
| Claude | 11% | 60% | 16% | 13% | 89% |
| Codex | 46% | 20% | 18% | 16% | 54% |
| Cursor | 1% | 20% | 20% | 59% (shadow) | 99% |
| Antigravity | 66% | 6% | 16% | 12% | 34% |
| Quorp (off-chart control) | 97% | 0% | 0% | 3% | 3% |

### Local derivation notes

- Claude Suppress is dominated by `platform_bootstrap_tax` (~50% of raw, driven by `/api/event_logging/v2/batch`) and `plugin_catalog_tax` (~28%, driven by `/mcp-registry/v0/servers` plus dedup/coalesce). Claude Compress is driven by tool-schema replay across 19 task calls (~1.35 MB duplicated) plus modest env-context delta.
- Codex Suppress is driven by plugin-catalog stubbing, retry-damper coalescing of six identical 107,943-byte POSTs, metrics short-circuit, and wrong-model route sanity. Codex Compress is dominated by tool-schema replay (~1.05 MB across 18 turns) plus cargo/nextest/shell packets and SSE reasoning-delta compaction.
- Cursor Suppress is derived from the definitely-inert analytics traffic (`TrackEvents`, `SubmitLogs`, and the initial-bootstrap portion of `BootstrapStatsig` that can be coalesced). Cursor Compress is structural compression of repeated Statsig response fields. The large projected band reflects the majority of `BootstrapStatsig` which is held under shadow-mode evaluation because Statsig responses may gate feature flags that unlock the task lane itself.
- Antigravity Suppress is smaller because most control traffic (`listExperiments`, `retrieveUserQuota`, metrics) is individually small. Antigravity Compress is dominated by environment-context delta on the opening ~7,400-character repo-tree dump reattached on every subsequent turn.
- Quorp is retained as the clean control: near-zero deterministic overhead and only a small (≤3%) projected band aligned with catalog Section 1 aggressive ceiling; it is deliberately omitted from the main-paper chart so the four visible bars preserve resolution.

These are local corpus-bounded opportunity estimates. They are not validated deployment outcomes and should not be generalized beyond the WarpOS capture without further study.

## Mistake-Reduction Inventory

The main paper separates token opportunity from mistake reduction. This table preserves that distinction.

### Deterministic safety effects

| Mechanism family | Primary effect |
| --- | --- |
| wrong-model / wrong-route short-circuit | prevents avoidable retry loops before provider contact |
| retry-storm suppression | stops repeated identical requests from amplifying cost and confusion |
| destructive argv firewall | blocks repo- or machine-damaging commands |
| dirty-worktree guard | avoids overwriting concurrent user edits |
| fake-test-pass detection | prevents green claims without observed proof |

### Projected steering effects

- Offline ISSUE-05 replay ceiling: `18/34` invalid runs prevented.
- Main-paper rule: treat that number as a ceiling for online steering, not as deployed performance.
- Any runtime use of hazard-gated steering should launch in shadow mode first and be reported as projected until online falsification exists.

## CrateAtlas Integration Notes

CrateAtlas is not a runtime stage and not a new flagship concept. It is a graph substrate that can supply:

- owner ranking for `cargo-mss` and runtime slice certificates
- dependency and reference slicing for negative-context or ignore certificates
- top-`K` path ranking for shell-packet summaries
- symbol/path existence checks for hallucination guards
- test and module topology for repair-capsule generation

Paper-facing phrasing:

- good: “CrateAtlas-class content-addressed repository graphs can feed localization and guardrails”
- avoid: “CrateAtlas is itself the next flagship system”

## Rollout Order

The runtime cascade should ship in this order:

1. deterministic `Suppress`
2. deterministic `Compress`
3. conservative `Reuse`
4. shadow-mode `Steer`
5. safety-first `Guard`, with rule-based checks ahead of learned ones

## What Stays Out of the Main Paper

The main paper should not carry:

- the full A–I intervention cards
- per-path token tables
- per-intervention crate ownership
- label-collection plans
- rollout playbooks
- hazard-model feature manifests

Those details belong here, in this artifact, or in the original WarpOS engineering notes.
