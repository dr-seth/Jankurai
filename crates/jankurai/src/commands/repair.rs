use crate::commands::repair_plan::RepairPlan;
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct RepairArgs {
    pub repo: PathBuf,
    pub plan: String,
    pub dry_run: bool,
    pub auto_pr: bool,
    pub max_risk: String,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepairRun {
    pub schema_version: String,
    pub repo: String,
    pub plan: String,
    pub generated_at: String,
    pub status: String,
    pub dry_run: bool,
    pub auto_pr_requested: bool,
    pub max_risk: String,
    pub planned_packets: usize,
    pub proof_lanes: Vec<String>,
    pub notes: Vec<String>,
}

pub fn run(args: RepairArgs) -> Result<()> {
    if !args.dry_run {
        bail!("only `--dry-run` repair execution is supported in this workspace");
    }
    let plan_text = fs::read_to_string(&args.plan)
        .with_context(|| format!("read repair plan {}", args.plan))?;
    let plan: RepairPlan = serde_json::from_str(&plan_text)
        .with_context(|| format!("parse repair plan {}", args.plan))?;
    let run = RepairRun {
        schema_version: "1.0.0".to_string(),
        repo: args.repo.display().to_string(),
        plan: args.plan.clone(),
        generated_at: now_string(),
        status: "complete".to_string(),
        dry_run: args.dry_run,
        auto_pr_requested: args.auto_pr,
        max_risk: args.max_risk,
        planned_packets: plan.packets.len(),
        proof_lanes: plan
            .packets
            .iter()
            .flat_map(|packet| packet.required_proof.clone())
            .collect(),
        notes: vec![
            "repair execution is intentionally dry-run only in this workspace".to_string(),
            "the plan can be reviewed without mutating files".to_string(),
        ],
    };
    if let Some(path) = args.out.as_deref() {
        crate::render::write_json(path, &serde_json::to_string_pretty(&run)?)?;
    } else {
        println!("{}", serde_json::to_string_pretty(&run)?);
    }
    if let Some(path) = args.md.as_deref() {
        crate::render::write_markdown(path, &render_markdown(&run))?;
    }
    Ok(())
}

fn render_markdown(run: &RepairRun) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Repair Run");
    let _ = writeln!(out);
    let _ = writeln!(out, "- repo: `{}`", run.repo);
    let _ = writeln!(out, "- plan: `{}`", run.plan);
    let _ = writeln!(out, "- status: `{}`", run.status);
    let _ = writeln!(out, "- dry run: `{}`", run.dry_run);
    let _ = writeln!(out, "- auto-pr requested: `{}`", run.auto_pr_requested);
    let _ = writeln!(out, "- max risk: `{}`", run.max_risk);
    let _ = writeln!(out, "- planned packets: `{}`", run.planned_packets);
    let _ = writeln!(out, "- proof lanes: `{}`", run.proof_lanes.join(", "));
    let _ = writeln!(out, "- notes: `{}`", run.notes.join(", "));
    out
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
