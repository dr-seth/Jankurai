use crate::commands::context_data::RepoCatalog;
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct RegistryArgs {
    pub repo: PathBuf,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryPlan {
    pub schema_version: String,
    pub command: String,
    pub repo: String,
    pub generated_at: String,
    pub status: String,
    pub summary: String,
    pub candidate_cells: Vec<RegistryCell>,
    pub required_sources: Vec<String>,
    pub proof_lanes: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryCell {
    pub cell_id: String,
    pub owner: String,
    pub category: String,
    pub lifecycle: String,
    pub certification_status: String,
    pub source_paths: Vec<String>,
    pub proof_lanes: Vec<String>,
    pub upgrade_notes: Vec<String>,
}

pub fn run(args: RegistryArgs) -> Result<()> {
    let plan = build_registry_plan(&args.repo)?;
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

pub fn build_registry_plan(repo: &Path) -> Result<RegistryPlan> {
    let catalog = RepoCatalog::load(repo)?;
    let required_sources = vec![
        "agent/owner-map.json".to_string(),
        "agent/test-map.json".to_string(),
        "agent/generated-zones.toml".to_string(),
        "agent/proof-lanes.toml".to_string(),
        "schemas/cell-manifest.schema.json".to_string(),
        "schemas/cell-registry.schema.json".to_string(),
    ];
    let proof_lanes = if catalog.proof_lane_names().is_empty() {
        vec!["fast".to_string(), "audit".to_string()]
    } else {
        catalog.proof_lane_names()
    };
    let candidate_cells = build_candidate_cells(&catalog, &proof_lanes);
    let mut notes = vec![
        "registry output is derived from the machine-readable owner/test maps".to_string(),
        "cell entries are generated from current repo routing and proof lanes".to_string(),
    ];
    if candidate_cells.len() == 1 && candidate_cells[0].cell_id == "workspace-registry" {
        notes.push(
            "repo does not yet expose owner map data; using a generic registry placeholder"
                .to_string(),
        );
    }
    Ok(RegistryPlan {
        schema_version: "1.0.0".to_string(),
        command: "jankurai registry".to_string(),
        repo: repo.display().to_string(),
        generated_at: now_string(),
        status: "complete".to_string(),
        summary: "candidate reuse registry for certified cells".to_string(),
        candidate_cells,
        required_sources,
        proof_lanes,
        notes,
    })
}

fn build_candidate_cells(catalog: &RepoCatalog, proof_lanes: &[String]) -> Vec<RegistryCell> {
    let mut owners = BTreeSet::new();
    for owner in catalog.owners.values() {
        owners.insert(owner.clone());
    }
    let mut cells = Vec::new();
    for owner in owners {
        cells.push(RegistryCell {
            cell_id: format!("{}-cell", owner),
            owner: owner.clone(),
            category: category_for_owner(&owner).to_string(),
            lifecycle: "draft".to_string(),
            certification_status: "candidate".to_string(),
            source_paths: source_paths_for_owner(catalog, &owner),
            proof_lanes: proof_lanes.to_vec(),
            upgrade_notes: vec![
                "add local contracts, tests, and proof lanes before certification".to_string(),
            ],
        });
    }
    if cells.is_empty() {
        cells.push(RegistryCell {
            cell_id: "workspace-registry".to_string(),
            owner: "workspace".to_string(),
            category: "engineering".to_string(),
            lifecycle: "draft".to_string(),
            certification_status: "candidate".to_string(),
            source_paths: vec![
                "agent/owner-map.json".to_string(),
                "agent/test-map.json".to_string(),
                "schemas/cell-manifest.schema.json".to_string(),
            ],
            proof_lanes: proof_lanes.to_vec(),
            upgrade_notes: vec![
                "populate owner map entries to split this placeholder into reusable cells"
                    .to_string(),
            ],
        });
    }
    cells
}

fn source_paths_for_owner(catalog: &RepoCatalog, owner: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for path in catalog.prefixes_for_owner(owner) {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
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

fn render_markdown(plan: &RegistryPlan) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Registry");
    let _ = writeln!(out);
    let _ = writeln!(out, "- command: `{}`", plan.command);
    let _ = writeln!(out, "- repo: `{}`", plan.repo);
    let _ = writeln!(out, "- generated at: `{}`", plan.generated_at);
    let _ = writeln!(out, "- status: `{}`", plan.status);
    let _ = writeln!(out, "- summary: {}", plan.summary);
    let _ = writeln!(
        out,
        "- required sources: `{}`",
        join_or_none(&plan.required_sources)
    );
    let _ = writeln!(out, "- proof lanes: `{}`", join_or_none(&plan.proof_lanes));
    let _ = writeln!(out, "- notes: `{}`", join_or_none(&plan.notes));
    for cell in &plan.candidate_cells {
        let _ = writeln!(out);
        let _ = writeln!(out, "## {}", cell.cell_id);
        let _ = writeln!(out, "- owner: `{}`", cell.owner);
        let _ = writeln!(out, "- category: `{}`", cell.category);
        let _ = writeln!(out, "- lifecycle: `{}`", cell.lifecycle);
        let _ = writeln!(out, "- certification: `{}`", cell.certification_status);
        let _ = writeln!(
            out,
            "- source paths: `{}`",
            join_or_none(&cell.source_paths)
        );
        let _ = writeln!(out, "- proof lanes: `{}`", join_or_none(&cell.proof_lanes));
        let _ = writeln!(
            out,
            "- upgrade notes: `{}`",
            join_or_none(&cell.upgrade_notes)
        );
    }
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
