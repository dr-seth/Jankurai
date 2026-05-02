# Phase 03: Proof Router And Evidence Ledger

Status: complete
Owner: tools
Last reviewed: 2026-05-02
Parallel MCP candidate: partial

## Objective

Make Humanlint answer the question agents currently guess at: which proof actually covers this change?

This phase builds the changed-path proof router and a normalized evidence ledger. A future agent should be able to run one command and receive a deterministic plan:

```text
Changed paths -> owners -> lanes -> commands -> artifacts -> merge decision
```

## Current State

Existing pieces:

- `agent/test-map.json` maps paths to commands and purposes.
- `agent/proof-lanes.toml` defines lane names and command strings.
- `humanlint lane` and `humanlint proof` build and optionally write a normalized proof plan (`schemas/proof-plan.schema.json`), including `planned_runs`, `skipped_lane_entries`, and routing risk metadata.
- `humanlint prove` executes planned commands, writes receipts under `target/humanlint/proof-receipts/`, command logs under `target/humanlint/logs/`, and `target/humanlint/evidence-index.json` (`schemas/evidence-index.schema.json`). The evidence index **`schema_version` is `1.2.0`** and, when those files exist at index write time, may include **optional repo-relative** `ux_qa_report_path` (`target/humanlint/ux-qa.json`), `security_evidence_path` (`target/humanlint/security/evidence.json`), `repo_score_json_path` (`agent/repo-score.json`), `sarif_path` (`target/humanlint/humanlint.sarif`), `github_step_summary_path` (`target/humanlint/summary.md`), and `repair_queue_jsonl_path` (`target/humanlint/repair-queue.jsonl`). Execution allowlists commands to the union of proof-lanes and test-map unless `--allow-unsigned-commands` and `HUMANLINT_ALLOW_UNSIGNED_PROOF_COMMANDS=1`.
- `humanlint prove` accepts either `--plan <path>` or changed-path inputs through `--changed` / `--changed-from`, reusing the same planner and runner path.
- `audit --changed` and `--changed-from` exist.
- `humanlint audit --proof-receipts` loads receipt JSON into `Report.proof_receipts`.
- `ProofReceipt.rules_covered` now has a compatibility-preserving rich/simple representation. Current proof execution keeps unknown or custom rule coverage empty rather than guessing from command text.
- `doctor` writes local receipts under `target/humanlint/receipts` and validates proof receipts plus evidence index when present (schema + optional stale `git_head` warning).
- `report` modules emit JSON, Markdown, SARIF, GitHub summary, JUnit-ish output, and repair queue JSONL.

Residual hardening:

- Persist generated changed-mode proof plans as a file-backed artifact if later release evidence requires a reusable plan path instead of the current in-memory marker.
- Add non-empty deterministic `rules_covered` only where lane-to-rule contracts are explicit enough to avoid false coverage claims.
- Expand evidence index cross-links beyond the standard report bundle when future phases define new artifact types.

## Residual Hardening Plan

This is the worker-ready plan for post-completion hardening. It assumes a strong planner has routed the work and a weaker implementation agent will execute the steps. The next logical implementation order is:

1. Add first-class `humanlint prove --changed` / `--changed-from` as a thin bridge over the existing planner and proof runner.
2. Add deterministic rule-ID linkage into proof runs without changing existing report JSON compatibility.
3. Update docs, phase receipt, and logs only after validation proves the behavior.

### Objective

Harden completed Phase 03 so changed-path proof execution leaves a more reusable plan artifact and safer rule-coverage metadata:

```bash
humanlint prove . --changed crates/humanlint/src/commands/proof.rs
humanlint prove . --changed-from origin/main
```

Hardening completion criteria:

- `prove --plan <path>` keeps existing behavior.
- `prove --changed <path>` and `prove --changed-from <ref>` build a proof plan, persist JSON and Markdown plan artifacts, execute the planned allowlisted commands, write receipts, and write an evidence index.
- Proof receipts populate `rules_covered` where deterministic route metadata exists; unknown linkage remains empty, not guessed.
- Phase docs move the shorthand out of "Missing pieces" and record remaining limitations honestly.
- `cargo test -p humanlint`, focused proof smoke tests, and `just fast` pass.

Non-goals:

- Do not build an MCP server.
- Do not execute commands outside the existing proof-lanes/test-map allowlist.
- Do not make audit scoring depend on proof receipts in this slice unless a separate release-mode policy change is approved.
- Do not hand-edit generated artifacts under `target/` or declared generated zones.

### Read First

Read in this order before editing:

1. `agent/HUMANLINT_STANDARD.md`
2. `agent/MASTER_PLAN.md`
3. `tips/phases/00-phase-index.md`
4. `tips/phases/03-proof-router-evidence-ledger.md`
5. `tips/phases/logs/03-proof-router-evidence-ledger.log`
6. `agent/owner-map.json`
7. `agent/test-map.json`
8. `agent/proof-lanes.toml`
9. `agent/generated-zones.toml`
10. `agent/standard-version.toml`
11. `crates/humanlint/src/main.rs`
12. `crates/humanlint/src/commands/proof.rs`
13. `crates/humanlint/src/commands/context_data.rs`
14. `crates/humanlint/src/audit/rules.rs`
15. `crates/humanlint/tests/proof_surface_smoke.rs`
16. `crates/humanlint/tests/schema_contracts.rs`
17. `schemas/proof-plan.schema.json`
18. `schemas/proof-receipt.schema.json`
19. `schemas/evidence-index.schema.json`
20. `docs/testing.md`

### Ownership

Owned paths for this phase:

- `crates/humanlint/src/main.rs`
- `crates/humanlint/src/commands/proof.rs`
- `crates/humanlint/tests/proof_surface_smoke.rs`
- `crates/humanlint/tests/schema_contracts.rs` only if schema assertions change
- `docs/testing.md`
- `tips/phases/03-proof-router-evidence-ledger.md`
- `tips/phases/logs/03-proof-router-evidence-ledger.log`
- `target/humanlint/phase03-*` proof artifacts

Forbidden or high-conflict paths:

- `reference/` is read-only.
- `paper/` is out of scope for Phase 03.
- `agent/test-map.json` and `agent/proof-lanes.toml` are shared contracts. Do not change them unless the implementation cannot be expressed with existing commands.
- `schemas/proof-plan.schema.json`, `schemas/proof-receipt.schema.json`, and `schemas/evidence-index.schema.json` should not change for `prove --changed`; the current data model is sufficient.
- Do not edit generated `target/humanlint/` artifacts by hand.

Concurrent-agent risks:

- Phase 05 and Phase 06 agents may touch evidence index companion paths. Keep Phase 03 additions additive and optional.
- Phase 08 agents may touch context/repair plan schema. Do not mix context-pack or repair-plan work into this phase.
- Rust CLI agents may also edit `main.rs`; inspect the latest file before patching and keep the diff scoped to `ProveArgs` and dispatch.

### Current State

Already implemented:

- `humanlint lane` and `humanlint proof` build plans from `--changed` and `--changed-from`.
- `build_proof_plan(repo, changed, changed_from)` in `crates/humanlint/src/commands/proof.rs` already supports changed paths and git refs.
- `humanlint prove --plan <path>` validates a persisted plan and executes allowlisted commands.
- Receipts include lane, command, exit code, elapsed time, artifacts, changed paths, owner, residual risk, log path, receipt path, repo root, git head, run ID, plan path, retryability, byte length, and reserved `rules_covered`.
- Evidence index schema version `1.2.0` records plan, receipt, log, failed receipt, skipped lane, risk, changed path, UX, security, repo-score, SARIF, GitHub summary, and repair queue links when present.
- `doctor` validates proof receipts and evidence index when present.

Do not rebuild:

- Do not duplicate proof planning logic in `main.rs`.
- Do not create a second proof runner.
- Do not infer rule IDs from free-form command text.

### Implementation Steps

Step 1: Log start.

- Append a canonical start row to `tips/phases/logs/03-proof-router-evidence-ledger.log`.
- Use `not-run` for validation and `none` for artifacts until proof is complete.

Step 2: Extend CLI args in `crates/humanlint/src/main.rs`.

Change `ProveArgs` from mandatory `plan: String` to an optional plan plus changed-path inputs:

```rust
#[derive(Args, Debug)]
struct ProveArgs {
    #[arg(default_value = ".", value_parser = parse_repo_arg)]
    repo: PathBuf,
    #[arg(long, value_name = "PATH")]
    plan: Option<String>,
    #[arg(long, value_name = "PATH")]
    changed: Vec<PathBuf>,
    #[arg(long, value_name = "REF")]
    changed_from: Option<String>,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/humanlint/proof-plan.json"
    )]
    plan_out: String,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/humanlint/proof-plan.md"
    )]
    plan_md: String,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/humanlint/proof-receipts"
    )]
    out_dir: String,
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/humanlint/evidence-index.json"
    )]
    evidence_index: String,
    #[arg(long)]
    continue_on_error: bool,
    #[arg(long = "allow-unsigned-commands")]
    allow_unsigned_commands: bool,
}
```

Then update the `Commands::Prove` dispatch to pass `plan`, `changed`, `changed_from`, `plan_out`, and `plan_md` into `proof::ProveArgs`.

Step 3: Mirror the command args in `crates/humanlint/src/commands/proof.rs`.

Update the public `ProveArgs` struct with the same new fields:

- `plan: Option<String>`
- `changed: Vec<PathBuf>`
- `changed_from: Option<String>`
- `plan_out: String`
- `plan_md: String`

Keep `out_dir`, `evidence_index`, `continue_on_error`, and `allow_unsigned_commands` unchanged.

Step 4: Refactor `run_prove`.

Extract plan loading and execution so plan-file mode and changed-path mode share the same runner:

```rust
pub fn run_prove(args: ProveArgs) -> Result<()> {
    let has_changed_input = !args.changed.is_empty() || args.changed_from.is_some();
    if args.plan.is_some() && has_changed_input {
        anyhow::bail!("use either --plan or --changed/--changed-from, not both");
    }

    let (plan, plan_path) = if let Some(plan_path) = args.plan.as_deref() {
        (load_proof_plan(&args.repo, plan_path)?, plan_path.to_string())
    } else if has_changed_input {
        if args.plan_out == "-" {
            anyhow::bail!("--plan-out must be a file path when prove builds a plan");
        }
        let plan = build_proof_plan(&args.repo, &args.changed, args.changed_from.as_deref())?;
        write_plan(&args.repo, &plan, Some(&args.plan_out), Some(&args.plan_md))?;
        (plan, args.plan_out.clone())
    } else {
        anyhow::bail!("provide --plan, --changed, or --changed-from");
    };

    execute_proof_plan(args, plan, plan_path)
}
```

Suggested helpers:

```rust
fn load_proof_plan(repo: &Path, plan_path: &str) -> Result<ProofPlan> {
    let plan_text =
        fs::read_to_string(plan_path).with_context(|| format!("read proof plan {plan_path}"))?;
    let plan_json: Value = serde_json::from_str(&plan_text)
        .with_context(|| format!("parse proof plan {plan_path}"))?;
    validation::validate_value(repo, ArtifactSchema::ProofPlan, &plan_json)?;
    Ok(serde_json::from_value(plan_json)?)
}

fn execute_proof_plan(args: ProveArgs, plan: ProofPlan, plan_path: String) -> Result<()> {
    // Move the existing run_prove body here after the old plan load.
}
```

Important details:

- `execute_run(..., plan_path.as_str())` must receive the actual persisted plan path.
- `ProofEvidenceIndex.plan_path` must use the same path.
- The plan must be written before command execution. If a proof command fails, the plan artifact must still exist for repair.
- Keep `ensure_planned_commands_allowed` before executing commands.

Step 5: Add route-to-rule linkage for receipts.

Add a deterministic function in `proof.rs` that maps a planned run to stable rule IDs without guessing from prose:

```rust
fn rules_covered_for_run(run: &PlannedRun) -> Vec<String> {
    let mut rules = Vec::new();
    match run.lane.as_str() {
        "fast" => {
            push_rule(&mut rules, "HLT-003-OWNERLESS-PATH");
            push_rule(&mut rules, "HLT-004-UNMAPPED-PROOF");
        }
        "audit" => {
            push_rule(&mut rules, "HLT-003-OWNERLESS-PATH");
            push_rule(&mut rules, "HLT-004-UNMAPPED-PROOF");
        }
        "contract" => {
            push_rule(&mut rules, "HLT-007-HANDWRITTEN-CONTRACT");
            push_rule(&mut rules, "HLT-002-GENERATED-MUTATION");
        }
        "db" => {
            push_rule(&mut rules, "HLT-006-DIRECT-DB-WRONG-LAYER");
            push_rule(&mut rules, "HLT-019-STREAMING-RUNTIME-DRIFT");
        }
        "web" => {
            push_rule(&mut rules, "HLT-013-RENDERED-UX-GAP");
            push_rule(&mut rules, "HLT-014-A11Y-GAP");
        }
        "security" => {
            push_rule(&mut rules, "HLT-009-GENERATED-SECURITY");
            push_rule(&mut rules, "HLT-010-SECRET-SPRAWL");
            push_rule(&mut rules, "HLT-011-PROMPT-INJECTION");
            push_rule(&mut rules, "HLT-012-OVERBROAD-AGENCY");
            push_rule(&mut rules, "HLT-016-SUPPLY-CHAIN-DRIFT");
        }
        "observability" => push_rule(&mut rules, "HLT-017-OPAQUE-OBSERVABILITY"),
        _ => {}
    }
    rules.retain(|rule_id| crate::audit::rules::lookup(rule_id).is_some());
    rules
}
```

Implementation note:

- If `proof.rs` does not currently import `crate::audit::rules`, use fully qualified lookup to keep imports small.
- Do not populate unknown or synthetic rule IDs.
- For test-map custom lanes such as `fixture`, `rules_covered` should remain empty.
- Assign this vector in `execute_run` when building `ProofReceipt`.

Step 6: Add focused proof tests in `crates/humanlint/tests/proof_surface_smoke.rs`.

Add or update fixture catalog data:

- Ensure an owner route exists for `fixtures/`.
- Ensure a test-map route exists for `fixtures/` with command `true`.
- Ensure proof-lanes has a named lane for `true`, such as `fixture`.

Required tests:

- `prove_changed_builds_plan_runs_and_indexes_evidence`
  - create `fixtures/demo.txt`
  - run `humanlint prove <repo> --changed fixtures/demo.txt --plan-out <tmp>/proof-plan.json --plan-md <tmp>/proof-plan.md --out-dir <tmp>/proof-receipts --evidence-index <tmp>/evidence-index.json`
  - assert status success
  - assert plan JSON exists and validates
  - assert plan Markdown exists and contains `humanlint Proof Plan`
  - assert one receipt exists and validates
  - assert evidence index validates and `plan_path` equals the `--plan-out` path

- `prove_changed_from_builds_plan_runs_and_records_base_ref`
  - initialize a temp git repo
  - configure local user name/email
  - create base commit
  - modify or add a fixture file and commit again
  - run `humanlint prove <repo> --changed-from <base_sha> ...`
  - assert plan JSON `base_ref` is `<base_sha>`
  - assert changed path includes the fixture path

- `prove_requires_plan_or_changed_input`
  - run `humanlint prove <repo>` with no plan or changed flags
  - assert failure and stderr contains `provide --plan, --changed, or --changed-from`

- `prove_rejects_plan_combined_with_changed`
  - run with `--plan <existing-plan> --changed fixtures/demo.txt`
  - assert failure and stderr contains `use either --plan or --changed/--changed-from`

- `prove_changed_rejects_stdout_plan_out`
  - run with `--changed fixtures/demo.txt --plan-out -`
  - assert failure and stderr contains `--plan-out must be a file path`

- `prove_receipts_include_rules_for_named_lanes`
  - use a plan or route whose planned run lane is `security` or `audit`
  - assert receipt `rules_covered` contains only IDs registered in `rules.rs`
  - do not require `fixture` runs to carry rules

Step 7: Update docs.

In `docs/testing.md`, add a concise section for proof execution:

```markdown
### Proof Execution

Use `humanlint lane` when you only need the plan. Use `humanlint prove` when you want receipts and an evidence index.

```bash
cargo run -p humanlint -- prove . --changed crates/humanlint/src/commands/proof.rs
cargo run -p humanlint -- prove . --changed-from origin/main
```

Changed-path `prove` writes `target/humanlint/proof-plan.json`, `target/humanlint/proof-plan.md`, proof receipts under `target/humanlint/proof-receipts/`, logs under `target/humanlint/logs/`, and `target/humanlint/evidence-index.json`. `--changed-from` uses committed git diff only; pass explicit `--changed` paths for uncommitted work.
```

In this phase file:

- Move first-class `prove --changed` out of Missing pieces after implementation.
- Keep rule-ID linkage in Current State only after it is implemented.
- If only `prove --changed` lands, leave Phase 03 `Status: partial`.
- If `prove --changed`, deterministic `rules_covered`, docs, and validation all land, mark `Status: complete` only if no material Phase 03 acceptance criterion remains unimplemented.

Step 8: Validate.

Run focused tests first:

```bash
rtk cargo test -p humanlint --test proof_surface_smoke prove_changed
rtk cargo test -p humanlint --test proof_surface_smoke
```

Run full local proof:

```bash
rtk cargo test -p humanlint
rtk just fast
```

Run a real command smoke from the repo:

```bash
rtk cargo run -p humanlint -- prove . \
  --changed crates/humanlint/src/commands/proof.rs \
  --plan-out target/humanlint/phase03-prove-changed-plan.json \
  --plan-md target/humanlint/phase03-prove-changed-plan.md \
  --out-dir target/humanlint/phase03-prove-changed-receipts \
  --evidence-index target/humanlint/phase03-prove-changed-evidence.json
```

Expected artifacts:

- `target/humanlint/phase03-prove-changed-plan.json`
- `target/humanlint/phase03-prove-changed-plan.md`
- `target/humanlint/phase03-prove-changed-receipts/*.json`
- `target/humanlint/phase03-prove-changed-evidence.json`
- `target/humanlint/fast-score.json`
- `target/humanlint/fast-score.md`

Step 9: Close the phase receipt.

Append a finish row to `tips/phases/logs/03-proof-router-evidence-ledger.log` with:

- changed paths
- validation commands and pass/fail result
- artifact paths
- current git SHA
- residual risk

Update the Phase Status Receipt in this file with the same facts.

### Hard Parts And Edge Cases

Path normalization:

- `normalize_changed_path` can turn repo root inputs into empty or broad paths. Reject empty, `"."`, and root-equivalent changed paths in `build_proof_plan` or before calling it from changed-mode `prove`.
- Keep absolute path handling compatible with existing `lane` behavior.

Plan path semantics:

- In `--plan` mode, the plan path is user-provided and may live outside `target/`.
- In changed mode, the plan path must be stable and file-backed because receipts and evidence index cite it.
- Do not support `--plan-out -` for `prove`; stdout plans are fine for planning commands, not for proof execution.

Git semantics:

- `--changed-from` should use the existing `changed_paths_from_git` behavior.
- Document that uncommitted changes are not included by `--changed-from`; use explicit `--changed` for worktree files.

Allowlist safety:

- Changed-mode plans must still pass `ensure_planned_commands_allowed`.
- Do not add a bypass path for generated plans.
- Keep the dual escape hatch for unsigned commands unchanged.

Rule linkage:

- Only populate `rules_covered` from deterministic lane-to-rule metadata.
- Do not parse shell command text to infer rules.
- Validate rule IDs through `rules::lookup` before including them.
- Keep unknown custom lanes empty rather than misleading.

Failure behavior:

- If one proof command fails and `--continue-on-error` is false, the failed receipt and evidence index should still be written for the attempted run, preserving existing behavior.
- If plan creation fails, no receipts should be written.

### Parallel Work Packets

Agent A: CLI bridge and proof runner

- Phase: 03
- Scope: implement `prove --changed` / `--changed-from`
- Owned paths: `crates/humanlint/src/main.rs`, `crates/humanlint/src/commands/proof.rs`, focused portions of `crates/humanlint/tests/proof_surface_smoke.rs`
- Forbidden paths: schemas unless a compile/test failure proves a schema mismatch; `agent/test-map.json`; `agent/proof-lanes.toml`; `paper/`; `reference/`
- Input contracts: existing `ProofPlan`, `ProofReceipt`, `ProofEvidenceIndex`, command allowlist
- Output contracts: existing `prove --plan` behavior unchanged; changed-mode plan artifacts are persisted before execution
- Validation: focused proof smoke tests, then `cargo test -p humanlint`
- Stop conditions: command allowlist bypass required; schema compatibility break; ambiguity about executing uncommitted work from `--changed-from`
- Handoff: changed files, CLI examples, focused test names, residual edge cases

Agent B: rule linkage and receipt assertions

- Phase: 03
- Scope: deterministic `rules_covered` population
- Owned paths: `crates/humanlint/src/commands/proof.rs`, `crates/humanlint/tests/proof_surface_smoke.rs`
- Forbidden paths: `crates/humanlint/src/audit/rules.rs` unless a missing rule ID is discovered; report rendering unless needed for an assertion
- Input contracts: stable rule IDs from `agent/HUMANLINT_STANDARD.md` and `audit/rules.rs`
- Output contracts: only registered HLT IDs appear in `rules_covered`
- Validation: proof receipt schema validation plus focused receipt assertions
- Stop conditions: mapping requires free-form command inference; custom lanes cannot map deterministically
- Handoff: lane-to-rule map and unknown-lane behavior

Agent C: docs and phase receipt

- Phase: 03
- Scope: docs, phase receipt, log closeout
- Owned paths: `docs/testing.md`, `tips/phases/03-proof-router-evidence-ledger.md`, `tips/phases/logs/03-proof-router-evidence-ledger.log`
- Forbidden paths: Rust code, schemas, paper
- Input contracts: final CLI flags and artifact paths from Agent A, rule linkage result from Agent B
- Output contracts: docs match actual commands; phase status does not overclaim
- Validation: `just fast`, plus cite proof artifacts from Agent A
- Stop conditions: implementation not merged; validation not run; phase completion criteria still materially open
- Handoff: exact final phase status and residual risk

Merge order:

1. Agent A CLI bridge
2. Agent B rule linkage
3. Agent C docs/receipt
4. Parent runs full validation and appends final log entry

## Dependencies

Requires Phase 01 report contract stability.

Benefits from Phase 02 rule metadata, but can start with existing `test-map` and `proof-lanes`.

## Public Interface Changes

Add one or both commands:

```bash
humanlint lane --changed-from origin/main
humanlint proof --changed-from origin/main --plan target/humanlint/proof-plan.json
```

If implementing both is too much, implement `humanlint lane` first as a non-running planner.

New artifacts:

- `target/humanlint/proof-plan.json`
- `target/humanlint/proof-receipts/*.json`
- optional `target/humanlint/evidence-index.json`

Do not put volatile local receipts under `agent/`.

## Data Model

Proof plan fields:

- schema version
- standard version
- repo root
- git head
- base ref if known
- changed paths
- matched owner-map entries
- matched test-map entries
- required lanes
- optional lanes
- skipped lanes with reason
- commands
- expected artifacts
- risk notes
- human approval requirements

Proof receipt fields (implemented; schema allows extension):

- lane, command, exit code, elapsed ms
- log path, optional receipt path, artifacts
- changed paths, owner, skipped reason, residual risk
- optional repo root, git head at run time, plan path, run id (content hash)
- optional rules covered (reserved), retryable flag for failures, stdout/stderr byte length

Evidence index fields (implemented subset):

- schema version (`1.2.0` as of companion-artifact slice; prior indexes may read as `1.0.0`)
- generated timestamp
- repo root
- git head at index write time
- proof plan path
- receipt directory
- log directory
- commands run
- receipt paths
- log paths
- failed receipt paths
- skipped lanes and plan risk metadata
- changed paths
- optional `ux_qa_report_path`, `security_evidence_path`, `repo_score_json_path` when the corresponding files exist under the repo at prove time

## Workstreams

### 1. Lane Planner

Implementation tasks:

- Parse `agent/test-map.json` and `agent/proof-lanes.toml`.
- Normalize changed paths from CLI args or git base.
- Match exact path and directory prefixes deterministically.
- If multiple mappings match, choose the most specific path and include inherited broad lanes if policy requires.
- Include owner-map owner where available.
- Produce a proof plan without executing commands.
- Add Markdown rendering for humans.

Acceptance:

- Given changed files under `crates/humanlint/`, planner selects Rust tests.
- Given changed files under `packages/ux-qa/`, planner selects UX QA package tests.
- Given docs-only changes, planner selects audit/docs validation and avoids expensive UX or paper unless mapped.
- Unknown paths produce a finding or hard diagnostic requiring owner/test-map updates.

### 2. Proof Runner

Implementation tasks:

- Add optional execution mode that runs planned commands from a validated proof plan.
- Enforce a command allowlist: each planned command must match `agent/proof-lanes.toml` or `agent/test-map.json` after whitespace normalization, unless `--allow-unsigned-commands` and `HUMANLINT_ALLOW_UNSIGNED_PROOF_COMMANDS=1` are both set.
- Store full command output under ignored `target/humanlint/logs/` when useful.
- Store receipt JSON under ignored `target/humanlint/proof-receipts/`.
- Make command execution fail fast by default, with an option to continue collecting receipts.
- Do not execute destructive commands.

Acceptance:

- Receipts are written for each attempted lane.
- Failed commands include exit code and output path.
- Audit can include receipt summaries without pasting large logs.

### 3. Audit Integration

Implementation tasks:

- Allow audit to read existing proof receipts from a path.
- Populate `Report.proof_receipts` when provided.
- Add findings for missing required proof receipts in release or ratchet modes.
- Keep advisory mode permissive.

Acceptance:

- `humanlint audit` still works without proof receipts.
- Release-mode policy can require receipts.
- Markdown report lists proof receipts compactly.

### 4. Evidence Ledger

Implementation tasks:

- Define an evidence root convention under `target/humanlint/`.
- Normalize artifact paths relative to repo root.
- Add schema for proof plan and receipt artifacts.
- Add `doctor` diagnostics for stale or malformed evidence when present.

Acceptance:

- Evidence can be archived by CI.
- Agents can find the latest proof run without rerunning everything.

## Parallel MCP Breakdown

Partial parallelization only.

Parallel agents:

- Agent A: parse and plan. Owns planner and tests.
- Agent B: receipt schemas and report rendering. Owns schemas and render modules.
- Agent C: docs and examples. Owns docs only.

Do not parallelize command execution and planner data model changes before the proof plan schema is stable.

## Validation

Minimum:

```bash
cargo test -p humanlint
just fast
```

Phase-specific smoke:

```bash
humanlint lane --changed crates/humanlint/src/main.rs
humanlint lane --changed packages/ux-qa/src/rules.ts
```

Proof execution smoke:

```bash
humanlint lane . --changed crates/humanlint/src/main.rs --out target/humanlint/proof-plan.json
humanlint prove . --plan target/humanlint/proof-plan.json
```

Escape hatch (not for default CI): `--allow-unsigned-commands` with `HUMANLINT_ALLOW_UNSIGNED_PROOF_COMMANDS=1`.

## Risks

- Running arbitrary commands is blocked by default: `prove` only runs commands present in `agent/proof-lanes.toml` or `agent/test-map.json`. An explicit flag plus environment variable enables unsigned commands for emergencies.
- Proof routing can create false confidence if mappings are incomplete.
- Receipts can bloat context if stored in prompt-loaded paths.

## Handoff Notes

Leave:

- proof plan schema
- receipt schema
- sample proof plans for Rust, UX, docs, and unknown paths
- command safety policy
- exact validation commands run

## Phase Status Receipt

- Phase status: complete proof router and evidence ledger; `prove --changed` / `--changed-from` exists and evidence index **1.2.0** records standard report artifact links when present at `prove` time
- Files changed (latest slices): `schemas/evidence-index.schema.json`, `schemas/proof-receipt.schema.json`, `crates/humanlint/src/model.rs`, `crates/humanlint/src/main.rs`, `crates/humanlint/src/commands/proof.rs`, `crates/humanlint/tests/proof_surface_smoke.rs`, `crates/humanlint/tests/schema_contracts.rs`, `tips/phases/03-proof-router-evidence-ledger.md`, `tips/phases/logs/03-proof-router-evidence-ledger.log`
- Earlier 2026-05-02 hardening: `crates/humanlint/src/commands/proof.rs`, `crates/humanlint/src/commands/doctor.rs`, `crates/humanlint/src/commands/context_data.rs`, `schemas/proof-plan.schema.json`, `schemas/proof-receipt.schema.json`, evidence index baseline, `docs/moonshot.md`, `docs/testing.md`
- Public interfaces changed: `humanlint prove` accepts `--changed` / `--changed-from`; `ProofEvidenceIndex` optional fields; written evidence index `schema_version` now **1.2.0** for new runs; proof receipt rule coverage accepts rich or simple entries
- Generated artifacts: proof plan, proof receipts, evidence index, and logs under `target/humanlint/` (gitignored)
- Routing maps changed: none required for this slice
- Validation commands: `cargo test -p humanlint`, `just fast`
- Results: see append-only log under `tips/phases/logs/`
- Skipped validation: none
- Exceptions created: unsigned command escape hatch documented for emergencies only
- Follow-up phases: 05 UX proof, 06 security evidence, 11 migration engine, 13 repair optimization
