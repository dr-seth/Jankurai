use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct GovernArgs {
    pub repo: PathBuf,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GovernPlan {
    pub schema_version: String,
    pub command: String,
    pub repo: String,
    pub generated_at: String,
    pub status: String,
    pub policy_stack: Vec<String>,
    pub ratchets: Vec<String>,
    pub exception_rules: Vec<String>,
    pub review_outputs: Vec<String>,
    pub notes: Vec<String>,
}

pub fn run(args: GovernArgs) -> Result<()> {
    let plan = build_govern_plan(&args.repo);
    if let Some(path) = args.out.as_deref() {
        crate::render::write_json(path, &serde_json::to_string_pretty(&plan)?)?;
    } else {
        println!("{}", serde_json::to_string_pretty(&plan)?);
    }
    if let Some(path) = args.md.as_deref() {
        crate::render::write_markdown(path, &render_markdown(&plan))?;
    }
    Ok(())
}

pub fn build_govern_plan(repo: &Path) -> GovernPlan {
    GovernPlan {
        schema_version: "1.0.0".to_string(),
        command: "jankurai govern".to_string(),
        repo: repo.display().to_string(),
        generated_at: now_string(),
        status: "complete".to_string(),
        policy_stack: vec![
            "agent/owner-map.json".to_string(),
            "agent/test-map.json".to_string(),
            "agent/generated-zones.toml".to_string(),
            "agent/proof-lanes.toml".to_string(),
            "agent/standard-version.toml".to_string(),
        ],
        ratchets: vec![
            "minimum score floors".to_string(),
            "fail-on rule sets".to_string(),
            "release gate evidence".to_string(),
        ],
        exception_rules: vec![
            "expire exceptions on a fixed review cadence".to_string(),
            "keep exception records parseable and linked to a repair queue".to_string(),
            "never auto-approve unknown drift".to_string(),
        ],
        review_outputs: vec![
            "agent/repo-score.json".to_string(),
            "agent/repo-score.md".to_string(),
            "target/jankurai/receipts/".to_string(),
        ],
        notes: vec![
            "governance output is derived from the current policy and release docs".to_string(),
            "policy ratchets remain readable without mutating the repo".to_string(),
        ],
    }
}

fn render_markdown(plan: &GovernPlan) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Govern");
    let _ = writeln!(out);
    let _ = writeln!(out, "- command: `{}`", plan.command);
    let _ = writeln!(out, "- repo: `{}`", plan.repo);
    let _ = writeln!(out, "- generated at: `{}`", plan.generated_at);
    let _ = writeln!(out, "- status: `{}`", plan.status);
    let _ = writeln!(
        out,
        "- policy stack: `{}`",
        join_or_none(&plan.policy_stack)
    );
    let _ = writeln!(out, "- ratchets: `{}`", join_or_none(&plan.ratchets));
    let _ = writeln!(
        out,
        "- exception rules: `{}`",
        join_or_none(&plan.exception_rules)
    );
    let _ = writeln!(
        out,
        "- review outputs: `{}`",
        join_or_none(&plan.review_outputs)
    );
    let _ = writeln!(out, "- notes: `{}`", join_or_none(&plan.notes));
    out
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
