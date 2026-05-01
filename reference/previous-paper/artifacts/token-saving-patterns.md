# Token-Saving Patterns for Agent-Efficient Rust

Status: supplement artifact for the V7 review-backed standard.

The paper's token thesis is simple: the biggest savings come from fewer wrong turns, narrower proof loops, and safer output shaping, not from aggressively shortening every message.

## Token-Savings Hierarchy

1. Prevent wrong-owner and wrong-contract exploration.
2. Route to the smallest sound proof lane.
3. Compress proof output without hiding decisive evidence.
4. Keep free-form narration terse.
5. Reduce review-transfer overhead with structured receipts.

## Structural Savings

These remove token cost before any search or summarization happens.

| Pattern | Token Buckets Reduced | Why It Helps | Must Not Worsen |
| --- | --- | --- | --- |
| Owner maps | navigation, file-read, wrong-turn | Agents stop opening unrelated crates and helpers | stale ownership metadata |
| Narrow crates/modules | navigation, reasoning, patch | Fewer reasons to change per surface | accidental over-fragmentation |
| Generated contracts | file-read, reasoning, review-transfer | One source of truth beats parallel handwritten copies | hidden generated drift |
| Legal edit zones | patch, wrong-turn | Reduces speculative cross-boundary edits | blocked legitimate migrations without escalation path |
| Proof-lane routing | proof-output, retry | Avoids whole-workspace thrash | under-testing risky changes |
| Unsafe ledgers | reasoning, security review | Makes hidden invariants explicit | false sense of proof |

## Navigation Savings

Use these before broad file reads.

| Pattern | Token Buckets Reduced | Mechanism | Guardrail |
| --- | --- | --- | --- |
| `cargo metadata --no-deps --format-version 1` | navigation | machine-readable workspace map | graph is not semantic ownership |
| `rg`-first search discipline | navigation | cheap lexical narrowing | do not stop at text-only evidence for semantic questions |
| Signature-first reads | file-read | inspect public types, trait bounds, and tests before whole files | escalate to full reads when invariants stay unclear |
| Repo-map/test-map usage | navigation, proof-output | link paths to proof lanes | keep maps fresh |
| Structural search with `ast-grep` | navigation, patch | match code shape, not just text | language grammar must match target |

## Proof-Output Savings

These are the highest-risk savings because they can create false-green summaries.

| Pattern | Token Buckets Reduced | Mechanism | Must Preserve |
| --- | --- | --- | --- |
| `cargo check --message-format=json` | proof-output | structured diagnostics | exit code, primary spans, error codes, notes |
| Failure-only nextest summaries | proof-output | only failing tests and panic info | test names, panic text, seeds, raw log path |
| Grouping diagnostics by crate/error code | proof-output, recovery | reduces repetition | owner clue and decisive span |
| Tracing span summaries | proof-output, recovery | collapse noisy logs into causal path | span path, request ID, raw log handle |
| Generated contract diffs | file-read, proof-output | show source-to-output drift only | source file and generated artifact identity |
| Raw-output tee files | proof-output | keep full logs off the model path | stable file path, hash, redaction status |

## Build-Loop Savings

These reduce retries by making proof cheap enough to run.

| Pattern | Token Buckets Reduced | Mechanism | Caveat |
| --- | --- | --- | --- |
| Targeted package checks | proof-output, retry | avoid validating unrelated crates | escalate when public contracts change |
| Stable `CARGO_TARGET_DIR` | recovery | reuses artifacts across loops | ensure isolation where required |
| Incremental compilation | retry | faster local rebuilds | not a substitute for scoping |
| `sccache` | retry | compiler cache reuse | measure first |
| `cargo build --timings` | reasoning, retry | exposes bottlenecks | diagnostics only, not speedup by itself |
| workspace `default-members` | proof-output | narrows default proof scope | keep release lane explicit |
| `cargo-chef` for containers | setup, retry | caches dependency layers in OCI builds | not a local inner-loop fix |
| `cargo-binstall` | setup | faster binary-tool bootstrap | pin and trust carefully |

## Low-Token Narration Policy

This is the formal version of terse-agent practice. The point is not macho minimalism. The point is to cut chatter without cutting proof.

- Be concise in prose.
- Skip motivational filler and repeated restatement.
- Summarize first; preserve raw evidence by path or ID.
- Never compress code, commands, paths, identifiers, error codes, test names, seeds, advisory IDs, or exit codes.
- Never summarize a security failure as green-looking.
- Prefer failure-first proof summaries and signature-first code summaries.

## Review-Transfer Savings

Humans pay token-like costs too. Re-review and reconstruction are expensive.

| Pattern | Token Buckets Reduced | Mechanism | Must Not Worsen |
| --- | --- | --- | --- |
| Proof receipts | review-transfer | state owner, proof lane, commands, raw evidence | stale or fabricated receipts |
| Structured patch summaries | review-transfer | highlight changed owners and contracts | over-trusting summaries instead of diff review |
| Generated diff summaries | file-read, review-transfer | show only source-of-truth boundary drift | hidden breaking field removal |
| Stable diagnostic codes | recovery, review-transfer | make failures comparable across runs | generic or non-actionable codes |

## What Never Counts as a Valid Savings

- Hiding a panic, advisory, failing test, seed, or raw-output path.
- Lowering visible tokens by skipping the required security lane.
- Replacing proof with prose.
- Shorter summaries that increase reviewer reconstruction time.
- Lowering context by forcing the agent into extra wrong-owner loops.
