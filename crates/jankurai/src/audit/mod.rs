pub mod analyzers;
pub mod boundaries_artifact;
pub mod caps;
pub mod evidence;
pub mod finding_builder;
pub mod fix_queue;
pub mod fs;
pub mod helpers;
pub mod policy;
pub mod rules;
pub mod scan;
pub mod security_artifact;
pub mod ux_artifact;

use crate::model::*;
use anyhow::Result;
use caps::{caps_applied, CAPS};
use finding_builder::{dimension_soft_route, FindingBuilder};
use helpers::AuditContext;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
pub struct AuditOptions {
    pub self_audit: bool,
    pub proof_receipts: Option<String>,
}

pub fn run_audit(root: &Path, changed: &[PathBuf]) -> Result<Report> {
    run_audit_with_options(root, changed, AuditOptions::default())
}

pub fn run_audit_with_options(
    root: &Path,
    changed: &[PathBuf],
    options: AuditOptions,
) -> Result<Report> {
    let started = Instant::now();
    let all_files = fs::inventory_repo(root)?;
    let scope_paths: Vec<String> = changed
        .iter()
        .filter_map(|p| normalize_changed_path(root, p))
        .collect();
    let scope_files = if scope_paths.is_empty() {
        all_files.clone()
    } else {
        all_files
            .iter()
            .filter(|f| path_matches_scope(&f.rel_path, &scope_paths))
            .cloned()
            .collect()
    };

    let ctx = AuditContext {
        root: root.to_path_buf(),
        all_files,
        scope_files,
        scope_paths,
        self_audit: options.self_audit,
    };
    let dimensions = analyzers::all_dimensions(&ctx);
    let raw_score = dimensions
        .iter()
        .map(|d| d.weighted_points)
        .sum::<f64>()
        .round() as i32;
    let destructive_sql_hits = scan::destructive_sql_hits(&ctx);
    let caps_applied = caps_applied(&ctx, !destructive_sql_hits.is_empty());
    let final_score = caps_applied
        .iter()
        .filter_map(|c| CAPS.iter().find(|(id, _)| id == c).map(|(_, m)| *m))
        .fold(raw_score, |acc, cap| acc.min(cap));
    let policy = load_policy(root);
    let ux_qa = attach_ux_report_artifact(root, analyzers::ux_qa_status(&ctx));
    let findings = build_findings(
        &ctx,
        &dimensions,
        &caps_applied,
        final_score,
        policy.minimum_score,
        ux_qa.artifact.as_ref(),
        &destructive_sql_hits,
    );
    let agent_fix_queue = fix_queue::build_agent_fix_queue(&findings);
    let decision = report_decision(final_score, &findings, policy.minimum_score);
    let git = git_summary(root, changed);
    let dirty_worktree = git.dirty_worktree.unwrap_or(false);
    let proof_receipts = load_proof_receipts(root, options.proof_receipts.as_deref())?;
    let mut report = Report {
        report_fingerprint: "sha256:pending".into(),
        input_fingerprint: input_fingerprint(&ctx),
        policy_fingerprint: file_fingerprint(&root.join("agent/audit-policy.toml"))
            .unwrap_or_else(|| missing_sha256()),
        manifest_fingerprints: manifest_fingerprints(root),
        dirty_worktree,
        generated_at: started_at(),
        schema_url: "schemas/repo-score.schema.json".into(),
        standard: "jankurai".into(),
        standard_version: STANDARD_VERSION.into(),
        auditor_version: AUDITOR_VERSION.into(),
        schema_version: SCHEMA_VERSION.into(),
        paper_edition: PAPER_EDITION.into(),
        target_stack_id: TARGET_STACK_ID.into(),
        target_stack: TARGET_STACK.into(),
        repo: root.display().to_string(),
        run_id: Some(run_id()),
        started_at: Some(started_at()),
        elapsed_ms: Some(started.elapsed().as_millis()),
        scope: Scope {
            mode: if ctx.scope_paths.is_empty() {
                "full".into()
            } else {
                "changed".into()
            },
            paths: ctx.scope_paths.clone(),
        },
        score: final_score,
        raw_score,
        decision: Some(decision),
        git: Some(git),
        policy: Some(policy),
        proof_receipts,
        caps_applied,
        hard_rules: CAPS
            .iter()
            .map(|(id, max_score)| HardRule {
                id: id.to_string(),
                max_score: *max_score,
            })
            .collect(),
        dimensions,
        ux_qa,
        security_evidence: SecurityEvidenceReadiness {
            artifact: security_artifact::load_report_summary(root),
        },
        boundaries: BoundariesReadiness {
            artifact: boundaries_artifact::load_manifest_summary(root),
        },
        findings,
        agent_fix_queue,
    };
    report.report_fingerprint = report_fingerprint(&report);
    Ok(report)
}

pub fn rebuild_agent_fix_queue(report: &mut Report) {
    report.agent_fix_queue = fix_queue::build_agent_fix_queue(&report.findings);
}

fn attach_ux_report_artifact(root: &Path, mut readiness: UxQaReadiness) -> UxQaReadiness {
    readiness.artifact = ux_artifact::load_report_summary(root);
    readiness
}

fn load_policy(root: &Path) -> PolicySummary {
    use serde::Deserialize;
    #[derive(Debug, Deserialize)]
    struct AuditPolicyFile {
        #[serde(default = "default_minimum_score")]
        minimum_score: i32,
        #[serde(default)]
        fail_on: Vec<String>,
        #[serde(default)]
        advisory_on: Vec<String>,
    }

    fn default_minimum_score() -> i32 {
        85
    }

    let path = root.join("agent/audit-policy.toml");
    let parsed = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| toml::from_str::<AuditPolicyFile>(&text).ok())
        .unwrap_or(AuditPolicyFile {
            minimum_score: default_minimum_score(),
            fail_on: vec!["critical".into(), "high".into()],
            advisory_on: vec!["medium".into(), "low".into()],
        });
    PolicySummary {
        path: path.display().to_string(),
        minimum_score: parsed.minimum_score,
        fail_on: parsed.fail_on,
        advisory_on: parsed.advisory_on,
        mode: Some("standard".into()),
        standard_version: Some(STANDARD_VERSION.into()),
        auditor_version: Some(AUDITOR_VERSION.into()),
        schema_version: Some(SCHEMA_VERSION.into()),
        paper_edition: Some(PAPER_EDITION.into()),
        target_stack: Some(TARGET_STACK_ID.into()),
    }
}

fn report_decision(score: i32, findings: &[Finding], minimum_score: i32) -> ReportDecision {
    let hard_findings = findings
        .iter()
        .filter(|f| f.severity == "high" || f.severity == "critical")
        .count();
    let soft_findings = findings.len().saturating_sub(hard_findings);
    let passed = score >= minimum_score && hard_findings == 0;
    ReportDecision {
        status: if passed { "pass".into() } else { "fail".into() },
        minimum_score,
        passed,
        hard_findings,
        soft_findings,
        ratchet: Some(ReportRatchet {
            baseline_score: score,
            allowed_drop: 0,
            passed,
        }),
    }
}

fn git_summary(root: &Path, changed: &[PathBuf]) -> GitSummary {
    GitSummary {
        head: git_output(root, &["rev-parse", "--short", "HEAD"]),
        base: None,
        changed_files: changed.len(),
        mode: if changed.is_empty() {
            "full".into()
        } else {
            "changed".into()
        },
        dirty_worktree: Some(
            Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(root)
                .output()
                .ok()
                .map(|out| !out.stdout.is_empty())
                .unwrap_or(false),
        ),
    }
}

fn git_output(root: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn run_id() -> String {
    started_at().replace(':', "-")
}

fn started_at() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

fn file_fingerprint(path: &Path) -> Option<String> {
    std::fs::read(path)
        .ok()
        .map(|bytes| format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn missing_sha256() -> String {
    "sha256:0000000000000000000000000000000000000000000000000000000000000000".into()
}

fn manifest_fingerprints(root: &Path) -> ManifestFingerprints {
    ManifestFingerprints {
        owner_map: file_fingerprint(&root.join("agent/owner-map.json")),
        test_map: file_fingerprint(&root.join("agent/test-map.json")),
        generated_zones: file_fingerprint(&root.join("agent/generated-zones.toml")),
        boundaries: file_fingerprint(&root.join("agent/boundaries.toml")),
        proof_lanes: file_fingerprint(&root.join("agent/proof-lanes.toml")),
        standard_version: file_fingerprint(&root.join("agent/standard-version.toml")),
    }
}

fn input_fingerprint(ctx: &AuditContext) -> String {
    let mut hasher = Sha256::new();
    for file in &ctx.all_files {
        hasher.update(file.rel_path.as_bytes());
        hasher.update([0]);
        hasher.update(file.text.as_bytes());
        hasher.update([0xff]);
    }
    format!("sha256:{:x}", hasher.finalize())
}

pub fn report_fingerprint(report: &Report) -> String {
    let mut value = serde_json::to_value(report).unwrap_or_default();
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "report_fingerprint".into(),
            serde_json::Value::String("sha256:pending".into()),
        );
    }
    let bytes = serde_json::to_vec(&value).unwrap_or_default();
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn build_findings(
    ctx: &AuditContext,
    dimensions: &[DimensionResult],
    caps_applied: &[String],
    final_score: i32,
    minimum_score: i32,
    ux_artifact: Option<&UxQaReportArtifactSummary>,
    destructive_sql_hits: &[scan::FindingHit],
) -> Vec<Finding> {
    let dim_by_name: HashMap<_, _> = dimensions.iter().map(|d| (d.name.as_str(), d)).collect();
    let mut b = FindingBuilder::new(ctx);

    if caps_applied.contains(&"no-root-agent-instructions".into()) {
        b.add(
            "medium",
            "context",
            "AGENTS.md",
            "no root agent/developer instruction file routes contributors at the repository root",
            "add a concise root `AGENTS.md` and move deeper ownership rules into local docs",
            vec!["no root `AGENTS.md` detected".into()],
            None,
            None,
        );
    }
    if caps_applied.contains(&"no-one-command-setup-or-validation".into()) {
        b.add(
            "high",
            "proof",
            ".",
            "no one-command setup or validation lane was detected",
            "add a canonical `setup`, `check`, `test`, or `verify` lane in one root command file",
            vec!["no root setup/check/test/verify target surfaced".into()],
            None,
            None,
        );
    }
    if caps_applied.contains(&"no-deterministic-fast-lane".into()) {
        b.add("high", "proof", ".", "no deterministic fast lane was detected", "add a fast lane that runs the narrowest deterministic proof loop and keep it canonical", vec!["no fast lane markers found".into()], Some("HLT-004-UNMAPPED-PROOF"), None);
    }
    if caps_applied.contains(&"no-security-lane-on-high-risk-repo".into()) {
        b.add("high", "security", ".github/workflows", "high-risk repo has no explicit security lane", "add a dedicated security lane with secret scanning, dependency review, and workflow linting", vec!["no security lane markers found".into()], Some("HLT-009-GENERATED-SECURITY"), None);
    }
    if caps_applied.contains(&"generated-contracts-or-public-api-drift-untested".into()) {
        b.add(
            "high",
            "boundary",
            "contracts/",
            "generated contracts or public API drift are not being checked",
            "generate boundary clients and gate drift with public-API or semver checks",
            vec!["contract surface exists".into()],
            Some("HLT-007-HANDWRITTEN-CONTRACT"),
            None,
        );
    }
    if caps_applied.contains(&"python-direct-product-truth-or-db-ownership".into()) {
        b.add(
            "high",
            "python",
            "python/",
            "Python appears outside the bounded AI/data service or owns product truth",
            "move Python into `python/ai-service` or keep it tooling-only under `tools/`",
            vec!["Python should stay away from product truth and production DB ownership".into()],
            Some("HLT-005-PYTHON-PRODUCT-TRUTH"),
            None,
        );
    }
    if caps_applied.contains(&"no-secret-or-dependency-scanning-in-ci".into()) {
        b.add(
            "high",
            "security",
            ".github/workflows",
            "no secret or dependency scanning was found in CI",
            "add secret scanning, dependency review, and SBOM or provenance checks to CI",
            vec!["no CI scan markers found".into()],
            Some("HLT-010-SECRET-SPRAWL"),
            None,
        );
    }
    if caps_applied.contains(&"no-jankurai-audit-lane-in-ci".into()) {
        b.add("high", "audit", ".github/workflows", "CI does not run the jankurai audit lane", "add a CI job that runs `jankurai . --json agent/repo-score.json --md agent/repo-score.md` and uploads both artifacts", vec!["audit output must stay JSON plus Markdown for agent repair routing".into()], None, None);
    }

    for hit in scan::manifest_parse_findings(ctx) {
        b.add(
            "high",
            "audit",
            &hit.path,
            "jankurai manifest could not be parsed",
            "fix the manifest syntax so audit policy and routing maps are authoritative",
            vec![hit.problem],
            Some("HLT-017-OPAQUE-OBSERVABILITY"),
            hit.line,
        );
    }
    for path in crate::audit::helpers::missing_owner_paths(ctx)
        .into_iter()
        .take(10)
    {
        b.add(
            "high",
            "context",
            "agent/owner-map.json",
            &format!("path `{path}` has no owner-map route"),
            "add the narrowest stable prefix for this path to `agent/owner-map.json`",
            vec![path],
            Some("HLT-003-OWNERLESS-PATH"),
            None,
        );
    }
    for path in crate::audit::helpers::missing_test_paths(ctx)
        .into_iter()
        .take(10)
    {
        b.add(
            "high",
            "proof",
            "agent/test-map.json",
            &format!("path `{path}` has no test-map proof route"),
            "add the narrowest stable prefix and runnable proof command to `agent/test-map.json`",
            vec![path],
            Some("HLT-004-UNMAPPED-PROOF"),
            None,
        );
    }

    if !helpers::non_optimal_language_hits(ctx).is_empty() {
        let hit = helpers::non_optimal_language_hits(ctx)[0].clone();
        b.add("high", "stack", &hit.rel_path, "runtime code uses a language outside the chosen optimal stack", "move product runtime behavior to Rust core, TypeScript web, SQL migrations, generated contracts, or bounded `python/ai-service` only", vec![format!("{} uses `{}`", hit.rel_path, hit.suffix), TARGET_STACK.into()], None, None);
    }
    let ratio = helpers::python_ratio(ctx);
    if ratio > 0.15 {
        b.add(if ratio > 0.30 { "high" } else { "medium" }, "python", "python/ai-service", "Python is too large a share of runtime product code for this standard", "keep Python bounded to model/data work and move durable product truth, authz, workflows, and core behavior into Rust", vec!["Python share is above the soft cap".into()], None, None);
    }
    if !scan::todo_hits(ctx).is_empty() {
        let hit = scan::todo_hits(ctx)[0].clone();
        b.add("high", "vibe", &hit.path, "product code contains TODO/stub/unimplemented/unreachable placeholder markers", "replace placeholders with implemented behavior, typed unsupported-state errors, or a tracked exception record with docs", vec![format!("{}:{} {}", hit.path, hit.line.unwrap_or(1), hit.text)], Some("HLT-001-DEAD-MARKER"), Some(hit.line.unwrap_or(1)));
    }
    if scan::fallback_hits(ctx).len() > 1 {
        let hit = scan::fallback_hits(ctx)[0].clone();
        b.add("high", "vibe", &hit.path, "fallback soup detected in product code", "collapse fallback chains into explicit typed states with bounded retry policy, telemetry, and documented repair guidance", vec![format!("{}:{} {}", hit.path, hit.line.unwrap_or(1), hit.text)], Some("HLT-001-DEAD-MARKER"), Some(hit.line.unwrap_or(1)));
    }
    for hit in scan::future_hostile_hits(ctx) {
        b.add(
            "high",
            "vibe",
            &hit.path,
            &format!(
                "future-hostile/dead-language term `{}` appears in product/runtime code",
                hit.matched_term.clone().unwrap_or_default()
            ),
            &hit.agent_fix,
            vec![
                format!("{}:{}", hit.path, hit.line.unwrap_or(1)),
                hit.problem.clone(),
            ],
            Some("HLT-001-DEAD-MARKER"),
            hit.line,
        );
    }
    if !scan::duplicate_blocks(ctx).is_empty() {
        let hit = scan::duplicate_blocks(ctx)[0].clone();
        b.add("high", "vibe", &hit.path, "duplicated product code block detected", "extract the duplicated behavior behind one named boundary and add focused tests before changing behavior", vec![hit.problem.clone()], None, hit.line);
    }
    if !scan::generated_zone_issues(ctx).is_empty() {
        let hit = scan::generated_zone_issues(ctx)[0].clone();
        b.add("high", "generated", &hit.path, "generated zone is not protected strongly enough against hand edits", "add `agent/generated-zones.toml`, require generated/do-not-edit markers, and route repairs to the source contract", vec![hit.problem.clone()], Some("HLT-002-GENERATED-MUTATION"), hit.line);
    }
    for hit in scan::generated_zone_manifest_metadata_issues(ctx) {
        b.add(
            "high",
            "generated",
            &hit.path,
            "generated zone manifest lacks reproducibility metadata",
            "declare non-empty `path`, `source`, and `command` for every `[[zone]]` in `agent/generated-zones.toml`",
            vec![hit.problem.clone()],
            Some("HLT-002-GENERATED-MUTATION"),
            hit.line,
        );
    }
    if !scan::wrong_layer_db_hits(ctx).is_empty() {
        let hit = scan::wrong_layer_db_hits(ctx)[0].clone();
        b.add("high", "data", &hit.path, "direct database access appears in a wrong layer", "move SQL and DB clients to `crates/adapters` or `db/`; expose typed application/domain APIs upward", vec![hit.problem.clone()], Some("HLT-006-DIRECT-DB-WRONG-LAYER"), hit.line);
    }

    // AST / Graph Pilot findings
    if !crate::audit::analyzers::ast::run_ast_pilot(ctx).is_empty() {
        let hit = crate::audit::analyzers::ast::run_ast_pilot(ctx)[0].clone();
        b.add(
            "high",
            "boundary",
            &hit.path,
            &hit.problem,
            &hit.agent_fix,
            vec![hit.text.clone()],
            Some("HLT-006-DIRECT-DB-WRONG-LAYER"), // Or maybe a more general boundary rule ID
            hit.line,
        );
    }
    if caps_applied.contains(&"missing-web-e2e-lane".into()) {
        b.add("high", "test", "apps/web", "web surface lacks a Playwright/Cypress e2e lane", "add Playwright e2e tests for critical user flows and wire them into the fast or CI proof map", vec!["web surface detected".into()], Some("HLT-013-RENDERED-UX-GAP"), None);
    }
    if caps_applied.contains(&"missing-rendered-ux-qa-lane".into()) {
        b.add("high", "ux-qa", "apps/web", "web surface lacks layered rendered UX QA evidence", "add Storybook state coverage, Playwright screenshots, visual review or `@jankurai/ux-qa`, accessibility scans, CLS checks, generated mocks, and design tokens", vec!["rendered UX QA lane missing".into()], Some("HLT-013-RENDERED-UX-GAP"), None);
    }
    if let Some(art) = ux_artifact {
        let missing_non_a11y_artifacts = art
            .missing_artifact_kinds
            .iter()
            .filter(|kind| kind.as_str() != "accessibility")
            .cloned()
            .collect::<Vec<_>>();
        if art.reports_missing_required_states > 0 || !missing_non_a11y_artifacts.is_empty() {
            let mut evidence = vec![format!(
                "{} validated reports; {} report(s) missing required states",
                art.report_count, art.reports_missing_required_states
            )];
            if !art.missing_state_names.is_empty() {
                evidence.push(format!(
                    "missing states: {}",
                    art.missing_state_names.join(", ")
                ));
            }
            if !missing_non_a11y_artifacts.is_empty() {
                evidence.push(format!(
                    "missing artifacts: {}",
                    missing_non_a11y_artifacts.join(", ")
                ));
            }
            b.add(
                "high",
                "ux-qa",
                &art.path,
                "validated UX QA evidence is missing required state or artifact coverage",
                "complete the configured route state matrix and emit required screenshot or ARIA artifacts before treating UX proof as complete",
                evidence,
                Some("HLT-013-RENDERED-UX-GAP"),
                None,
            );
        }
        if art.visual_baseline_review > 0 || art.visual_baseline_block > 0 {
            b.add(
                "high",
                "ux-qa",
                &art.path,
                "validated UX QA evidence has visual baseline gaps",
                "review or block the changed visual baseline evidence and regenerate the baseline artifacts before treating the surface as proven",
                vec![
                    format!(
                        "visual baseline missing/changed: {}/{}",
                        art.visual_baseline_missing, art.visual_baseline_changed
                    ),
                    format!(
                        "visual baseline review/block: {}/{}",
                        art.visual_baseline_review, art.visual_baseline_block
                    ),
                    format!(
                        "artifact fingerprints: {}",
                        art.artifact_fingerprint_count
                    ),
                ],
                Some("HLT-013-RENDERED-UX-GAP"),
                None,
            );
        }
        if art.accessibility_violation_total > 0
            || art.reports_missing_required_accessibility_artifact > 0
        {
            b.add(
                "high",
                "ux-qa",
                &art.path,
                "validated UX QA evidence has accessibility gaps",
                "fix axe accessibility violations and emit the required accessibility artifact for every configured report",
                vec![
                    format!("accessibility violations: {}", art.accessibility_violation_total),
                    format!("accessibility incomplete: {}", art.accessibility_incomplete_total),
                    format!(
                        "reports missing accessibility artifact: {}",
                        art.reports_missing_required_accessibility_artifact
                    ),
                ],
                Some("HLT-014-A11Y-GAP"),
                None,
            );
        }
    }
    if !scan::prompt_injection_hits(ctx).is_empty() {
        let hit = scan::prompt_injection_hits(ctx)[0].clone();
        b.add("high","security",&hit.path,"trusted agent/tool policy contains prompt-injection or policy-bypass language","isolate untrusted instructions from trusted policy, remove bypass wording, and validate tool calls against the repository standard", vec![hit.problem], Some("HLT-011-PROMPT-INJECTION"), hit.line);
    }
    if !scan::agency_hits(ctx).is_empty() {
        let hit = scan::agency_hits(ctx)[0].clone();
        b.add("high","security",&hit.path,"agent/tool permissions appear broader than the requested proof lane","replace broad terminal/browser/network/filesystem permissions with least-privilege lane profiles and explicit approval gates", vec![hit.problem], Some("HLT-012-OVERBROAD-AGENCY"), hit.line);
    }
    if !scan::secret_hits(ctx).is_empty() {
        let hit = scan::secret_hits(ctx)[0].clone();
        b.add("critical","security",&hit.path,"secret-like value or credential material appears in repository text","remove and rotate the credential, add local and CI secret scanning, and scan transcripts/artifacts/MCP config for related exposure", vec![hit.problem], Some("HLT-010-SECRET-SPRAWL"), hit.line);
    }
    if !scan::false_green_hits(ctx).is_empty() {
        let hit = scan::false_green_hits(ctx)[0].clone();
        b.add("high","test",&hit.path,"test code contains disabled, focused, tautological, or snapshot-only proof","replace false-green tests with behavior assertions, red/green evidence, and mutation or fault checks for changed behavior", vec![hit.problem], Some("HLT-008-FALSE-GREEN-RISK"), hit.line);
    }
    if !scan::ci_hardening_hits(ctx).is_empty() {
        let hit = scan::ci_hardening_hits(ctx)[0].clone();
        b.add(
            "high",
            "security",
            &hit.path,
            &hit.problem,
            &hit.agent_fix,
            vec![hit.text],
            Some("HLT-020-CI-HARDENING-GAP"),
            hit.line,
        );
    }
    if !destructive_sql_hits.is_empty() {
        let hit = destructive_sql_hits[0].clone();
        let fix = hit.agent_fix.as_str();
        b.add(
            "high",
            "data",
            &hit.path,
            "destructive migration lacks documented safety evidence",
            fix,
            vec![hit.problem],
            Some("HLT-021-DESTRUCTIVE-MIGRATION"),
            hit.line,
        );
    }
    if caps_applied.contains(&"missing-rust-property-or-integration-tests".into()) {
        b.add("high","test","crates/","Rust surface lacks required property and/or integration tests","add `proptest` or equivalent invariant tests plus `tests/` integration coverage routed through `cargo nextest` or `cargo test`", vec!["Rust surface detected".into()], Some("HLT-008-FALSE-GREEN-RISK"), None);
    }
    if caps_applied.contains(&"no-agent-friendly-exception-pattern".into()) {
        let exception = helpers::audit_repair_exception();
        b.add(
            "high",
            "exceptions",
            "crates/domain",
            "no agent-friendly exception/error pattern was detected",
            exception.repair_hint,
            vec![
                exception.purpose.into(),
                exception.reason.into(),
                exception.common_fixes.join("; "),
                exception.docs_url.into(),
            ],
            Some("HLT-017-OPAQUE-OBSERVABILITY"),
            None,
        );
    }
    if caps_applied.contains(&"missing-agent-readable-docs".into()) {
        let missing = helpers::missing_core_docs(ctx);
        b.add("medium","docs","docs/","agent-readable documentation is incomplete","add concise docs for architecture, boundaries, tests, generated zones, and audit rules; route them from root `AGENTS.md`", missing, None, None);
    }
    if !scan::streaming_runtime_hits(ctx).is_empty() {
        let hit = scan::streaming_runtime_hits(ctx)[0].clone();
        b.add(
            "high",
            "boundary",
            &hit.path,
            "queue or streaming runtime client appears outside the declared adapter boundary",
            "move Kafka/Tansu/Iggy/Fluvio/NATS/Redis-stream clients behind `crates/adapters/queues` or document a brownfield exception with owner, expiry, and migration path",
            vec![hit.problem],
            Some("HLT-019-STREAMING-RUNTIME-DRIFT"),
            hit.line,
        );
    }
    // Phase 07 H1: contract source detection
    for hit in scan::contract_source_hits(ctx) {
        b.add(
            "high",
            "boundary",
            &hit.path,
            &hit.problem,
            &hit.agent_fix,
            vec![hit.text],
            Some("HLT-007-HANDWRITTEN-CONTRACT"),
            hit.line,
        );
    }
    // Phase 07 H2: generated zone existence + header
    for hit in scan::generated_zone_existence_hits(ctx) {
        b.add(
            "high",
            "generated",
            &hit.path,
            &hit.problem,
            &hit.agent_fix,
            vec![hit.text],
            Some("HLT-002-GENERATED-MUTATION"),
            hit.line,
        );
    }
    // Phase 07 H4: event contract path validation
    for hit in scan::event_contract_path_hits(ctx) {
        b.add(
            "high",
            "boundary",
            &hit.path,
            &hit.problem,
            &hit.agent_fix,
            vec![hit.text],
            Some("HLT-007-HANDWRITTEN-CONTRACT"),
            hit.line,
        );
    }

    for dimension in dimensions.iter().filter(|dimension| dimension.score < 85) {
        let (category, path, rule_id, fix) = dimension_soft_route(&dimension.name);
        let evidence = if dimension.evidence.is_empty() && dimension.notes.is_empty() {
            vec![format!("{} scored {}", dimension.name, dimension.score)]
        } else {
            dimension
                .evidence
                .iter()
                .chain(dimension.notes.iter())
                .take(4)
                .cloned()
                .collect()
        };
        b.add(
            "medium",
            category,
            path,
            &format!(
                "`{}` scored {} below the standard floor of 85",
                dimension.name, dimension.score
            ),
            fix,
            evidence,
            Some(rule_id),
            None,
        );
    }

    if final_score < minimum_score && !b.has_any_finding() {
        b.add(
            "medium",
            "audit",
            "agent/audit-policy.toml",
            "repository score is below the configured floor but no specific repair finding was emitted",
            "add or tune audit rules so every below-floor score maps to at least one actionable repair queue entry",
            vec![format!("score {final_score} is below minimum_score {minimum_score}")],
            Some("HLT-017-OPAQUE-OBSERVABILITY"),
            None,
        );
    }

    if let Some(ownership) = dim_by_name.get("Ownership and navigation surface") {
        if ownership.score < 55 && !b.has_context_finding() {
            b.add("medium","context",".","navigation surface is thin for agent work","add local routing docs and machine-readable owner/test maps where the repo needs them", ownership.evidence.iter().take(2).cloned().collect(), None, None);
        }
    }
    if let Some(shape) = dim_by_name.get("Code shape and semantic surface") {
        if shape.notes.iter().any(|n| n.contains("large code files")) {
            if let Some(max) = helpers::max_loc(&helpers::product_code_files(ctx)) {
                b.add(if max <= 1000 { "medium" } else { "high" }, "shape", ".", &format!("largest code file is {} LOC", max), "split the file along ownership or semantic boundaries before agents have to patch it again", vec![format!("largest authored code file: {} LOC", max)], None, None);
            }
        }
    }

    let mut findings = b.into_findings();
    findings.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.line.unwrap_or(0).cmp(&b.line.unwrap_or(0)))
            .then(a.problem.cmp(&b.problem))
    });
    findings
}

fn path_matches_scope(rel_path: &str, scopes: &[String]) -> bool {
    if scopes.is_empty() {
        return true;
    }
    scopes.iter().any(|scope| {
        rel_path == scope
            || rel_path.starts_with(&format!("{}/", scope))
            || scope.starts_with(&format!("{}/", rel_path))
    })
}

fn normalize_changed_path(root: &Path, path: &Path) -> Option<String> {
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let rel = candidate
        .strip_prefix(root)
        .ok()?
        .to_string_lossy()
        .replace('\\', "/");
    Some(rel)
}

pub fn changed_paths_from_git(root: &Path, base: &str) -> Result<Vec<PathBuf>> {
    let refspec = format!("{base}...HEAD");
    let output = Command::new("git")
        .args(["diff", "--name-only", refspec.as_str()])
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Ok(vec![]);
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| root.join(line.trim()))
        .collect())
}

fn load_proof_receipts(root: &Path, path: Option<&str>) -> Result<Vec<ProofReceipt>> {
    let Some(path) = path else {
        return Ok(vec![]);
    };
    let path = root.join(path);
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut entries = Vec::new();
    if path.is_dir() {
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            entries.push(entry_path);
        }
        entries.sort();
    } else {
        entries.push(path);
    }

    let mut receipts = Vec::new();
    for entry in entries {
        let text = std::fs::read_to_string(&entry)?;
        let value: serde_json::Value = serde_json::from_str(&text)?;
        crate::validation::validate_value(
            root,
            crate::validation::ArtifactSchema::ProofReceipt,
            &value,
        )?;
        let receipt: ProofReceipt = serde_json::from_value(value)?;
        receipts.push(receipt);
    }
    Ok(receipts)
}

pub fn docs_for_rule_id(rule: &str) -> Option<&'static str> {
    rules::docs_for_rule_id(rule)
}

pub fn rule_registry() -> &'static [rules::RuleSpec] {
    rules::all()
}
