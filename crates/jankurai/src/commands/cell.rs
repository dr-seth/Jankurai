use crate::commands::cell_catalog::{
    manifest_for_cell, owner_for_cell, CellEvidence, CellManifest,
};
use crate::commands::context_data::RepoCatalog;
use crate::validation::{self, ArtifactSchema};
use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct CellArgs {
    pub repo: PathBuf,
    pub cell_id: String,
    pub mode: String,
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
    pub manifest: CellManifest,
    pub install_plan: InstallPlan,
    pub certification_evidence: Vec<CellEvidence>,
    pub proof_commands: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallPlan {
    pub dry_run: bool,
    pub planned_writes: Vec<String>,
    pub conflict_policy: String,
    pub forbidden_overwrites: Vec<String>,
}

pub fn run(args: CellArgs) -> Result<()> {
    let plan = build_cell_plan(&args.repo, &args.cell_id, &args.mode)?;
    validation::validate_serializable(&args.repo, ArtifactSchema::CellManifest, &plan.manifest)?;
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
    let manifest = manifest_for_cell(repo, &catalog, cell_id);
    let owner = manifest
        .source_paths
        .first()
        .and_then(|path| catalog.owner_for_path(path))
        .map(|owner| owner.to_string())
        .unwrap_or_else(|| owner_for_cell(&catalog, cell_id));
    let install_plan = build_install_plan(&manifest);
    let certification_evidence = if mode == "prove" {
        manifest.certification_evidence.clone()
    } else {
        Vec::new()
    };
    let proof_commands = if mode == "prove" {
        manifest.proof_commands.clone()
    } else {
        Vec::new()
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
        category: manifest.category.clone(),
        source_paths: manifest.source_paths.clone(),
        proof_lanes: manifest.proof_lanes.clone(),
        install_plan,
        certification_evidence,
        proof_commands,
        manifest,
        notes: vec![
            "cell output is generated from current ownership and proof routing".to_string(),
            "install-ready mode emits a dry-run plan only and never writes files".to_string(),
            "prove mode emits evidence and proof commands without executing them".to_string(),
        ],
    })
}

fn build_install_plan(manifest: &CellManifest) -> InstallPlan {
    let planned_writes = manifest
        .source_paths
        .iter()
        .chain(manifest.contract_paths.iter())
        .chain(manifest.migration_paths.iter())
        .chain(manifest.ui_routes.iter())
        .chain(manifest.docs.iter())
        .cloned()
        .collect::<Vec<_>>();
    InstallPlan {
        dry_run: true,
        forbidden_overwrites: planned_writes.clone(),
        planned_writes,
        conflict_policy: manifest.conflict_policy.clone(),
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
    let _ = writeln!(
        out,
        "- certification: `{}`",
        plan.manifest.certification_status
    );
    let _ = writeln!(
        out,
        "- install strategy: `{}`",
        plan.manifest.install_strategy
    );
    let _ = writeln!(
        out,
        "- conflict policy: `{}`",
        plan.install_plan.conflict_policy
    );
    let _ = writeln!(out, "- dry run: `{}`", plan.install_plan.dry_run);
    let _ = writeln!(out, "- source paths: `{}`", plan.source_paths.join(", "));
    let _ = writeln!(out, "- proof lanes: `{}`", plan.proof_lanes.join(", "));
    let _ = writeln!(
        out,
        "- proof commands: `{}`",
        plan.proof_commands.join(", ")
    );
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
