use crate::audit::run_audit;
use crate::commands::migrate;
use crate::init;
use crate::validation::{self, ArtifactSchema};
use anyhow::{bail, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_OUT: &str = "target/jankurai/adoption-plan.json";
pub const DEFAULT_MD: &str = "target/jankurai/adoption-plan.md";

#[derive(Debug, Clone)]
pub struct AdoptArgs {
    pub repo: PathBuf,
    pub profile: String,
    pub mode: String,
    pub out: String,
    pub md: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdoptionPlan {
    pub schema_version: String,
    pub command: String,
    pub status: String,
    pub generated_at: String,
    pub source_root: String,
    pub mode: String,
    pub requested_profile: String,
    pub recommended_profile: String,
    pub risk_tier: String,
    pub detected_surfaces: Vec<String>,
    pub source_stack: String,
    pub target_stack: String,
    pub liability_score: u32,
    pub audit_score: Option<i32>,
    pub safe_commands: Vec<String>,
    pub stop_conditions: Vec<String>,
    pub next_milestones: Vec<String>,
    pub artifacts: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn run(args: AdoptArgs) -> Result<()> {
    let mut plan = build_adoption_plan(&args.repo, &args.profile, &args.mode)?;
    plan.artifacts = vec![args.out.clone(), args.md.clone()];
    validation::write_json(&args.repo, ArtifactSchema::AdoptionPlan, &args.out, &plan)?;
    crate::render::write_markdown(&args.md, &render_markdown(&plan))?;
    eprintln!("wrote {} and {}", args.out, args.md);
    Ok(())
}

pub fn build_adoption_plan(
    repo: &Path,
    requested_profile: &str,
    mode: &str,
) -> Result<AdoptionPlan> {
    if !matches!(mode, "observe" | "advisory" | "ratchet") {
        bail!("unknown adoption mode `{mode}`; expected observe, advisory, or ratchet");
    }

    let detected_surfaces = init::detect::detect_surfaces(repo);
    let migration_report = migrate::build_migration_report(repo, "rust-ts-postgres")?;
    let recommended_profile =
        recommend_profile(requested_profile, &detected_surfaces, &migration_report);
    let audit_score = read_existing_audit_score(repo)
        .or_else(|| run_audit(repo, &[]).ok().map(|report| report.score));
    let risk_tier = risk_tier(migration_report.liability_score);
    let mut warnings = Vec::new();
    if requested_profile != "auto" && requested_profile != recommended_profile {
        warnings.push(format!(
            "requested profile `{requested_profile}` overrides recommended `{recommended_profile}`"
        ));
    }
    if audit_score.is_none() {
        warnings.push(
            "audit score unavailable; run the no-write audit command before ratcheting".into(),
        );
    }
    if recommended_profile == "migration-target" {
        warnings.push(
            "repo appears far enough from the target stack to route through migration planning"
                .into(),
        );
    }
    let safe_commands = safe_commands(mode, &recommended_profile);

    Ok(AdoptionPlan {
        schema_version: "1.0.0".into(),
        command: "jankurai adopt".into(),
        status: "complete".into(),
        generated_at: now_string(),
        source_root: repo.display().to_string(),
        mode: mode.to_string(),
        requested_profile: requested_profile.to_string(),
        recommended_profile,
        risk_tier: risk_tier.to_string(),
        detected_surfaces,
        source_stack: migration_report.source_stack,
        target_stack: migration_report.target_stack,
        liability_score: migration_report.liability_score,
        audit_score,
        safe_commands,
        stop_conditions: stop_conditions(mode),
        next_milestones: next_milestones(mode, risk_tier),
        artifacts: vec![DEFAULT_OUT.into(), DEFAULT_MD.into()],
        warnings,
    })
}

fn recommend_profile(
    requested_profile: &str,
    surfaces: &[String],
    report: &migrate::MigrationReport,
) -> String {
    if requested_profile != "auto" {
        return requested_profile.to_string();
    }
    let has_rust = report
        .inventory
        .languages
        .iter()
        .any(|item| item.name == "rust");
    let has_node = report
        .inventory
        .languages
        .iter()
        .any(|item| item.name == "typescript");
    let has_db =
        !report.inventory.db_clients.is_empty() || surfaces.iter().any(|s| s == "postgres");
    let has_web = surfaces.iter().any(|s| s == "vite-react")
        || report
            .inventory
            .frameworks
            .iter()
            .any(|item| matches!(item.name.as_str(), "react" | "vue" | "angular" | "svelte"));
    let has_api = !report.inventory.api_surfaces.is_empty();

    if report.source_stack != "unknown" && !has_rust {
        return "migration-target".into();
    }
    if has_rust && (has_node || has_web || has_db) {
        return "rust-ts-postgres".into();
    }
    if has_rust && has_api {
        return "rust-api".into();
    }
    if has_web {
        return "react-web".into();
    }
    if report.source_stack == "unknown" {
        return "rust-ts-postgres".into();
    }
    "migration-target".into()
}

fn safe_commands(mode: &str, recommended_profile: &str) -> Vec<String> {
    let mut commands = vec![
        "jankurai audit . --mode advisory --json target/jankurai/repo-score.json --md target/jankurai/repo-score.md".into(),
        "jankurai adopt . --mode observe --out target/jankurai/adoption-plan.json --md target/jankurai/adoption-plan.md".into(),
        format!("jankurai init . --profile {recommended_profile} --dry-run --plan-json target/jankurai/init-plan.json"),
        "jankurai ci install . --github --mode observe --dry-run".into(),
    ];
    if mode == "ratchet" {
        commands.push(
            "jankurai audit . --mode ratchet --baseline target/jankurai/baseline-score.json --json target/jankurai/repo-score.json --md target/jankurai/repo-score.md".into(),
        );
    }
    commands
}

fn stop_conditions(mode: &str) -> Vec<String> {
    let mut conditions = vec![
        "a generated command requires this source workspace instead of the installed `jankurai` binary".into(),
        "dry-run output plans to overwrite existing AGENTS.md, Justfile, workflows, or docs without a merge action".into(),
        "the plan claims production compliance before audit evidence and proof lanes exist".into(),
    ];
    if mode != "ratchet" {
        conditions.push("CI enforces a score floor before a baseline has been accepted".into());
    }
    conditions
}

fn next_milestones(mode: &str, risk_tier: &str) -> Vec<String> {
    let mut milestones = vec![
        "review adoption plan and liability evidence".into(),
        "run init dry-run and inspect planned control-plane files".into(),
        "install observe-mode CI only after preserving existing workflows".into(),
    ];
    if risk_tier == "high" {
        milestones
            .push("route through migration-target profile and slice the legacy migration".into());
    }
    if mode == "ratchet" {
        milestones
            .push("commit an accepted baseline before enforcing score regression checks".into());
    } else {
        milestones.push("stay advisory until the first accepted baseline exists".into());
    }
    milestones
}

fn risk_tier(score: u32) -> &'static str {
    match score {
        0..=39 => "low",
        40..=59 => "medium",
        _ => "high",
    }
}

fn read_existing_audit_score(repo: &Path) -> Option<i32> {
    let path = repo.join("agent/repo-score.json");
    let text = fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    value.get("score")?.as_i64().map(|score| score as i32)
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

fn render_markdown(plan: &AdoptionPlan) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Adoption Plan");
    let _ = writeln!(out);
    let _ = writeln!(out, "- mode: `{}`", plan.mode);
    let _ = writeln!(out, "- recommended profile: `{}`", plan.recommended_profile);
    let _ = writeln!(out, "- risk tier: `{}`", plan.risk_tier);
    let _ = writeln!(out, "- source stack: `{}`", plan.source_stack);
    let _ = writeln!(out, "- liability score: `{}`", plan.liability_score);
    if let Some(score) = plan.audit_score {
        let _ = writeln!(out, "- audit score: `{score}`");
    } else {
        let _ = writeln!(out, "- audit score: unavailable");
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "## Safe Commands");
    for command in &plan.safe_commands {
        let _ = writeln!(out, "- `{command}`");
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "## Stop Conditions");
    for condition in &plan.stop_conditions {
        let _ = writeln!(out, "- {condition}");
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "## Next Milestones");
    for milestone in &plan.next_milestones {
        let _ = writeln!(out, "- {milestone}");
    }
    if !plan.warnings.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "## Warnings");
        for warning in &plan.warnings {
            let _ = writeln!(out, "- {warning}");
        }
    }
    out
}
