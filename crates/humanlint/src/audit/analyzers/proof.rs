use crate::audit::analyzers;
use crate::audit::helpers::*;
use crate::model::DimensionResult;

pub fn analyze(ctx: &AuditContext) -> DimensionResult {
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
    let ux = analyzers::ux_qa_status(ctx);
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
