use crate::commands::context_data::RepoCatalog;
use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct MigrateArgs {
    pub repo: PathBuf,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationPlan {
    pub schema_version: String,
    pub command: String,
    pub repo: String,
    pub generated_at: String,
    pub status: String,
    pub target: String,
    pub inventory: Vec<String>,
    pub slice_plan: Vec<String>,
    pub equivalence_proof: Vec<String>,
    pub rollback_notes: Vec<String>,
}

pub fn run(args: MigrateArgs) -> Result<()> {
    let plan = build_migration_plan(&args.repo, "legacy-inventory")?;
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

pub fn build_migration_plan(repo: &Path, target: &str) -> Result<MigrationPlan> {
    let catalog = RepoCatalog::load(repo)?;
    Ok(MigrationPlan {
        schema_version: "1.0.0".to_string(),
        command: "humanlint migrate".to_string(),
        repo: repo.display().to_string(),
        generated_at: now_string(),
        status: "complete".to_string(),
        target: target.to_string(),
        inventory: build_inventory(&catalog),
        slice_plan: vec![
            "inventory legacy surfaces".to_string(),
            "classify liabilities and cutover slices".to_string(),
            "extract contracts before code movement".to_string(),
        ],
        equivalence_proof: vec![
            "compare source and target contracts".to_string(),
            "preserve rollback guidance for each slice".to_string(),
        ],
        rollback_notes: vec![
            "migration planning is derived from repo inventory and audit evidence".to_string(),
        ],
    })
}

fn build_inventory(catalog: &RepoCatalog) -> Vec<String> {
    let mut inventory = vec![
        "agent/owner-map.json".to_string(),
        "agent/test-map.json".to_string(),
        "schemas/repo-score.schema.json".to_string(),
    ];
    for path in catalog.prefixes_for_owner("agent") {
        if !inventory.contains(&path) {
            inventory.push(path);
        }
    }
    inventory
}

fn render_markdown(plan: &MigrationPlan) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# humanlint Migration Plan");
    let _ = writeln!(out);
    let _ = writeln!(out, "- command: `{}`", plan.command);
    let _ = writeln!(out, "- target: `{}`", plan.target);
    let _ = writeln!(out, "- status: `{}`", plan.status);
    let _ = writeln!(out, "- inventory: `{}`", plan.inventory.join(", "));
    let _ = writeln!(out, "- slice plan: `{}`", plan.slice_plan.join(", "));
    let _ = writeln!(
        out,
        "- equivalence proof: `{}`",
        plan.equivalence_proof.join(", ")
    );
    let _ = writeln!(out, "- rollback: `{}`", plan.rollback_notes.join(", "));
    out
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
