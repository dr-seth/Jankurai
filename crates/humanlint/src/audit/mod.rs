mod fs;

use crate::model::*;
use aho_corasick::AhoCorasick;
use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const WEIGHTS: &[(&str, u32)] = &[
    ("Ownership and navigation surface", 14),
    ("Contract and boundary integrity", 14),
    ("Proof lanes and test routing", 14),
    ("Security and supply-chain posture", 14),
    ("Code shape and semantic surface", 12),
    ("Data truth and workflow safety", 8),
    ("Observability and repair evidence", 8),
    ("Context economy and agent instructions", 8),
    ("Python containment and polyglot hygiene", 4),
    ("Build speed signals", 4),
];

const CAPS: &[(&str, i32)] = &[
    ("no-root-agent-instructions", 75),
    ("no-one-command-setup-or-validation", 70),
    ("no-deterministic-fast-lane", 65),
    ("no-security-lane-on-high-risk-repo", 60),
    ("generated-contracts-or-public-api-drift-untested", 80),
    ("python-direct-product-truth-or-db-ownership", 72),
    ("no-secret-or-dependency-scanning-in-ci", 78),
    ("no-humanlint-audit-lane-in-ci", 82),
    ("non-optimal-product-language-found", 74),
    ("too-much-python-in-product-surface", 72),
    ("vibe-placeholders-in-product-code", 68),
    ("fallback-soup-in-product-code", 70),
    ("future-hostile-dead-language-in-product-code", 64),
    ("severe-duplication-in-product-code", 70),
    ("generated-zone-mutation-risk", 76),
    ("direct-db-access-from-wrong-layer", 66),
    ("missing-web-e2e-lane", 82),
    ("missing-rendered-ux-qa-lane", 84),
    ("prompt-injection-risk", 78),
    ("overbroad-agent-agency", 65),
    ("secret-like-content-detected", 60),
    ("false-green-test-risk", 76),
    ("destructive-migration-risk", 70),
    ("missing-rust-property-or-integration-tests", 82),
    ("no-agent-friendly-exception-pattern", 76),
    ("missing-agent-readable-docs", 80),
];

const ALLOWED_PYTHON_ROOTS: &[&str] = &[
    ".github",
    "benchmarks",
    "docs",
    "examples",
    "paper",
    "python/ai-service",
    "reference",
    "scripts",
    "tests",
    "tools",
];

const FUTURE_HOSTILE_TERMS: &[&str] = &[
    "cleanup later",
    "remove later",
    "best effort",
    "dead code",
    "deprecated",
    "depricated",
    "temporary",
    "workaround",
    "backcompat",
    "placeholder",
    "fallback",
    "obsolete",
    "legacy",
    "unused",
    "stale",
    "fixme",
    "dummy",
    "compat",
    "shim",
    "stub",
    "hack",
    "todo",
    "temp",
    "old",
];

const FUTURE_HOSTILE_ALLOWLIST_PREFIXES: &[&str] = &["docs/", "reference/", "vendor/"];
const FUTURE_HOSTILE_PRODUCT_COPY_PARTS: &[&str] = &[
    "copy-deck",
    "copydeck",
    "i18n",
    "l10n",
    "locale",
    "locales",
    "marketing-copy",
    "messages",
    "product-copy",
    "productcopy",
    "translations",
];

const TODO_PATTERNS: &[&str] = &[
    "TODO",
    "FIXME",
    "HACK",
    "XXX",
    "stub",
    "placeholder",
    "not implemented",
    "todo!(",
    "unimplemented!(",
    "panic!(\"todo",
    "panic!(\"not implemented",
];
const FALLBACK_PATTERNS: &[&str] = &[
    "fallback",
    "best effort",
    "try again",
    "retry",
    "except Exception",
    "except:",
    "unwrap_or_default(",
    "or_else(",
    "return null",
    "return undefined",
];
const PROMPT_PATTERNS: &[&str] = &[
    "ignore previous instructions",
    "ignore prior instructions",
    "reveal the secret",
    "reveal the token",
    "bypass policy",
];
const AGENCY_PATTERNS: &[&str] = &[
    "danger-full-access",
    "approval_policy: never",
    "sandbox_mode: danger-full-access",
    "allow all tools",
    "unrestricted terminal",
    "unrestricted browser",
    "unrestricted filesystem",
];
const FALSE_GREEN_PATTERNS: &[&str] = &[
    ".skip(",
    ".only(",
    "xtest(",
    "xit(",
    "expect(true).toBe(true)",
    "assert true",
    "toMatchSnapshot(",
    "toMatchInlineSnapshot(",
];
const DESTRUCTIVE_SQL_PATTERNS: &[&str] = &[
    "drop table",
    "drop column",
    "drop database",
    "drop schema",
    "truncate table",
    "alter table",
    "delete from",
];

const OWNER_MAP_PREFIXES: &[(&str, &str)] = &[
    ("AGENTS.md", "agent"),
    ("README.md", "workspace"),
    ("Justfile", "workspace"),
    ("package.json", "workspace"),
    ("package-lock.json", "workspace"),
    (".github/", "ops"),
    ("agent/", "agent"),
    ("assets/", "paper"),
    ("docs/", "standard"),
    ("crates/", "tools"),
    ("packages/ux-qa/", "tools"),
    ("paper/", "paper"),
    ("reference/", "read-only"),
    ("tips/", "paper"),
    ("tools/", "tools"),
];

pub fn run_audit(root: &Path, changed: &[PathBuf]) -> Result<Report> {
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
        all_files,
        scope_files,
        scope_paths,
    };
    let dimensions = dimensions(&ctx);
    let raw_score = dimensions
        .iter()
        .map(|d| d.weighted_points)
        .sum::<f64>()
        .round() as i32;
    let caps_applied = caps_applied(&ctx);
    let final_score = caps_applied
        .iter()
        .filter_map(|c| CAPS.iter().find(|(id, _)| id == c).map(|(_, m)| *m))
        .fold(raw_score, |acc, cap| acc.min(cap));
    let findings = build_findings(&ctx, &dimensions, &caps_applied);
    let agent_fix_queue = build_agent_fix_queue(&findings);
    Ok(Report {
        standard: "humanlint".into(),
        standard_version: STANDARD_VERSION.into(),
        auditor_version: AUDITOR_VERSION.into(),
        schema_version: SCHEMA_VERSION.into(),
        paper_edition: PAPER_EDITION.into(),
        target_stack_id: TARGET_STACK_ID.into(),
        target_stack: TARGET_STACK.into(),
        repo: root.display().to_string(),
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
        caps_applied,
        hard_rules: CAPS
            .iter()
            .map(|(id, max_score)| HardRule {
                id: id.to_string(),
                max_score: *max_score,
            })
            .collect(),
        dimensions,
        ux_qa: ux_qa_status(&ctx),
        findings,
        agent_fix_queue,
    })
}

#[derive(Clone)]
struct AuditContext {
    all_files: Vec<FileInfo>,
    scope_files: Vec<FileInfo>,
    scope_paths: Vec<String>,
}

fn dimensions(ctx: &AuditContext) -> Vec<DimensionResult> {
    vec![
        dimension_ownership(ctx),
        dimension_contracts(ctx),
        dimension_proof(ctx),
        dimension_security(ctx),
        dimension_shape(ctx),
        dimension_data(ctx),
        dimension_observability(ctx),
        dimension_context(ctx),
        dimension_python(ctx),
        dimension_speed(ctx),
    ]
}

fn caps_applied(ctx: &AuditContext) -> Vec<String> {
    let mut caps = Vec::new();
    if !has_root_agents(ctx) {
        caps.push("no-root-agent-instructions".into());
    }
    if !has_one_command(ctx) {
        caps.push("no-one-command-setup-or-validation".into());
    }
    if !has_fast_lane(ctx) {
        caps.push("no-deterministic-fast-lane".into());
    }
    if is_high_risk_repo(ctx) && !has_security_lane(ctx) {
        caps.push("no-security-lane-on-high-risk-repo".into());
    }
    if (has_contract_surface(ctx) || has_polyglot_boundary(ctx))
        && !(has_generated_contracts(ctx) || has_api_drift_checks(ctx))
    {
        caps.push("generated-contracts-or-public-api-drift-untested".into());
    }
    if bad_python_paths(ctx) {
        caps.push("python-direct-product-truth-or-db-ownership".into());
    }
    if is_high_risk_repo(ctx) && !has_secret_or_dependency_scans(ctx) {
        caps.push("no-secret-or-dependency-scanning-in-ci".into());
    }
    if is_high_risk_repo(ctx) && !has_humanlint_audit_ci_lane(ctx) {
        caps.push("no-humanlint-audit-lane-in-ci".into());
    }
    if !non_optimal_language_hits(ctx).is_empty() {
        caps.push("non-optimal-product-language-found".into());
    }
    if python_ratio(ctx) > 0.15 {
        caps.push("too-much-python-in-product-surface".into());
    }
    if !todo_hits(ctx).is_empty() {
        caps.push("vibe-placeholders-in-product-code".into());
    }
    if fallback_hits(ctx).len() > 1 {
        caps.push("fallback-soup-in-product-code".into());
    }
    if !future_hostile_hits(ctx).is_empty() {
        caps.push("future-hostile-dead-language-in-product-code".into());
    }
    if !duplicate_blocks(ctx).is_empty() {
        caps.push("severe-duplication-in-product-code".into());
    }
    if !generated_zone_issues(ctx).is_empty() {
        caps.push("generated-zone-mutation-risk".into());
    }
    if !wrong_layer_db_hits(ctx).is_empty() {
        caps.push("direct-db-access-from-wrong-layer".into());
    }
    if has_web_surface(ctx) && !has_playwright_e2e(ctx) {
        caps.push("missing-web-e2e-lane".into());
    }
    if has_web_surface(ctx) && !ux_qa_status(ctx).has_rendered_ux_lane {
        caps.push("missing-rendered-ux-qa-lane".into());
    }
    if !prompt_injection_hits(ctx).is_empty() {
        caps.push("prompt-injection-risk".into());
    }
    if !agency_hits(ctx).is_empty() {
        caps.push("overbroad-agent-agency".into());
    }
    if !secret_hits(ctx).is_empty() {
        caps.push("secret-like-content-detected".into());
    }
    if !false_green_hits(ctx).is_empty() {
        caps.push("false-green-test-risk".into());
    }
    if !destructive_sql_hits(ctx).is_empty() {
        caps.push("destructive-migration-risk".into());
    }
    if has_rust_surface(ctx) && (!has_rust_property_tests(ctx) || !has_rust_integration_tests(ctx))
    {
        caps.push("missing-rust-property-or-integration-tests".into());
    }
    if !has_agent_friendly_exceptions(ctx) && !product_code_files(ctx).is_empty() {
        caps.push("no-agent-friendly-exception-pattern".into());
    }
    if !missing_core_docs(ctx).is_empty() {
        caps.push("missing-agent-readable-docs".into());
    }
    caps
}

fn weight_for(name: &str) -> u32 {
    WEIGHTS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, w)| *w)
        .unwrap_or(0)
}
fn make_dim(name: &str, score: i32, evidence: Vec<String>, notes: Vec<String>) -> DimensionResult {
    let w = weight_for(name);
    DimensionResult {
        name: name.into(),
        weight: w,
        score: score.clamp(0, 100),
        weighted_points: (w as f64) * (score.clamp(0, 100) as f64) / 100.0,
        evidence,
        notes,
    }
}

fn dimension_ownership(ctx: &AuditContext) -> DimensionResult {
    let mut score = 35;
    let mut evidence = vec![];
    let notes = vec![];
    if has_root_agents(ctx) {
        score += 18;
        evidence.push("root `AGENTS.md` present".into());
    }
    if ctx.all_files.iter().any(|f| f.name == "CODEOWNERS") {
        score += 10;
        evidence.push("`CODEOWNERS` present".into());
    }
    if ctx
        .all_files
        .iter()
        .any(|f| f.rel_path == "agent/owner-map.json")
    {
        score += 10;
        evidence.push("owner map present".into());
    }
    if ctx
        .all_files
        .iter()
        .any(|f| f.rel_path == "agent/test-map.json" || f.rel_path == "agent/proof-lanes.toml")
    {
        score += 8;
        evidence.push("test/proof routing map present".into());
    }
    if ctx
        .all_files
        .iter()
        .any(|f| f.name == "AGENTS.md" && f.rel_path != "AGENTS.md")
    {
        score += 4;
        evidence.push("local `AGENTS.md` file(s)".into());
    }
    if root_readme_routes(ctx) {
        score += 6;
        evidence.push("root `README.md` routes to workspace layout".into());
    }
    if let Some(max) = max_loc(&product_code_files(ctx)) {
        if max > 500 {
            score -= 8;
            evidence.push("authored code file exceeds 500 LOC".into());
        }
        if max > 1000 {
            score -= 8;
            evidence.push("authored code file exceeds 1000 LOC".into());
        }
    }
    make_dim("Ownership and navigation surface", score, evidence, notes)
}

fn dimension_contracts(ctx: &AuditContext) -> DimensionResult {
    let mut score = 35;
    let mut evidence = vec![];
    let notes = vec![];
    if has_contract_surface(ctx) {
        score += 15;
        evidence.push("contract surface found".into());
    }
    if has_generated_contracts(ctx) {
        score += 15;
        evidence.push("generated contract artifacts found".into());
    }
    if has_polyglot_boundary(ctx) {
        score += 10;
        evidence.push("polyglot boundary layout present".into());
    }
    if has_api_drift_checks(ctx) {
        score += 10;
        evidence.push("public API drift checks found".into());
    }
    if has_web_surface(ctx)
        && ctx.all_files.iter().any(|f| {
            f.rel_path == "tsconfig.json" && f.text.to_ascii_lowercase().contains("strict")
        })
    {
        score += 10;
        evidence.push("TypeScript strict mode hinted by `tsconfig.json`".into());
    }
    if has_rust_surface(ctx)
        && ctx.all_files.iter().any(|f| {
            f.text.contains("serde") || f.text.contains("thiserror") || f.text.contains("anyhow")
        })
    {
        score += 8;
        evidence.push("Rust typed boundary helpers found".into());
    }
    if handwritten_api_hits(ctx) {
        score -= 15;
        evidence.push("handwritten web DTO/API marker found".into());
    }
    if product_files(ctx).iter().any(|f| {
        f.text.contains("fetch(") || f.text.contains("axios") || f.text.contains("graphql")
    }) && !has_generated_contracts(ctx)
    {
        score -= 15;
        evidence.push("frontend appears to hand-write API access".into());
    }
    if wrong_layer_db_hits(ctx).is_empty() == false {
        score -= 10;
        evidence.push("DB access found in likely wrong layer".into());
    }
    make_dim("Contract and boundary integrity", score, evidence, notes)
}

fn dimension_proof(ctx: &AuditContext) -> DimensionResult {
    let mut score = 20;
    let mut evidence = vec![];
    let mut notes = vec![];
    if has_one_command(ctx) {
        score += 15;
        evidence.push("one-command setup/validation lane found".into());
    } else {
        notes.push("no one-command setup/validation lane".into());
    }
    if has_fast_lane(ctx) {
        score += 15;
        evidence.push("deterministic fast lane found".into());
    } else {
        notes.push("no deterministic fast lane".into());
    }
    let surface_text = command_surface_text(ctx);
    if surface_text.contains("cargo test")
        || surface_text.contains("nextest")
        || surface_text.contains("pytest")
        || surface_text.contains("vitest")
    {
        score += 10;
        evidence.push("test runner present in automation surface".into());
    }
    if ctx
        .all_files
        .iter()
        .any(|f| f.rel_path.starts_with(".github/workflows/"))
    {
        score += 8;
        evidence.push("GitHub workflow files present".into());
    }
    if ctx
        .all_files
        .iter()
        .any(|f| f.rel_path == "agent/test-map.json" || f.rel_path == "agent/proof-lanes.toml")
    {
        score += 8;
        evidence.push("test/proof routing map present".into());
    }
    if has_humanlint_audit_ci_lane(ctx) {
        score += 8;
        evidence.push("humanlint audit lane found in CI".into());
    } else if is_high_risk_repo(ctx) {
        score -= 8;
        notes.push("no humanlint audit lane in CI".into());
    }
    if has_playwright_e2e(ctx) {
        score += 8;
        evidence.push("web e2e lane present or no web surface".into());
    } else {
        score -= 10;
        notes.push("web surface lacks Playwright/Cypress e2e lane".into());
    }
    let ux = ux_qa_status(ctx);
    if ux.has_rendered_ux_lane {
        score += 6;
        evidence.push("rendered UX QA lane present or no web surface".into());
    } else if ux.web_surface {
        score -= 6;
        notes.push("web surface lacks layered rendered UX QA".into());
    }
    if ux
        .evidence
        .get("geometry_runtime")
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false)
    {
        score += 4;
        evidence.push("DOM geometry UX QA runtime found".into());
    }
    if has_rust_property_tests(ctx) && has_rust_integration_tests(ctx) {
        score += 8;
        evidence.push("Rust property/integration tests present or no Rust surface".into());
    } else if has_rust_surface(ctx) {
        score -= 10;
        notes.push("Rust surface lacks property or integration tests".into());
    }
    if !(surface_text.contains("cargo")
        || surface_text.contains("pytest")
        || surface_text.contains("vitest")
        || surface_text.contains("go test"))
    {
        score -= 10;
        notes.push("no obvious test automation commands".into());
    }
    make_dim("Proof lanes and test routing", score, evidence, notes)
}

fn dimension_security(ctx: &AuditContext) -> DimensionResult {
    let mut score = 20;
    let mut evidence = vec![];
    let mut notes = vec![];
    if ctx.all_files.iter().any(|f| {
        [
            "Cargo.lock",
            "package-lock.json",
            "pnpm-lock.yaml",
            "yarn.lock",
            "poetry.lock",
            "uv.lock",
            "Gemfile.lock",
        ]
        .contains(&f.name.as_str())
    }) {
        score += 12;
        evidence.push("lockfile present".into());
    }
    let surface_text = command_surface_text(ctx);
    if [
        "gitleaks",
        "detect-secrets",
        "secret",
        "audit",
        "deny",
        "dependency-review",
    ]
    .iter()
    .any(|n| surface_text.contains(n))
    {
        score += 12;
        evidence.push("secret or dependency scan tooling found".into());
    }
    if ["syft", "grype", "slsa", "sbom", "cosign"]
        .iter()
        .any(|n| surface_text.contains(n))
    {
        score += 8;
        evidence.push("provenance/SBOM tooling found".into());
    }
    if ["actionlint", "zizmor"]
        .iter()
        .any(|n| surface_text.contains(n))
    {
        score += 8;
        evidence.push("workflow linting tooling found".into());
    }
    if has_security_lane(ctx) {
        score += 8;
        evidence.push("security lane present".into());
    } else {
        notes.push("no explicit security lane found".into());
    }
    if has_humanlint_audit_ci_lane(ctx) {
        score += 6;
        evidence.push("agent-readiness audit gate found in CI".into());
    } else {
        score -= 6;
        notes.push("CI does not run the humanlint audit".into());
    }
    if ctx
        .all_files
        .iter()
        .any(|f| f.rel_path.ends_with(".rs") && f.text.contains("unsafe"))
    {
        score += 4;
        evidence.push("unsafe usage appears to be tracked".into());
    }
    make_dim("Security and supply-chain posture", score, evidence, notes)
}

fn dimension_shape(ctx: &AuditContext) -> DimensionResult {
    let files = product_code_files(ctx);
    if files.is_empty() {
        return make_dim(
            "Code shape and semantic surface",
            65,
            vec!["no authored code files in scope".into()],
            vec![],
        );
    }
    let mut score = 55;
    let mut evidence = vec![];
    let mut notes = vec![];
    if let Some(max) = max_loc(&files) {
        if max > 500 {
            score -= 15;
            evidence.push("code file exceeds 500 LOC".into());
        }
        if max > 1000 {
            score -= 20;
            evidence.push("code file exceeds 1000 LOC".into());
        }
    }
    if files.len() >= 5
        && files.iter().filter(|f| f.line_count <= 300).count() * 10 / files.len() >= 7
    {
        score += 10;
        evidence.push("most code files stay under 300 LOC".into());
    }
    if duplicate_blocks(ctx).is_empty() == false {
        score -= 18;
        evidence.push("duplicate code block marker found".into());
    }
    if !todo_hits(ctx).is_empty() {
        score -= 20;
        evidence.push("TODO/stub marker found".into());
    }
    if fallback_hits(ctx).len() > 1 {
        score -= 18;
        evidence.push("fallback soup marker found".into());
    }
    if !future_hostile_hits(ctx).is_empty() {
        score -= 24;
        evidence.push("future-hostile/dead-language marker found".into());
        notes.push("product/runtime code contains future-hostile or dead-language terms".into());
    }
    if !weak_name_hits(ctx).is_empty() {
        score -= 10;
        evidence.push("weak name marker found".into());
    }
    if !domain_io_hits(ctx).is_empty() {
        score -= 10;
        evidence.push("IO markers found in domain/core files".into());
    }
    make_dim("Code shape and semantic surface", score, evidence, notes)
}

fn dimension_data(ctx: &AuditContext) -> DimensionResult {
    let mut score = 50;
    let mut evidence = vec![];
    let mut notes = vec![];
    let files = product_files(ctx);
    if has_prefix(ctx, "db") || files.iter().any(|f| f.suffix == ".sql") {
        score += 15;
        evidence.push("database surface present".into());
    }
    if has_prefix(ctx, "db/migrations") || has_prefix(ctx, "migrations") {
        score += 10;
        evidence.push("migration directory present".into());
    }
    if files.iter().any(|f| {
        f.text.to_ascii_lowercase().contains("foreign key")
            || f.text.to_ascii_lowercase().contains("check constraint")
            || f.text.to_ascii_lowercase().contains("row level security")
    }) {
        score += 10;
        evidence.push("constraint or RLS language found".into());
    }
    if files.iter().any(|f| {
        f.rel_path.starts_with("db/")
            || f.rel_path.starts_with("crates/adapters")
            || f.rel_path.starts_with("adapters")
            || f.rel_path.starts_with("infra")
            || f.rel_path.starts_with("data")
    }) {
        score += 10;
        evidence.push("data access appears compartmentalized".into());
    }
    let wrong_layer_db = wrong_layer_db_hits(ctx);
    if !wrong_layer_db.is_empty() {
        score -= 20;
        evidence.push(format!(
            "strict DB boundary violation: {}",
            wrong_layer_db[0].path
        ));
        notes.push("direct DB access leaks out of the data boundary".into());
    }
    make_dim("Data truth and workflow safety", score, evidence, notes)
}

fn dimension_observability(ctx: &AuditContext) -> DimensionResult {
    let mut score = 35;
    let mut evidence = vec![];
    let mut notes = vec![];
    let files = product_files(ctx);
    if files.iter().any(|f| {
        f.text.contains("tracing")
            || f.text.contains("opentelemetry")
            || f.text.contains("thiserror")
            || f.text.contains("anyhow")
    }) {
        score += 15;
        evidence.push("observability libraries or patterns found".into());
    }
    if files.iter().any(|f| {
        f.text.contains("request id")
            || f.text.contains("correlation id")
            || f.text.contains("json diagnostics")
    }) {
        score += 10;
        evidence.push("diagnostic shaping hints found".into());
    }
    if has_prefix(ctx, "ops") || has_prefix(ctx, "observability") {
        score += 10;
        evidence.push("ops/observability directory present".into());
    }
    if files.iter().any(|f| {
        f.text.contains("receipt") || f.text.contains("artifact") || f.text.contains("trace")
    }) {
        score += 8;
        evidence.push("repair receipts or raw artifact language found".into());
    }
    if has_agent_friendly_exceptions(ctx) {
        score += 12;
        evidence.push("agent-friendly exception pattern found".into());
    } else if !files.is_empty() {
        score -= 12;
        notes.push("no agent-friendly exception pattern found".into());
    }
    if files.iter().any(|f| {
        f.text.contains("println!") || f.text.contains("console.log") || f.text.contains("print(")
    }) {
        score -= 8;
        notes.push("free-form logging appears in scope".into());
    }
    make_dim("Observability and repair evidence", score, evidence, notes)
}

fn dimension_context(ctx: &AuditContext) -> DimensionResult {
    let mut score = 20;
    let mut evidence = vec![];
    let mut notes = vec![];
    if has_root_agents(ctx) {
        score += 25;
        evidence.push("root `AGENTS.md` present".into());
        score += 10;
        evidence.push("root `AGENTS.md` stays short".into());
    } else {
        notes.push("no root `AGENTS.md`".into());
    }
    if ctx.all_files.iter().any(|f| {
        [
            "agent-map.json",
            "test-map.json",
            "proof-lanes.toml",
            "generated-zones.toml",
            "agent/owner-map.json",
        ]
        .contains(&f.rel_path.as_str())
    }) {
        score += 10;
        evidence.push("machine-readable routing artifacts present".into());
    }
    if ctx
        .all_files
        .iter()
        .any(|f| f.name == "AGENTS.md" && f.rel_path != "AGENTS.md")
    {
        score += 10;
        evidence.push("local instruction files present".into());
    }
    if root_readme_routes(ctx) {
        score += 10;
        evidence.push("root README routes to the right docs".into());
    }
    let missing = missing_core_docs(ctx);
    if !missing.is_empty() {
        score -= 6 * missing.len() as i32;
        notes.push(format!(
            "missing agent-readable docs: {}",
            missing
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    } else {
        score += 8;
        evidence.push("core agent-readable docs present".into());
    }
    if !has_root_agents(ctx)
        && !ctx
            .all_files
            .iter()
            .any(|f| f.name == "AGENTS.md" && f.rel_path != "AGENTS.md")
    {
        score -= 10;
        notes.push("no instruction files to route agents".into());
    }
    make_dim(
        "Context economy and agent instructions",
        score,
        evidence,
        notes,
    )
}

fn dimension_python(ctx: &AuditContext) -> DimensionResult {
    let python_files: Vec<_> = ctx
        .scope_files
        .iter()
        .filter(|f| f.suffix == ".py")
        .cloned()
        .collect();
    let non_optimal = non_optimal_language_hits(ctx);
    if python_files.is_empty() {
        let mut score = 100;
        let mut evidence = vec!["no Python files in scope".into()];
        let mut notes = vec![];
        if !non_optimal.is_empty() {
            score -= 10;
            evidence.push("non-optimal product language marker".into());
            notes.push("runtime code should converge to Rust, TypeScript, SQL, contracts, and bounded Python".into());
        }
        return make_dim(
            "Python containment and polyglot hygiene",
            score,
            evidence,
            notes,
        );
    }
    let mut score = 40;
    let mut evidence = vec![];
    let mut notes = vec![];
    let bad_paths: Vec<_> = python_files
        .iter()
        .filter(|f| !is_allowed_python_path(&f.rel_path))
        .map(|f| f.rel_path.clone())
        .collect();
    if bad_paths.is_empty() {
        score += 30;
        evidence.push("Python stays inside allowed non-product roots".into());
    } else {
        score -= 30;
        evidence.push(format!(
            "Python appears outside the allowed roots: {}",
            bad_paths[0]
        ));
        notes.push("Python leaks into product surface".into());
    }
    if python_files
        .iter()
        .any(|f| f.rel_path.starts_with("python/ai-service"))
    {
        score += 10;
        evidence.push("bounded AI/data service path present".into());
    }
    let ratio = python_ratio(ctx);
    if ratio > 0.3 {
        score -= 35;
        evidence.push(format!(
            "Python is {:.0}% of runtime product code",
            ratio * 100.0
        ));
        notes.push("too much Python for the selected optimal stack".into());
    } else if ratio > 0.15 {
        score -= 15;
        evidence.push(format!(
            "Python is {:.0}% of runtime product code",
            ratio * 100.0
        ));
    }
    if !non_optimal.is_empty() {
        score -= 10;
        evidence.push(format!(
            "non-optimal product language marker: {}",
            non_optimal[0].rel_path
        ));
        notes.push(
            "runtime code should converge to Rust, TypeScript, SQL, contracts, and bounded Python"
                .into(),
        );
    }
    if bad_paths
        .iter()
        .any(|p| p.contains("psycopg") || p.contains("sqlalchemy"))
    {
        score -= 20;
        evidence.push("Python directly touches DB truth outside AI service".into());
    }
    make_dim(
        "Python containment and polyglot hygiene",
        score,
        evidence,
        notes,
    )
}

fn dimension_speed(ctx: &AuditContext) -> DimensionResult {
    let mut score = 20;
    let mut evidence = vec![];
    let mut notes = vec![];
    let surface_text = command_surface_text(ctx);
    if [
        "cargo check",
        "cargo nextest",
        "cargo build --timings",
        "sccache",
        "rust-cache",
        "bacon",
        "vitest",
        "pytest",
        "go test",
        "dotnet test",
        "pnpm",
        "bun test",
        "turbo",
        "nx",
    ]
    .iter()
    .any(|n| surface_text.contains(n))
    {
        score += 20;
        evidence.push("build acceleration markers found".into());
    }
    if [
        "cargo check",
        "nextest",
        "vitest",
        "pytest",
        "go test",
        "dotnet test",
    ]
    .iter()
    .any(|n| surface_text.contains(n))
    {
        score += 10;
        evidence.push("targeted test/build commands found".into());
    }
    if ctx.all_files.iter().any(|f| {
        [
            "Cargo.lock",
            "pnpm-lock.yaml",
            "package-lock.json",
            "yarn.lock",
            "uv.lock",
            "poetry.lock",
        ]
        .contains(&f.name.as_str())
    }) {
        score += 10;
        evidence.push("locked dependency graph present".into());
    }
    if ctx
        .all_files
        .iter()
        .any(|f| f.rel_path.starts_with(".github/workflows") && f.text.contains("cache"))
    {
        score += 10;
        evidence.push("CI cache hint found".into());
    }
    if !has_one_command(ctx) {
        score -= 10;
        notes.push("missing one-command setup/validation".into());
    }
    if !has_fast_lane(ctx) {
        score -= 10;
        notes.push("missing deterministic fast lane".into());
    }
    make_dim("Build speed signals", score, evidence, notes)
}

fn build_findings(
    ctx: &AuditContext,
    dimensions: &[DimensionResult],
    caps_applied: &[String],
) -> Vec<Finding> {
    let dim_by_name: HashMap<_, _> = dimensions.iter().map(|d| (d.name.as_str(), d)).collect();
    let mut findings = vec![];
    let has_context_finding = Cell::new(false);
    let mut add = |severity: &str,
                   category: &str,
                   path: &str,
                   problem: &str,
                   fix: &str,
                   evidence: Vec<String>,
                   rule_id: Option<&str>,
                   line: Option<usize>| {
        if category == "context" {
            has_context_finding.set(true);
        }
        findings.push(Finding {
            severity: severity.into(),
            category: category.into(),
            path: path.into(),
            problem: problem.into(),
            agent_fix: fix.into(),
            evidence,
            rule_id: rule_id.map(|s| s.into()),
            tlr: tlr_for(category).map(|s| s.into()),
            lane: lane_for(category).map(|s| s.into()),
            docs_url: rule_id.and_then(|r| docs_for_rule(r)).map(|s| s.into()),
            owner: owner_for_path(ctx, path).map(|s| s.into()),
            line,
            matched_term: None,
            reason: None,
        })
    };
    if caps_applied.contains(&"no-root-agent-instructions".into()) {
        add(
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
        add(
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
        add("high", "proof", ".", "no deterministic fast lane was detected", "add a fast lane that runs the narrowest deterministic proof loop and keep it canonical", vec!["no fast lane markers found".into()], Some("HLT-004-UNMAPPED-PROOF"), None);
    }
    if caps_applied.contains(&"no-security-lane-on-high-risk-repo".into()) {
        add("high", "security", ".github/workflows", "high-risk repo has no explicit security lane", "add a dedicated security lane with secret scanning, dependency review, and workflow linting", vec!["no security lane markers found".into()], Some("HLT-009-GENERATED-SECURITY"), None);
    }
    if caps_applied.contains(&"generated-contracts-or-public-api-drift-untested".into()) {
        add(
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
        add(
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
        add(
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
    if caps_applied.contains(&"no-humanlint-audit-lane-in-ci".into()) {
        add("high", "audit", ".github/workflows", "CI does not run the humanlint audit lane", "add a CI job that runs `humanlint . --json repo-score.json --md repo-score.md` and uploads both artifacts", vec!["audit output must stay JSON plus Markdown for agent repair routing".into()], None, None);
    }

    if !non_optimal_language_hits(ctx).is_empty() {
        let hit = non_optimal_language_hits(ctx)[0].clone();
        add("high", "stack", &hit.rel_path, "runtime code uses a language outside the chosen optimal stack", "move product runtime behavior to Rust core, TypeScript web, SQL migrations, generated contracts, or bounded `python/ai-service` only", vec![format!("{} uses `{}`", hit.rel_path, hit.suffix), TARGET_STACK.into()], None, None);
    }
    let ratio = python_ratio(ctx);
    if ratio > 0.15 {
        add(if ratio > 0.30 { "high" } else { "medium" }, "python", "python/ai-service", "Python is too large a share of runtime product code for this standard", "keep Python bounded to model/data work and move durable product truth, authz, workflows, and core behavior into Rust", vec!["Python share is above the soft cap".into()], None, None);
    }
    if !todo_hits(ctx).is_empty() {
        let hit = todo_hits(ctx)[0].clone();
        add("high", "vibe", &hit.path, "product code contains TODO/stub/unimplemented/unreachable placeholder markers", "replace placeholders with implemented behavior, typed unsupported-state errors, or a tracked exception record with docs", vec![format!("{}:{} {}", hit.path, hit.line.unwrap_or(1), hit.text)], Some("HLT-001-DEAD-MARKER"), Some(hit.line.unwrap_or(1)));
    }
    if fallback_hits(ctx).len() > 1 {
        let hit = fallback_hits(ctx)[0].clone();
        add("high", "vibe", &hit.path, "fallback soup detected in product code", "collapse fallback chains into explicit typed states with bounded retry policy, telemetry, and documented repair guidance", vec![format!("{}:{} {}", hit.path, hit.line.unwrap_or(1), hit.text)], Some("HLT-001-DEAD-MARKER"), Some(hit.line.unwrap_or(1)));
    }
    for hit in future_hostile_hits(ctx) {
        add(
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
    if !duplicate_blocks(ctx).is_empty() {
        let hit = duplicate_blocks(ctx)[0].clone();
        add("high", "vibe", &hit.path, "duplicated product code block detected", "extract the duplicated behavior behind one named boundary and add focused tests before changing behavior", vec![hit.problem.clone()], None, hit.line);
    }
    if !generated_zone_issues(ctx).is_empty() {
        let hit = generated_zone_issues(ctx)[0].clone();
        add("high", "generated", &hit.path, "generated zone is not protected strongly enough against hand edits", "add `agent/generated-zones.toml`, require generated/do-not-edit markers, and route repairs to the source contract", vec![hit.problem.clone()], Some("HLT-002-GENERATED-MUTATION"), hit.line);
    }
    if !wrong_layer_db_hits(ctx).is_empty() {
        let hit = wrong_layer_db_hits(ctx)[0].clone();
        add("high", "data", &hit.path, "direct database access appears in a wrong layer", "move SQL and DB clients to `crates/adapters` or `db/`; expose typed application/domain APIs upward", vec![hit.problem.clone()], Some("HLT-006-DIRECT-DB-WRONG-LAYER"), hit.line);
    }
    if caps_applied.contains(&"missing-web-e2e-lane".into()) {
        add("high", "test", "apps/web", "web surface lacks a Playwright/Cypress e2e lane", "add Playwright e2e tests for critical user flows and wire them into the fast or CI proof map", vec!["web surface detected".into()], Some("HLT-013-RENDERED-UX-GAP"), None);
    }
    if caps_applied.contains(&"missing-rendered-ux-qa-lane".into()) {
        add("high", "ux-qa", "apps/web", "web surface lacks layered rendered UX QA evidence", "add Storybook state coverage, Playwright screenshots, visual review or `@humanlint/ux-qa`, accessibility scans, CLS checks, generated mocks, and design tokens", vec!["rendered UX QA lane missing".into()], Some("HLT-013-RENDERED-UX-GAP"), None);
    }
    if !prompt_injection_hits(ctx).is_empty() {
        let hit = prompt_injection_hits(ctx)[0].clone();
        add("high","security",&hit.path,"trusted agent/tool policy contains prompt-injection or policy-bypass language","isolate untrusted instructions from trusted policy, remove bypass wording, and validate tool calls against the repository standard", vec![hit.problem], Some("HLT-011-PROMPT-INJECTION"), hit.line);
    }
    if !agency_hits(ctx).is_empty() {
        let hit = agency_hits(ctx)[0].clone();
        add("high","security",&hit.path,"agent/tool permissions appear broader than the requested proof lane","replace broad terminal/browser/network/filesystem permissions with least-privilege lane profiles and explicit approval gates", vec![hit.problem], Some("HLT-012-OVERBROAD-AGENCY"), hit.line);
    }
    if !secret_hits(ctx).is_empty() {
        let hit = secret_hits(ctx)[0].clone();
        add("critical","security",&hit.path,"secret-like value or credential material appears in repository text","remove and rotate the credential, add local and CI secret scanning, and scan transcripts/artifacts/MCP config for related exposure", vec![hit.problem], Some("HLT-010-SECRET-SPRAWL"), hit.line);
    }
    if !false_green_hits(ctx).is_empty() {
        let hit = false_green_hits(ctx)[0].clone();
        add("high","test",&hit.path,"test code contains disabled, focused, tautological, or snapshot-only proof","replace false-green tests with behavior assertions, red/green evidence, and mutation or fault checks for changed behavior", vec![hit.problem], Some("HLT-008-FALSE-GREEN-RISK"), hit.line);
    }
    if !destructive_sql_hits(ctx).is_empty() {
        let hit = destructive_sql_hits(ctx)[0].clone();
        add("high","data",&hit.path,"destructive migration lacks rollback, backfill, lock, or safety evidence","add migration safety evidence: rollback/down plan, backfill strategy, lock timeout, staged deploy note, and DB proof lane", vec![hit.problem], Some("HLT-006-DIRECT-DB-WRONG-LAYER"), hit.line);
    }
    if caps_applied.contains(&"missing-rust-property-or-integration-tests".into()) {
        add("high","test","crates/","Rust surface lacks required property and/or integration tests","add `proptest` or equivalent invariant tests plus `tests/` integration coverage routed through `cargo nextest` or `cargo test`", vec!["Rust surface detected".into()], Some("HLT-008-FALSE-GREEN-RISK"), None);
    }
    if caps_applied.contains(&"no-agent-friendly-exception-pattern".into()) {
        add("high","exceptions","crates/domain","no agent-friendly exception/error pattern was detected","define typed errors with stable code, purpose, reason, common fixes, and documentation URL; mirror the shape in TypeScript boundaries", vec!["agents need structured repair hints instead of opaque failures".into()], Some("HLT-017-OPAQUE-OBSERVABILITY"), None);
    }
    if caps_applied.contains(&"missing-agent-readable-docs".into()) {
        let missing = missing_core_docs(ctx);
        add("medium","docs","docs/","agent-readable documentation is incomplete","add concise docs for architecture, boundaries, tests, generated zones, and audit rules; route them from root `AGENTS.md`", missing, None, None);
    }

    if let Some(ownership) = dim_by_name.get("Ownership and navigation surface") {
        if ownership.score < 55 && !has_context_finding.get() {
            add("medium","context",".","navigation surface is thin for agent work","add local routing docs and machine-readable owner/test maps where the repo needs them", ownership.evidence.iter().take(2).cloned().collect(), None, None);
        }
    }
    if let Some(shape) = dim_by_name.get("Code shape and semantic surface") {
        if shape.notes.iter().any(|n| n.contains("large code files")) {
            if let Some(max) = max_loc(&product_code_files(ctx)) {
                add(if max <= 1000 { "medium" } else { "high" }, "shape", ".", &format!("largest code file is {} LOC", max), "split the file along ownership or semantic boundaries before agents have to patch it again", vec![format!("largest authored code file: {} LOC", max)], None, None);
            }
        }
    }
    findings
}

fn build_agent_fix_queue(findings: &[Finding]) -> Vec<AgentFix> {
    let mut seen = HashSet::new();
    let mut ordered = findings.to_vec();
    ordered.sort_by_key(finding_priority);
    let mut queue = vec![];
    for finding in ordered.iter() {
        let key = (finding.path.clone(), finding.agent_fix.clone());
        if !seen.insert(key) {
            continue;
        }
        queue.push(AgentFix {
            path: finding.path.clone(),
            priority: finding.severity.clone(),
            rule_id: finding.rule_id.clone(),
            tlr: finding.tlr.clone(),
            lane: finding.lane.clone(),
            owner: finding.owner.clone(),
            task: finding.agent_fix.clone(),
            why: finding.problem.clone(),
        });
    }
    queue
}

fn finding_priority(f: &Finding) -> (i32, i32, String) {
    let severity = match f.severity.as_str() {
        "critical" => 0,
        "high" => 1,
        "medium" => 2,
        _ => 3,
    };
    let tlr_priority = match f.tlr.as_deref().unwrap_or("") {
        "Security" => 0,
        "Business truth" => 1,
        "Contracts/data" => 2,
        "Verification" => 3,
        "Repair" => 4,
        "Context/setup" => 5,
        _ => 6,
    };
    (tlr_priority, severity, f.path.clone())
}

fn ux_qa_status(ctx: &AuditContext) -> UxQaReadiness {
    let evidence = serde_json::json!({
        "storybook": paths_with(ctx, &[".storybook/", ".stories.", ".story."], &["@storybook", "storybook", "component story format", "csf"]),
        "playwright_visual": paths_with(ctx, &[], &["tohavescreenshot", "page.screenshot", "locator.screenshot", "visual comparisons", "screenshotpath"]),
        "visual_review": paths_with(ctx, &["backstop", "loki", "argos", "chromatic", "percy", "applitools"], &["@argos-ci", "argos", "chromatic", "percy", "applitools", "backstopjs", "loki", "visual regression", "visual review"]),
        "accessibility": paths_with(ctx, &[], &["@axe-core", "axe-core", "pa11y", "storybook-addon-a11y", "eslint-plugin-jsx-a11y", "accessibility testing", "wcag"]),
        "layout_stability": paths_with(ctx, &[], &["lighthouse", "lhci", "web-vitals", "cumulative layout shift", "layout shift", "cls"]),
        "api_mocks": paths_with(ctx, &[], &["msw", "mock service worker", "msw-storybook-addon", "mockserviceworker", "orval"]),
        "design_tokens": paths_with(ctx, &["tokens/", "design-tokens", "style-dictionary"], &["design tokens", "design-token", "style dictionary", "style-dictionary", "figma variables", "semantic tokens"]),
        "geometry_runtime": paths_with(ctx, &["packages/ux-qa", "ux-qa"], &["@humanlint/ux-qa", "humanlint-ux-qa", "analyzepage", "expectnouxviolations", "edge clearance", "target size", "getboundingclientrect"]),
        "artifact_backed_proof": paths_with(ctx, &["ux-qa-artifacts", "test-results", "playwright-report"], &["--artifacts-dir", "--screenshot", "--aria-snapshot", "artifactpath", "artifactsdir", "ariasnapshot", "tohavescreenshot", "tomatchariasnapshot", "page.screenshot", "trace"]),
    });
    let web_surface = has_web_surface(ctx);
    let missing = if !web_surface {
        vec![]
    } else {
        let mut v = vec![];
        let storybook = evidence
            .get("storybook")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(true);
        let playwright = evidence
            .get("playwright_visual")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(true);
        let visual = evidence
            .get("visual_review")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(true);
        let accessibility = evidence
            .get("accessibility")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(true);
        let layout = evidence
            .get("layout_stability")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(true);
        let mocks = evidence
            .get("api_mocks")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(true);
        let design = evidence
            .get("design_tokens")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(true);
        let proof = evidence
            .get("artifact_backed_proof")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(true);
        if storybook {
            v.push("Storybook state coverage".into());
        }
        if playwright {
            v.push("Playwright screenshot capture".into());
        }
        if visual {
            v.push("visual review or geometry runtime".into());
        }
        if accessibility {
            v.push("accessibility automation".into());
        }
        if layout {
            v.push("layout stability checks".into());
        }
        if mocks {
            v.push("generated API mocks".into());
        }
        if design {
            v.push("design token discipline".into());
        }
        if proof {
            v.push("artifact-backed UX proof receipts".into());
        }
        v
    };
    UxQaReadiness {
        web_surface,
        has_rendered_ux_lane: !web_surface || missing.is_empty(),
        missing_categories: missing,
        evidence,
    }
}

fn paths_with(ctx: &AuditContext, path_markers: &[&str], markers: &[&str]) -> Vec<String> {
    let mut out = vec![];
    for f in &ctx.all_files {
        let rel = f.rel_path.to_ascii_lowercase();
        let text = f.text.to_ascii_lowercase();
        let path_hit = path_markers
            .iter()
            .any(|m| rel.contains(&m.to_ascii_lowercase()));
        let text_hit = markers
            .iter()
            .any(|m| text.contains(&m.to_ascii_lowercase()));
        if path_hit || text_hit {
            out.push(f.rel_path.clone());
        }
        if out.len() >= 5 {
            break;
        }
    }
    out
}

fn command_surface_text(ctx: &AuditContext) -> String {
    ctx.all_files
        .iter()
        .filter(|f| {
            matches!(
                f.name.as_str(),
                "Justfile"
                    | "Makefile"
                    | "makefile"
                    | "justfile"
                    | "package.json"
                    | "taskfile.yml"
                    | "taskfile.yaml"
            ) || f.rel_path.starts_with(".github/workflows")
        })
        .map(|f| f.text.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

fn has_root_agents(ctx: &AuditContext) -> bool {
    ctx.all_files.iter().any(|f| f.rel_path == "AGENTS.md")
}
fn has_one_command(ctx: &AuditContext) -> bool {
    let t = command_surface_text(ctx);
    t.contains("setup")
        || t.contains("check")
        || t.contains("verify")
        || t.contains("install")
        || t.contains("bootstrap")
}
fn has_fast_lane(ctx: &AuditContext) -> bool {
    let t = command_surface_text(ctx);
    t.contains("cargo check")
        || t.contains("cargo nextest")
        || t.contains("vitest")
        || t.contains("pytest")
        || t.contains("go test")
        || t.contains("humanlint")
}
fn has_security_lane(ctx: &AuditContext) -> bool {
    let t = command_surface_text(ctx);
    t.contains("gitleaks")
        || t.contains("dependency-review")
        || t.contains("syft")
        || t.contains("grype")
        || t.contains("zizmor")
        || t.contains("security")
}
fn has_humanlint_audit_ci_lane(ctx: &AuditContext) -> bool {
    let t = command_surface_text(ctx).to_ascii_lowercase();
    t.contains("humanlint") && t.contains("repo-score")
}
fn is_high_risk_repo(ctx: &AuditContext) -> bool {
    product_code_files(ctx).iter().any(|f| f.is_code)
        || ctx.all_files.iter().any(|f| {
            ["package.json", "pyproject.toml", "Cargo.toml", "go.mod"].contains(&f.name.as_str())
        })
}
fn has_contract_surface(ctx: &AuditContext) -> bool {
    has_prefix(ctx, "contracts")
        || ctx.all_files.iter().any(|f| {
            f.text.contains("openapi") || f.text.contains("protobuf") || f.suffix == ".proto"
        })
}
fn has_polyglot_boundary(ctx: &AuditContext) -> bool {
    has_prefix(ctx, "apps/web")
        || has_prefix(ctx, "apps/api")
        || has_prefix(ctx, "crates/domain")
        || has_prefix(ctx, "crates/application")
        || has_prefix(ctx, "crates/adapters")
}
fn has_generated_contracts(ctx: &AuditContext) -> bool {
    ctx.all_files.iter().any(|f| f.text.contains("generated"))
}
fn has_api_drift_checks(ctx: &AuditContext) -> bool {
    let t = command_surface_text(ctx);
    t.contains("cargo public-api")
        || t.contains("cargo semver")
        || t.contains("api-extractor")
        || t.contains("tsd")
}
fn has_secret_or_dependency_scans(ctx: &AuditContext) -> bool {
    let t = command_surface_text(ctx);
    t.contains("gitleaks")
        || t.contains("dependency-review")
        || t.contains("syft")
        || t.contains("grype")
}
fn has_playwright_e2e(ctx: &AuditContext) -> bool {
    if !has_web_surface(ctx) {
        return true;
    }
    let t = command_surface_text(ctx);
    t.contains("playwright")
        || ctx
            .all_files
            .iter()
            .any(|f| f.rel_path.contains("e2e") || f.text.contains("@playwright/test"))
}
fn has_rust_surface(ctx: &AuditContext) -> bool {
    ctx.all_files
        .iter()
        .any(|f| f.suffix == ".rs" && is_runtime_stack_surface(f))
}
fn has_rust_property_tests(ctx: &AuditContext) -> bool {
    !has_rust_surface(ctx)
        || ctx.all_files.iter().any(|f| {
            f.text.contains("proptest")
                || f.text.contains("quickcheck")
                || f.text.contains("rstest")
        })
}
fn has_rust_integration_tests(ctx: &AuditContext) -> bool {
    !has_rust_surface(ctx)
        || ctx.all_files.iter().any(|f| {
            f.suffix == ".rs"
                && (f.rel_path.contains("/tests/")
                    || f.rel_path.starts_with("tests/")
                    || f.text.contains("#[test]")
                    || f.text.contains("#[tokio::test]"))
        })
}
fn has_web_surface(ctx: &AuditContext) -> bool {
    ctx.all_files.iter().any(|f| {
        f.rel_path.starts_with("apps/web")
            || f.rel_path.starts_with("frontend")
            || f.rel_path.starts_with("ui")
            || f.rel_path.starts_with("packages/web")
            || f.rel_path.starts_with("packages/ui")
            || f.text.contains("react")
            || f.text.contains("vite")
            || f.text.contains("storybook")
    })
}
fn is_runtime_stack_surface(file: &FileInfo) -> bool {
    !file.is_generated
        && !matches!(
            file.name.as_str(),
            "AGENTS.md" | "Justfile" | "Makefile" | "README.md" | "makefile" | "justfile"
        )
        && !file.rel_path.starts_with("agent/")
        && !file.rel_path.starts_with("crates/humanlint/")
        && !["contracts/", "db/", "migrations/", "ops/", "/.github/"]
            .iter()
            .any(|p| file.rel_path.starts_with(p))
        && !file.rel_path.starts_with("docs/")
        && !file.rel_path.starts_with(".github/")
        && !file.rel_path.starts_with("paper/")
        && !file.rel_path.starts_with("reference/")
        && !file.rel_path.starts_with("packages/ux-qa/")
        && !file.rel_path.starts_with("scripts/")
        && !file.rel_path.starts_with("tests/")
        && !file.rel_path.starts_with("tips/")
        && !file.rel_path.starts_with("tools/")
}
fn product_files(ctx: &AuditContext) -> Vec<FileInfo> {
    ctx.all_files
        .iter()
        .filter(|f| is_runtime_stack_surface(f))
        .cloned()
        .collect()
}
fn product_code_files(ctx: &AuditContext) -> Vec<FileInfo> {
    product_files(ctx)
        .into_iter()
        .filter(|f| f.is_code)
        .collect()
}
fn python_ratio(ctx: &AuditContext) -> f64 {
    let files = product_files(ctx);
    let total: usize = files.iter().map(|f| f.line_count).sum();
    let py: usize = files
        .iter()
        .filter(|f| f.suffix == ".py")
        .map(|f| f.line_count)
        .sum();
    if total == 0 {
        0.0
    } else {
        py as f64 / total as f64
    }
}
fn is_allowed_python_path(path: &str) -> bool {
    ALLOWED_PYTHON_ROOTS
        .iter()
        .any(|p| path == *p || path.starts_with(&format!("{}/", p.trim_end_matches('/'))))
}
fn bad_python_paths(ctx: &AuditContext) -> bool {
    ctx.all_files
        .iter()
        .any(|f| f.suffix == ".py" && !is_allowed_python_path(&f.rel_path))
}
fn non_optimal_language_hits(ctx: &AuditContext) -> Vec<FileInfo> {
    product_code_files(ctx)
        .into_iter()
        .filter(|f| {
            [
                ".c", ".cc", ".cpp", ".cs", ".dart", ".ex", ".exs", ".go", ".h", ".hh", ".hpp",
                ".java", ".js", ".jsx", ".kt", ".kts", ".lua", ".m", ".mm", ".php", ".rb",
                ".scala", ".swift",
            ]
            .contains(&f.suffix.as_str())
                || (f.suffix == ".py" && !f.rel_path.starts_with("python/ai-service/"))
        })
        .collect()
}
fn todo_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    pattern_hits(&product_code_files(ctx), TODO_PATTERNS)
}
fn fallback_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    let hits = pattern_hits(&product_code_files(ctx), FALLBACK_PATTERNS);
    if hits.len() <= 1 {
        vec![]
    } else {
        hits
    }
}
fn secret_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    static SECRET_ASSIGNMENT: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?i)\b(?:api[_-]?key|access[_-]?token|client[_-]?secret|private[_-]?key|password|secret)\b\s*[:=]\s*['"]?[A-Za-z0-9_+/=.-]{8,}"#,
        )
        .expect("secret regex is valid")
    });
    let mut hits = vec![];
    for file in &ctx.all_files {
        if file.is_generated
            || file.rel_path.starts_with("crates/humanlint/")
            || file.rel_path.starts_with("docs/")
            || file.rel_path.starts_with("paper/")
            || file.rel_path.starts_with("reference/")
            || file.rel_path.starts_with("tips/")
        {
            continue;
        }
        for (idx, line) in file.text.lines().enumerate() {
            let strong_token = [
                "AKIA",
                "ghp_",
                "xoxb-",
                "xoxa-",
                "xoxp-",
                "sk-",
                "-----BEGIN ",
            ]
            .iter()
            .any(|needle| line.contains(needle));
            if strong_token
                || SECRET_ASSIGNMENT.is_match(line)
                || (line.to_ascii_lowercase().contains("eyj")
                    && line.to_ascii_lowercase().contains("token"))
            {
                let problem = line.trim().chars().take(160).collect::<String>();
                hits.push(FindingHit {
                    path: file.rel_path.clone(),
                    line: Some(idx + 1),
                    text: problem.clone(),
                    matched_term: Some("secret-like material".into()),
                    agent_fix: "remove the credential, rotate the secret, and replace it with a config reference or test fixture that cannot be used as a live credential".into(),
                    problem,
                });
                if hits.len() >= 20 {
                    return hits;
                }
            }
        }
    }
    hits
}
fn prompt_injection_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    pattern_hits(
        &ctx.all_files
            .iter()
            .filter(|f| {
                !f.is_generated
                    && (f.rel_path == "AGENTS.md"
                        || f.rel_path.starts_with("agent/")
                        || f.rel_path.starts_with(".github/"))
            })
            .cloned()
            .collect::<Vec<_>>(),
        PROMPT_PATTERNS,
    )
}
fn agency_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    pattern_hits(
        &ctx.all_files
            .iter()
            .filter(|f| {
                !f.is_generated
                    && (f.rel_path == "AGENTS.md"
                        || f.rel_path.starts_with("agent/")
                        || f.rel_path.starts_with(".github/"))
            })
            .cloned()
            .collect::<Vec<_>>(),
        AGENCY_PATTERNS,
    )
}
fn false_green_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    pattern_hits(
        &ctx.all_files
            .iter()
            .filter(|f| {
                !f.is_generated
                    && (f.rel_path.contains("/test")
                        || f.rel_path.contains("/spec")
                        || f.name.ends_with(".test.ts")
                        || f.name.ends_with(".spec.ts")
                        || f.name.ends_with("_test.rs")
                        || f.name.ends_with("_test.go"))
            })
            .cloned()
            .collect::<Vec<_>>(),
        FALSE_GREEN_PATTERNS,
    )
}
fn destructive_sql_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    pattern_hits(
        &ctx.all_files
            .iter()
            .filter(|f| {
                f.suffix == ".sql"
                    && !f.is_generated
                    && (f.rel_path.starts_with("db/")
                        || f.rel_path.starts_with("migrations/")
                        || f.rel_path.starts_with("crates/adapters/")
                        || f.rel_path.starts_with("apps/api/migrations/"))
            })
            .cloned()
            .collect::<Vec<_>>(),
        DESTRUCTIVE_SQL_PATTERNS,
    )
}
fn generated_zone_issues(ctx: &AuditContext) -> Vec<FindingHit> {
    let generated = ctx
        .all_files
        .iter()
        .filter(|f| f.is_generated && f.is_code)
        .cloned()
        .collect::<Vec<_>>();
    if generated.is_empty() {
        return vec![];
    }
    let mut issues = vec![];
    if !ctx
        .all_files
        .iter()
        .any(|f| f.rel_path == "agent/generated-zones.toml")
    {
        issues.push(FindingHit::new(
            &generated[0].rel_path,
            1,
            "generated code exists without `agent/generated-zones.toml` ownership rules",
        ));
    }
    for file in generated {
        if !file.text.to_ascii_lowercase().contains("generated")
            && !file.text.to_ascii_lowercase().contains("do not edit")
        {
            issues.push(FindingHit::new(
                &file.rel_path,
                1,
                "generated file lacks a clear generated/do-not-edit marker",
            ));
        }
        if !pattern_hits(&vec![file.clone()], TODO_PATTERNS).is_empty() {
            issues.push(FindingHit::new(
                &file.rel_path,
                1,
                "generated file contains TODO/stub markers",
            ));
        }
    }
    issues
}
fn handwritten_api_hits(ctx: &AuditContext) -> bool {
    product_code_files(ctx).iter().any(|f| {
        (f.rel_path.starts_with("apps/web/")
            || f.rel_path.starts_with("frontend/")
            || f.rel_path.starts_with("ui/")
            || f.rel_path.starts_with("src/"))
            && (f.text.contains("fetch(")
                || f.text.contains("axios.")
                || f.text.contains("XMLHttpRequest")
                || f.text.contains("Request") && f.text.contains("Response"))
    })
}
fn wrong_layer_db_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    let mut hits = vec![];
    for file in product_files(ctx) {
        if !file.is_code && file.suffix != ".sql" {
            continue;
        }
        if file.rel_path.starts_with("db/")
            || file.rel_path.starts_with("migrations/")
            || file.rel_path.starts_with("crates/adapters/")
        {
            continue;
        }
        if ["apps/web/", "crates/domain/", "frontend/", "ui/", "src/"]
            .iter()
            .any(|p| file.rel_path.starts_with(p))
            && [
                "select ", "insert ", "update ", "delete ", "sqlx", "diesel", "psycopg", "sqlite3",
            ]
            .iter()
            .any(|m| file.text.to_ascii_lowercase().contains(m))
        {
            hits.push(FindingHit::new(
                &file.rel_path,
                1,
                "DB marker in non-adapter layer",
            ));
        }
    }
    hits
}
fn has_agent_friendly_exceptions(ctx: &AuditContext) -> bool {
    product_files(ctx).iter().any(|f| {
        let lower = f.text.to_ascii_lowercase();
        let shape = lower.contains("thiserror")
            || lower.contains("enum error")
            || lower.contains("extends error")
            || lower.contains("exception");
        let markers = [
            "purpose",
            "reason",
            "common fixes",
            "docs_url",
            "repair_hint",
        ]
        .iter()
        .filter(|m| lower.contains(**m))
        .count();
        shape && markers >= 3
    })
}
fn missing_core_docs(ctx: &AuditContext) -> Vec<String> {
    let present: HashSet<_> = ctx.all_files.iter().map(|f| f.rel_path.as_str()).collect();
    let mut missing = vec![];
    for need in ["AGENTS.md", "README.md"] {
        if !present.contains(need) {
            missing.push(need.into());
        }
    }
    if !(present.contains("docs/architecture.md") || present.contains("docs/boundaries.md")) {
        missing.push("docs/architecture.md or docs/boundaries.md".into());
    }
    if (has_web_surface(ctx) || has_rust_surface(ctx)) && !present.contains("docs/testing.md") {
        missing.push("docs/testing.md".into());
    }
    missing
}
fn root_readme_routes(ctx: &AuditContext) -> bool {
    ctx.all_files
        .iter()
        .find(|f| f.rel_path == "README.md")
        .map(|f| {
            let lower = f.text.to_ascii_lowercase();
            [
                "build",
                "flow",
                "layout",
                "map",
                "paper/",
                "reference/",
                "tools/",
                "validate",
                "workspace",
            ]
            .iter()
            .any(|m| lower.contains(m))
        })
        .unwrap_or(false)
}
fn has_prefix(ctx: &AuditContext, prefix: &str) -> bool {
    ctx.all_files.iter().any(|f| {
        f.rel_path == prefix
            || f.rel_path
                .starts_with(&format!("{}/", prefix.trim_end_matches('/')))
    })
}
fn max_loc(files: &[FileInfo]) -> Option<usize> {
    files.iter().map(|f| f.line_count).max()
}
fn domain_io_hits(ctx: &AuditContext) -> Vec<String> {
    product_code_files(ctx)
        .into_iter()
        .filter(|f| f.rel_path.contains("/core/") || f.rel_path.contains("/domain/"))
        .filter(|f| {
            [
                "fetch(",
                "open(",
                "read(",
                "requests.",
                "socket",
                "std::fs",
                "subprocess",
                "write(",
                "println!",
            ]
            .iter()
            .any(|m| f.text.contains(m))
        })
        .map(|f| f.rel_path)
        .collect()
}
fn weak_name_hits(_ctx: &AuditContext) -> Vec<String> {
    vec![]
}
fn duplicate_blocks(ctx: &AuditContext) -> Vec<FindingHit> {
    let mut seen = HashMap::new();
    let mut dups = vec![];
    for file in product_code_files(ctx) {
        let lines: Vec<_> = file
            .text
            .lines()
            .filter_map(|l| {
                let l = l.trim();
                if l.is_empty()
                    || l.starts_with("//")
                    || l.starts_with("#")
                    || l.starts_with("use ")
                    || l.starts_with("import ")
                {
                    return None;
                }
                let norm = l
                    .replace('"', "\"S\"")
                    .replace('\'', "\"S\"")
                    .replace('`', "\"S\"")
                    .chars()
                    .map(|c| if c.is_ascii_digit() { 'N' } else { c })
                    .collect::<String>();
                if norm.len() < 12 {
                    None
                } else {
                    Some(norm)
                }
            })
            .collect();
        for win in lines.windows(8) {
            let body = win.join("\n");
            if let Some(prev) = seen.insert(body.clone(), format!("{}:1", file.rel_path)) {
                dups.push(FindingHit::new(
                    &file.rel_path,
                    1,
                    &format!("duplicate block also appears at {}", prev),
                ));
            }
        }
    }
    dups
}
fn future_hostile_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    let mut out = vec![];
    for file in product_code_files(ctx) {
        if is_future_hostile_allowlisted(&file) {
            continue;
        }
        for (idx, line) in file.text.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            if let Some(term) = FUTURE_HOSTILE_TERMS
                .iter()
                .find(|term| lower.contains(*term))
            {
                out.push(FindingHit { path: file.rel_path.clone(), line: Some(idx + 1), text: line.to_string(), matched_term: Some((*term).into()), agent_fix: "remove or rename the marker, implement the intended behavior, model a typed unsupported state, or move docs/generated/vendor/product-copy text into an allowlisted context".into(), problem: format!("future-hostile/dead-language term `{}` appears", term) });
            }
        }
    }
    out
}
fn is_future_hostile_allowlisted(file: &FileInfo) -> bool {
    file.is_generated
        || FUTURE_HOSTILE_ALLOWLIST_PREFIXES
            .iter()
            .any(|p| file.rel_path.starts_with(p))
        || FUTURE_HOSTILE_PRODUCT_COPY_PARTS
            .iter()
            .any(|p| file.rel_path.to_ascii_lowercase().contains(p))
}

#[derive(Clone)]
struct FindingHit {
    path: String,
    line: Option<usize>,
    text: String,
    matched_term: Option<String>,
    agent_fix: String,
    problem: String,
}

impl FindingHit {
    fn new(path: &str, line: usize, text: &str) -> Self {
        Self {
            path: path.into(),
            line: Some(line),
            text: text.into(),
            matched_term: None,
            agent_fix: String::new(),
            problem: text.into(),
        }
    }
}

fn pattern_hits(files: &[FileInfo], patterns: &[&str]) -> Vec<FindingHit> {
    if files.is_empty() || patterns.is_empty() {
        return vec![];
    }
    let ac = AhoCorasick::new(patterns).unwrap();
    let mut out = vec![];
    for file in files {
        for (idx, line) in file.text.lines().enumerate() {
            if let Some(mat) = ac.find(line) {
                out.push(FindingHit {
                    path: file.rel_path.clone(),
                    line: Some(idx + 1),
                    text: line.trim().chars().take(160).collect(),
                    matched_term: Some(patterns[mat.pattern()].to_string()),
                    agent_fix: String::new(),
                    problem: line.trim().chars().take(160).collect(),
                });
                if out.len() >= 20 {
                    return out;
                }
            }
        }
    }
    out
}

fn tlr_for(category: &str) -> Option<&'static str> {
    Some(match category {
        "security" => "Security",
        "python" => "Business truth",
        "data" => "Contracts/data",
        "test" => "Verification",
        "audit" => "Context/setup",
        "generated" => "Contracts/data",
        "context" => "Context/setup",
        "vibe" => "Entropy",
        "shape" => "Entropy",
        "docs" => "Context/setup",
        "proof" => "Verification",
        "stack" => "Context/setup",
        _ => return None,
    })
}
fn lane_for(category: &str) -> Option<&'static str> {
    Some(match category {
        "security" => "security",
        "python" => "contract",
        "data" => "db",
        "test" => "fast",
        "audit" => "audit",
        "generated" => "contract",
        "context" => "fast",
        "vibe" => "fast",
        "shape" => "fast",
        "docs" => "audit",
        "proof" => "fast",
        "stack" => "audit",
        _ => return None,
    })
}
fn docs_for_rule(rule: &str) -> Option<&'static str> {
    Some(match rule {
        "HLT-001-DEAD-MARKER" => "docs/audit-rubric.md#future-hostile-language-rule",
        "HLT-002-GENERATED-MUTATION" => "agent/HUMANLINT_STANDARD.md#generated-zones",
        "HLT-004-UNMAPPED-PROOF" => "agent/HUMANLINT_STANDARD.md#proof-lanes",
        "HLT-005-PYTHON-PRODUCT-TRUTH" => "docs/agent-native-standard.md",
        "HLT-006-DIRECT-DB-WRONG-LAYER" => "docs/audit-rubric.md#required-shape",
        "HLT-007-HANDWRITTEN-CONTRACT" => "docs/audit-rubric.md#known-vibe-coding-insults",
        "HLT-008-FALSE-GREEN-RISK" => "docs/testing.md",
        "HLT-009-GENERATED-SECURITY" => "docs/audit-rubric.md#top-level-risk-mapping",
        "HLT-010-SECRET-SPRAWL" => "docs/audit-rubric.md#top-level-risk-mapping",
        "HLT-011-PROMPT-INJECTION" => "docs/audit-rubric.md#top-level-risk-mapping",
        "HLT-012-OVERBROAD-AGENCY" => "docs/audit-rubric.md#top-level-risk-mapping",
        "HLT-013-RENDERED-UX-GAP" => "docs/testing.md",
        "HLT-016-SUPPLY-CHAIN-DRIFT" => "docs/audit-rubric.md#top-level-risk-mapping",
        "HLT-017-OPAQUE-OBSERVABILITY" => "agent/HUMANLINT_STANDARD.md#repair-receipts",
        "HLT-018-PERF-CONCURRENCY-DRIFT" => "docs/testing.md",
        _ => return None,
    })
}
fn owner_for_path(_ctx: &AuditContext, rel_path: &str) -> Option<&'static str> {
    for (prefix, owner) in OWNER_MAP_PREFIXES {
        if rel_path == *prefix || rel_path.starts_with(prefix) {
            return Some(owner);
        }
    }
    None
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
