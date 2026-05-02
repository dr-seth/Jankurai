use crate::audit::helpers::*;
use crate::model::DimensionResult;

pub fn analyze(ctx: &AuditContext) -> DimensionResult {
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
    let security_text = security_lane_text(ctx);
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
        .any(|n| security_text.contains(n))
    {
        score += 8;
        evidence.push("provenance/SBOM tooling found".into());
    }
    if ["actionlint", "zizmor"]
        .iter()
        .any(|n| security_text.contains(n))
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
    if ctx
        .all_files
        .iter()
        .any(|f| f.rel_path == "tools/security-lane.sh")
    {
        score += 6;
        evidence.push("canonical security lane wrapper present".into());
    }
    if has_jankurai_audit_ci_lane(ctx) {
        score += 6;
        evidence.push("agent-readiness audit gate found in CI".into());
    } else {
        score -= 6;
        notes.push("CI does not run the jankurai audit".into());
    }
    if ctx
        .all_files
        .iter()
        .any(|f| f.rel_path.ends_with(".rs") && f.text.contains("unsafe"))
    {
        score += 4;
        evidence.push("unsafe usage appears to be tracked".into());
    }
    if security_text.contains("cargo audit") && security_text.contains("npm audit") {
        score += 8;
        evidence.push("Rust and npm dependency audits are operational commands".into());
    }
    if security_text.contains("gitleaks detect") {
        score += 6;
        evidence.push("secret scanning command is operational".into());
    }
    make_dim("Security and supply-chain posture", score, evidence, notes)
}
