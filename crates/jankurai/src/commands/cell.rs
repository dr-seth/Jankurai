use crate::commands::context_data::RepoCatalog;
use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct CellArgs {
    pub repo: PathBuf,
    pub cell_id: String,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CellPlan {
    pub schema_version: String,
    pub command: String,
    pub repo: String,
    pub generated_at: String,
    pub status: String,
    pub mode: String,
    pub cell_id: String,
    pub owner: String,
    pub category: String,
    pub source_paths: Vec<String>,
    pub proof_lanes: Vec<String>,
    pub notes: Vec<String>,
}

pub fn run(args: CellArgs) -> Result<()> {
    let plan = build_cell_plan(&args.repo, &args.cell_id, "install-ready")?;
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

pub fn build_cell_plan(repo: &Path, cell_id: &str, mode: &str) -> Result<CellPlan> {
    let catalog = RepoCatalog::load(repo)?;
    let owner = owner_for_cell(&catalog, cell_id);
    let proof_lanes = if catalog.proof_lane_names().is_empty() {
        vec!["fast".to_string(), "audit".to_string()]
    } else {
        catalog.proof_lane_names()
    };
    Ok(CellPlan {
        schema_version: "1.0.0".to_string(),
        command: "jankurai cell".to_string(),
        repo: repo.display().to_string(),
        generated_at: now_string(),
        status: "complete".to_string(),
        mode: mode.to_string(),
        cell_id: cell_id.to_string(),
        owner: owner.clone(),
        category: category_for_owner(&owner).to_string(),
        source_paths: source_paths_for_owner(&catalog, &owner),
        proof_lanes,
        notes: vec![
            "cell output is generated from current ownership and proof routing".to_string(),
            "install scaffolds remain safe to review before any future write path".to_string(),
        ],
    })
}

fn owner_for_cell(catalog: &RepoCatalog, cell_id: &str) -> String {
    if let Some(owner) = cell_id.split_once('-').map(|(owner, _)| owner.to_string()) {
        if catalog.owners.values().any(|candidate| candidate == &owner) {
            return owner;
        }
    }
    "workspace".to_string()
}

fn source_paths_for_owner(catalog: &RepoCatalog, owner: &str) -> Vec<String> {
    let mut paths = catalog.prefixes_for_owner(owner);
    if paths.is_empty() {
        paths.push("crates/jankurai/src/commands/".to_string());
    }
    paths
}

fn category_for_owner(owner: &str) -> &'static str {
    match owner {
        "agent" => "agent-surface",
        "paper" => "documentation",
        "ops" => "governance",
        "standard" => "standard",
        "tools" => "tooling",
        _ => "engineering",
    }
}

fn render_markdown(plan: &CellPlan) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Cell Plan");
    let _ = writeln!(out);
    let _ = writeln!(out, "- command: `{}`", plan.command);
    let _ = writeln!(out, "- mode: `{}`", plan.mode);
    let _ = writeln!(out, "- cell: `{}`", plan.cell_id);
    let _ = writeln!(out, "- owner: `{}`", plan.owner);
    let _ = writeln!(out, "- category: `{}`", plan.category);
    let _ = writeln!(out, "- source paths: `{}`", plan.source_paths.join(", "));
    let _ = writeln!(out, "- proof lanes: `{}`", plan.proof_lanes.join(", "));
    let _ = writeln!(out, "- notes: `{}`", plan.notes.join(", "));
    out
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
