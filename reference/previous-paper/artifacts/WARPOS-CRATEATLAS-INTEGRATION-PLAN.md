# Final Integration Plan: Runtime Intervention Cascade (Suppress / Compress / Reuse / Steer / Guard)

Status: **final plan, executing now**. This supersedes my earlier three-plane draft. It is a synthesis of (a) the V10 plan (runtime-companion framing, verb-based stage names, replace-not-augment envelope) and (b) my earlier draft (touch-point matrix, per-agent measured numbers, CrateAtlas as substrate). The synthesis keeps V10's discipline (bounded single section, replace chart, paper identity unchanged) and keeps the best of my prior draft (per-agent measured numbers in the chart, hook-point precision, CrateAtlas treatment). Concepts: WarpOS MITM Intervention Catalog (9 buckets A–I, ~25 interventions) and CrateAtlas (`/Users/bentaylor/Code/CrateAtlas/`, 11-crate content-addressed Rust code-graph prototype).

## Core decisions

**1. Paper identity stays: MSS Rust repository standard paper.** WarpOS is a companion runtime layer, not a new paper thesis. No three-plane reframing. The abstract adds two sentences but does not rename the paper.

**2. One new bounded section, placed between Token Economy and Testing/Proof.** Title: **Runtime Intervention Cascade**. Target length ~1 page including one figure and one compact table. Renumber existing §8–§12 to §9–§13.

**3. Five-stage taxonomy using verb names (V10's best idea):** **Suppress / Compress / Reuse / Steer / Guard**. The 9 A–I buckets from the WarpOS catalog map into these five, and the full catalog stays in a supplement artifact:

| Stage | Folds in | What it does | Token classes attacked | ML required? |
|---|---|---|---|---|
| **Suppress** | A (dedup, catalog stub, retry damper), D (bootstrap filter, wrong-model shortcircuit), I (route sanity) | Remove or short-circuit requests that are unnecessary, duplicated, wrong-routed, or inert | instruction, navigation, proof-output | none |
| **Compress** | B (cargo packet, shell packet, SSE compactor), C.1 (schema dedup), C.2 (env-context delta) | Preserve decisive evidence while shrinking tool output, tool schemas, and repeated environment context | proof-output, wrong-turn, instruction | none (structural); small classifier for shell long tail |
| **Reuse** | C.3 (semantic-checkpoint), bridges to `cargo-obligation-cache` flagship | Avoid paying again for equivalent forward passes or proof obligations when state is conservatively unchanged | proof-output, retry | hazard head for safety gate |
| **Steer** | E (slice certificate, widen-lease synth), G (turn-hazard-gate, patch-proposal-nudge, validation-suppressor), B.2 (nextest router) | Alter the next action before the agent wastes a turn | navigation, file-read, patch, reasoning | yes |
| **Guard** | F (argv firewall, path-scope firewall, dirty-worktree guard, stall-breaker), H (hallucination check, test-pass-truth, diff-applies-check) | Prevent destructive or false-grounded actions even when they are token-cheap | wrong-turn, patch, review-transfer | none for F/H |

**Why verb names beat letter labels.** Suppress/Compress/Reuse/Steer/Guard name *what is done to the agent↔LLM stream*. A reviewer can hold all five in memory after one read. "M1–M5" cannot. This is V10's call and it is correct.

**4. Figure 2 is replaced, not augmented.** The existing doctrinal envelope (KBP → KBP+RI → KBP+RI+FH) is removed from the main paper and preserved in the supplement. The new Figure 2 is a *measurement-backed opportunity envelope* derived from the five-agent WarpOS corpus. Per-agent horizontal bars show three segments:

- **Task-core** (solid dark): what remains after all interventions.
- **Deterministic removable** (solid medium): what Suppress + Compress + static-Reuse kill; derived from observed wire traffic.
- **Projected removable** (hatched): what ML-Reuse + Steer + Guard add on top; projected, bounded by the ISSUE-05 offline ceiling (18/34 invalid runs prevented, 1,242,298 optimistic tokens saved).

Legend explicitly says **solid = observed or directly derived from observed telemetry; hatched = projected intervention opportunity, not validated deployment outcome.**

**5. CrateAtlas cited as substrate, not flagship.** One paragraph in the new section: CrateAtlas-class content-addressed code graph (deterministic `NodeId([u8;32])`, 20 node kinds, 14 edge kinds, provenance-tagged edges with `Definite / Resolved / Heuristic / Textual` confidence) supplies owner ranking, dependency and reference slicing, top-K path ranking for shell-packet summaries, symbol/path existence checks for the Guard stage, and repair-capsule generation for `cargo-mss`.

**6. Flagship trio stays intact.** cargo-mss, ProofLens, cargo-obligation-cache remain the three flagships in §10 (renumbered §11). But their bridge to the runtime cascade is made explicit:

- **cargo-mss** = compile-time owner/contract/proof substrate that the **Steer** stage consumes at runtime via slice certificates.
- **ProofLens** = packet-contract substrate that the **Compress** stage operationalizes at runtime via cargo/shell packets.
- **cargo-obligation-cache** = conservative repeated-proof substrate that the **Reuse** stage operationalizes at runtime via semantic checkpoints.

This is V10's framing and it makes the trio stronger, not weaker. They go from "three speculative future systems" to "three compile-time substrates each of which already has a runtime operationalization in WarpOS."

## Best-case total-savings numbers (per-agent, derived from WarpOS corpus)

From `paper/four-agent-telemetry-study/generated/2026-04-18-five-agent-quorp-paper/data/*.csv` and the §13 table in the WarpOS catalog. Percentages are of baseline raw tokens per session.

| Agent | Raw tokens | Deterministic removable (Suppress + Compress + static-Reuse) | Projected removable (ML-Reuse + Steer + Guard) | Task-core remaining (best case) |
|---|---|---|---|---|
| Claude | 302,950 | ~230k (76%) | ~20k (+7%) | ~53k (17%) |
| Codex | 1,891,748 | ~720k (38%) | ~300k (+16%) | ~870k (46%) |
| Cursor | 334,840 | ~330k (99%) | marginal | ~5k (1%) |
| Antigravity | 133,357 | ~30k (22%) | ~15k (+12%) | ~88k (66%) |
| Quorp | 15,992 | 0 (0%) | ~2k (12%) | ~14k (88%) |

**Best-case total savings when all five stages operate on top of the MSS baseline:** 83% for Claude, 54% for Codex, 99% for Cursor, 34% for Antigravity, 12% for Quorp. The spread is the interesting part: Cursor is essentially all platform-tax chatter, Quorp is essentially clean, the other three sit between. The chart tells that story at a glance.

**Mistake reduction:** 18/34 invalid runs prevented (~53% on ISSUE-05, offline guard result). Online expectation with `abstain_margin=50`: ~0.7 of ceiling = ~12 prevented invalids per 34 runs. Presented as a **projected band, not a measured deployment outcome** per V10 discipline.

## Execution sequence (the edits I am making now)

1. **Replace Figure 2** (`fig:envelope`) with the new per-agent opportunity chart. New colors, new title, new legend. Keep the same float machinery (`figure*[!t]`, spans both columns).
2. **Rewrite §7.4** (`sec:envelope`): retitle to "Runtime Intervention Opportunity Envelope", rewrite the prose to present the cascade as measured-plus-projected not doctrinal-only, reference §8.
3. **Insert new §8 "Runtime Intervention Cascade"** before the existing §8 Testing/Proof:
   - §8.1 bridge paragraph (repo-side vs runtime-side waste)
   - §8.2 five-stage cascade table (Table 9)
   - §8.3 hook-point diagram + three action-type taxonomy
   - §8.4 per-agent measured fingerprint (Table 10, compact)
   - §8.5 CrateAtlas substrate paragraph
   - §8.6 connection to the flagship trio
4. **Renumber existing §§8–12 to §§9–13** (via labels; `\section` numbers auto-renumber via LaTeX).
5. **Update §10 (now §11) Future Work**: add one bridging paragraph tying each flagship to the runtime-cascade stage it operationalizes.
6. **Update abstract**: +2 sentences referencing the measured opportunity.
7. **Update conclusion (§12, now §13)**: one-sentence addition connecting runtime cascade to the thesis.
8. **Add supplement artifact** `paper/artifacts/warpos-runtime-cascade.md` carrying the full A–I catalog, per-intervention details, hook-point table, per-agent fingerprint derivation, label and feature inventory, and the preserved doctrinal envelope.
9. **Add references** to `references.bib`: `warpos2026` (project-reported), `crateAtlas2026` (project-reported), `mitmproxy2026`, `blake3_2026`.
10. **Compile, verify, copy preview PDF.**

## Answer to the specific organizational question

> *"Do we need a 4th? 5th? or can we merge these into existing categories?"*

Three merges and two new stages is the right answer. The existing paper has token-class buckets (instruction / navigation / file-read / reasoning / patch / proof-output / wrong-turn / review-transfer) — those are **what is wasted**. The runtime cascade introduces five **how waste is eliminated** stages — a different axis. Suppress / Compress / Reuse are the three that merge cleanly with existing paper content (Suppress + Compress extend ProofLens's scope; Reuse extends cargo-obligation-cache). Steer and Guard are genuinely new categories the paper does not have today; Steer corresponds to no existing section, and Guard is a first-class safety concern the paper only touches via the "Safe-compression rule" in §7. So:

- **Not new top-level buckets** — they are a second axis of the same paper.
- **Steer and Guard are new stages** that need their own naming in the manuscript.
- **Suppress, Compress, and Reuse** connect directly to existing flagships, not replacing them.

## Page budget estimate

Current paper: 18 pages under the article-class sandbox, ~10 pages under IEEE conference class.

Added content:
- Abstract: +2 sentences → neutral (displaced by §7.4 rewrite).
- §7.4 rewrite: similar length, replaces Figure 2 and surrounding prose → neutral.
- New §8 Runtime Intervention Cascade: ~1 page including Table 9 (wide, spans two columns) and Table 10 (compact, single column).
- §11 (renumbered) one bridging paragraph: +120 words, ~1/5 column.
- Conclusion: +1 sentence.

Net: roughly +1 page. Paper moves from 10 → 11 pages under IEEE. Inside typical conference submission tolerance.

---

*Plan end. Proceeding to execution.*
