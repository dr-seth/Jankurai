use crate::commands::context_data::RepoCatalog;
use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct BenchArgs {
    pub repo: PathBuf,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchPlan {
    pub schema_version: String,
    pub command: String,
    pub repo: String,
    pub generated_at: String,
    pub status: String,
    pub suite: String,
    pub corpus: Vec<String>,
    pub expected_findings: Vec<String>,
    pub proof_lanes: Vec<String>,
    pub notes: Vec<String>,
}

pub fn run(args: BenchArgs) -> Result<()> {
    let plan = build_bench_plan(&args.repo, "smoke")?;
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

pub fn build_bench_plan(repo: &Path, suite: &str) -> Result<BenchPlan> {
    let catalog = RepoCatalog::load(repo)?;
    let proof_lanes = if catalog.proof_lane_names().is_empty() {
        vec!["fast".to_string(), "audit".to_string()]
    } else {
        catalog.proof_lane_names()
    };
    Ok(BenchPlan {
        schema_version: "1.0.0".to_string(),
        command: "jankurai bench".to_string(),
        repo: repo.display().to_string(),
        generated_at: now_string(),
        status: "complete".to_string(),
        suite: suite.to_string(),
        corpus: build_corpus(&catalog, suite),
        expected_findings: vec![
            "capture smoke thresholds before any public badge or certification is emitted"
                .to_string(),
            "keep benchmark fixtures narrow and reproducible".to_string(),
        ],
        proof_lanes,
        notes: vec![
            "benchmark output is derived from current repo surfaces".to_string(),
            "the smoke suite is ready for future fixture expansion".to_string(),
        ],
    })
}

fn build_corpus(catalog: &RepoCatalog, suite: &str) -> Vec<String> {
    let mut corpus = vec![
        "agent/repo-score.json".to_string(),
        "agent/repo-score.md".to_string(),
    ];
    if suite.contains("smoke") {
        corpus.push("target/jankurai/repair-queue.jsonl".to_string());
    }
    for owner in catalog.owners.values() {
        let path = format!("owner:{owner}");
        if !corpus.contains(&path) {
            corpus.push(path);
        }
    }
    corpus
}

fn render_markdown(plan: &BenchPlan) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Bench Plan");
    let _ = writeln!(out);
    let _ = writeln!(out, "- command: `{}`", plan.command);
    let _ = writeln!(out, "- suite: `{}`", plan.suite);
    let _ = writeln!(out, "- status: `{}`", plan.status);
    let _ = writeln!(out, "- corpus: `{}`", plan.corpus.join(", "));
    let _ = writeln!(
        out,
        "- expected findings: `{}`",
        plan.expected_findings.join(", ")
    );
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
