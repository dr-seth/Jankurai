use crate::model::{VibeCoverageGap, VibeCoverageSummary, PAPER_EDITION, SCHEMA_VERSION};
use crate::validation::{self, ArtifactSchema};
use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct VibeCoverageArgs {
    pub repo: PathBuf,
    pub source: String,
    pub tips: String,
    pub json: Option<String>,
    pub md: Option<String>,
    pub tex: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VibeValidateArgs {
    pub repo: PathBuf,
    pub source: String,
    pub tips: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoverageSource {
    pub schema_version: String,
    pub release: String,
    pub paper_edition: String,
    pub source_count: usize,
    pub issues: Vec<CoverageIssue>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoverageIssue {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub tlr: String,
    pub category: String,
    pub description: String,
    pub recommended_control: String,
    pub source_refs: Vec<String>,
    pub coverage: String,
    pub coverage_reason: String,
    pub rule_ids: Vec<String>,
    pub tool_ids: Vec<String>,
    pub proof_lanes: Vec<String>,
    pub artifacts: Vec<String>,
    pub gap: String,
    pub next_action: String,
    pub owner: String,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VibeCoverageReport {
    pub schema_version: String,
    pub schema_url: String,
    pub command: String,
    pub source_path: String,
    pub tips_path: String,
    pub release: String,
    pub paper_edition: String,
    pub issue_count: usize,
    pub source_ref_count: usize,
    pub expected_source_row_count: usize,
    pub unmapped_source_rows: usize,
    pub duplicate_source_refs: Vec<String>,
    pub missing_source_refs: Vec<String>,
    pub unexpected_source_refs: Vec<String>,
    pub coverage_counts: BTreeMap<String, usize>,
    pub tlr_counts: BTreeMap<String, usize>,
    pub rule_id_counts: BTreeMap<String, usize>,
    pub tool_id_counts: BTreeMap<String, usize>,
    pub priority_counts: BTreeMap<String, usize>,
    pub top_gaps: Vec<VibeCoverageGap>,
    pub issues: Vec<CoverageIssue>,
}

pub fn run_validate(args: VibeValidateArgs) -> Result<()> {
    let repo = args.repo.canonicalize()?;
    let report = build_report(&repo, &args.source, &args.tips)?;
    if report.unmapped_source_rows != 0
        || !report.duplicate_source_refs.is_empty()
        || !report.unexpected_source_refs.is_empty()
    {
        bail!(
            "vibe coverage is incomplete: unmapped={} duplicates={} unexpected={}",
            report.unmapped_source_rows,
            report.duplicate_source_refs.len(),
            report.unexpected_source_refs.len()
        );
    }
    println!(
        "vibe coverage ok: issues={} refs={} absolute={} partial={} none={}",
        report.issue_count,
        report.source_ref_count,
        report.coverage_counts.get("absolute").copied().unwrap_or(0),
        report.coverage_counts.get("partial").copied().unwrap_or(0),
        report.coverage_counts.get("none").copied().unwrap_or(0)
    );
    Ok(())
}

pub fn run_coverage(args: VibeCoverageArgs) -> Result<()> {
    let repo = args.repo.canonicalize()?;
    let report = build_report(&repo, &args.source, &args.tips)?;
    validation::validate_serializable(&repo, ArtifactSchema::VibeCoverageReport, &report)?;
    if let Some(path) = args.json.as_deref() {
        validation::write_json(&repo, ArtifactSchema::VibeCoverageReport, path, &report)?;
    }
    if let Some(path) = args.md.as_deref() {
        crate::render::write_markdown(path, &render_markdown(&report))?;
    }
    if let Some(path) = args.tex.as_deref() {
        crate::render::write_markdown(path, &render_tex(&report))?;
    }
    Ok(())
}

pub fn audit_summary(root: &Path) -> Option<VibeCoverageSummary> {
    build_report(root, "agent/vibe-coverage.toml", "tips/vibe_coding")
        .ok()
        .map(|report| report.summary())
}

pub fn build_report(repo: &Path, source: &str, tips: &str) -> Result<VibeCoverageReport> {
    let source_path = repo.join(source);
    let source_text = fs::read_to_string(&source_path).with_context(|| format!("read {source}"))?;
    let source_value = validation::validate_vibe_coverage_source_toml_text(repo, &source_text)?;
    validation::validate_value(repo, ArtifactSchema::VibeCoverageSource, &source_value)?;
    let parsed: CoverageSource =
        toml::from_str(&source_text).with_context(|| format!("parse {source}"))?;
    validate_source_shape(&parsed)?;

    let expected = parse_tip_refs(&repo.join(tips))?;
    let expected_set = expected.iter().cloned().collect::<BTreeSet<_>>();
    let mut seen = BTreeMap::<String, usize>::new();
    for issue in &parsed.issues {
        for source_ref in &issue.source_refs {
            *seen.entry(source_ref.clone()).or_insert(0) += 1;
        }
    }
    let seen_set = seen.keys().cloned().collect::<BTreeSet<_>>();
    let duplicate_source_refs = seen
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(source_ref, _)| source_ref.clone())
        .collect::<Vec<_>>();
    let missing_source_refs = expected_set
        .difference(&seen_set)
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_source_refs = seen_set
        .difference(&expected_set)
        .cloned()
        .collect::<Vec<_>>();

    let mut coverage_counts = BTreeMap::new();
    let mut tlr_counts = BTreeMap::new();
    let mut rule_id_counts = BTreeMap::new();
    let mut tool_id_counts = BTreeMap::new();
    let mut priority_counts = BTreeMap::new();
    for issue in &parsed.issues {
        incr(&mut coverage_counts, &issue.coverage);
        incr(&mut tlr_counts, &issue.tlr);
        incr(&mut priority_counts, &issue.priority);
        for rule in &issue.rule_ids {
            incr(&mut rule_id_counts, rule);
        }
        for tool in &issue.tool_ids {
            incr(&mut tool_id_counts, tool);
        }
    }

    let mut top_gaps = parsed
        .issues
        .iter()
        .filter(|issue| issue.coverage != "absolute")
        .map(|issue| VibeCoverageGap {
            id: issue.id.clone(),
            name: issue.name.clone(),
            coverage: issue.coverage.clone(),
            priority: issue.priority.clone(),
            gap: issue.gap.clone(),
            next_action: issue.next_action.clone(),
        })
        .collect::<Vec<_>>();
    top_gaps.sort_by(|a, b| {
        coverage_rank(&a.coverage)
            .cmp(&coverage_rank(&b.coverage))
            .then(a.priority.cmp(&b.priority))
            .then(a.id.cmp(&b.id))
    });
    top_gaps.truncate(12);

    Ok(VibeCoverageReport {
        schema_version: SCHEMA_VERSION.into(),
        schema_url: "schemas/vibe-coverage-report.schema.json".into(),
        command: "jankurai vibe coverage".into(),
        source_path: source.into(),
        tips_path: tips.into(),
        release: parsed.release,
        paper_edition: parsed.paper_edition,
        issue_count: parsed.issues.len(),
        source_ref_count: seen.values().sum(),
        expected_source_row_count: expected.len(),
        unmapped_source_rows: missing_source_refs.len(),
        duplicate_source_refs,
        missing_source_refs,
        unexpected_source_refs,
        coverage_counts,
        tlr_counts,
        rule_id_counts,
        tool_id_counts,
        priority_counts,
        top_gaps,
        issues: parsed.issues,
    })
}

impl VibeCoverageReport {
    pub fn summary(&self) -> VibeCoverageSummary {
        VibeCoverageSummary {
            source_path: self.source_path.clone(),
            issue_count: self.issue_count,
            source_ref_count: self.source_ref_count,
            unmapped_source_rows: self.unmapped_source_rows,
            coverage_counts: self.coverage_counts.clone(),
            tlr_counts: self.tlr_counts.clone(),
            priority_counts: self.priority_counts.clone(),
            top_gaps: self.top_gaps.iter().take(6).cloned().collect(),
        }
    }
}

fn validate_source_shape(source: &CoverageSource) -> Result<()> {
    if source.schema_version != "1.4.0" {
        bail!("agent/vibe-coverage.toml schema_version must be 1.4.0");
    }
    if source.paper_edition != PAPER_EDITION {
        bail!(
            "agent/vibe-coverage.toml paper_edition must match CLI paper edition {PAPER_EDITION}"
        );
    }
    if source.source_count != source.issues.len() {
        bail!(
            "source_count {} does not match issue count {}",
            source.source_count,
            source.issues.len()
        );
    }
    let mut ids = BTreeSet::new();
    let source_ref_re = Regex::new(r"^tip[1-5]:[1-9][0-9]*$")?;
    for issue in &source.issues {
        if !ids.insert(issue.id.as_str()) {
            bail!("duplicate vibe issue id {}", issue.id);
        }
        if !matches!(issue.coverage.as_str(), "absolute" | "partial" | "none") {
            bail!("{} has invalid coverage {}", issue.id, issue.coverage);
        }
        if issue.source_refs.is_empty() {
            bail!("{} has no source_refs", issue.id);
        }
        for source_ref in &issue.source_refs {
            if !source_ref_re.is_match(source_ref) {
                bail!("{} has invalid source_ref {}", issue.id, source_ref);
            }
        }
        for (label, values) in [
            ("aliases", &issue.aliases),
            ("rule_ids", &issue.rule_ids),
            ("tool_ids", &issue.tool_ids),
            ("proof_lanes", &issue.proof_lanes),
            ("artifacts", &issue.artifacts),
        ] {
            if values.is_empty() {
                bail!("{} has empty {}", issue.id, label);
            }
        }
    }
    Ok(())
}

fn parse_tip_refs(tips_dir: &Path) -> Result<Vec<String>> {
    let row_re = Regex::new(r"^\|\s*(\d+)\s*\|")?;
    let mut refs = Vec::new();
    for tip in 1..=5 {
        let file = tips_dir.join(format!("tip{tip}.txt"));
        let text = fs::read_to_string(&file).with_context(|| format!("read {}", file.display()))?;
        for line in text.lines() {
            if let Some(caps) = row_re.captures(line) {
                refs.push(format!("tip{tip}:{}", &caps[1]));
            }
        }
    }
    Ok(refs)
}

fn incr(map: &mut BTreeMap<String, usize>, key: &str) {
    *map.entry(key.to_string()).or_insert(0) += 1;
}

fn coverage_rank(coverage: &str) -> u8 {
    match coverage {
        "none" => 0,
        "partial" => 1,
        "absolute" => 2,
        _ => 3,
    }
}

fn render_markdown(report: &VibeCoverageReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# Vibe Coding Coverage");
    let _ = writeln!(out);
    let _ = writeln!(out, "- Source: `{}`", report.source_path);
    let _ = writeln!(out, "- Tips: `{}`", report.tips_path);
    let _ = writeln!(out, "- Release: `{}`", report.release);
    let _ = writeln!(out, "- Paper edition: `{}`", report.paper_edition);
    let _ = writeln!(out, "- Issues: `{}`", report.issue_count);
    let _ = writeln!(out, "- Source refs: `{}`", report.source_ref_count);
    let _ = writeln!(
        out,
        "- Unmapped source rows: `{}`",
        report.unmapped_source_rows
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "## Coverage Counts");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Coverage | Count |");
    let _ = writeln!(out, "| --- | ---: |");
    for key in ["absolute", "partial", "none"] {
        let _ = writeln!(
            out,
            "| `{}` | {} |",
            key,
            report.coverage_counts.get(key).copied().unwrap_or(0)
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "## Top Gaps");
    let _ = writeln!(out);
    let _ = writeln!(out, "| ID | Coverage | Priority | Gap | Next action |");
    let _ = writeln!(out, "| --- | --- | --- | --- | --- |");
    for gap in &report.top_gaps {
        let _ = writeln!(
            out,
            "| `{}` | `{}` | `{}` | {} | {} |",
            gap.id,
            gap.coverage,
            gap.priority,
            md_escape(&gap.gap),
            md_escape(&gap.next_action)
        );
    }
    out
}

fn render_tex(report: &VibeCoverageReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "% Generated by: jankurai vibe coverage {}",
        report.release
    );
    let _ = writeln!(out, "% Source: {}", report.source_path);
    let _ = writeln!(
        out,
        "% Command: cargo run -p jankurai -- vibe coverage --source {} --tips {} --tex paper/tex/generated/vibe_coverage_table.tex",
        report.source_path, report.tips_path
    );
    let _ = writeln!(out, "% DO NOT EDIT BY HAND.");
    let _ = writeln!(out);
    let _ = writeln!(out, "\\begin{{longtable}}{{p{{0.15\\textwidth}}p{{0.19\\textwidth}}p{{0.12\\textwidth}}p{{0.13\\textwidth}}p{{0.33\\textwidth}}}}");
    let _ = writeln!(
        out,
        "\\caption{{Vibe-coding source-row coverage. Green = absolute, yellow = partial, red = none.}}\\\\"
    );
    let _ = writeln!(out, "\\toprule");
    let _ = writeln!(out, "\\textbf{{Source}} & \\textbf{{Issue}} & \\textbf{{Coverage}} & \\textbf{{Rule}} & \\textbf{{Gap / next action}} \\\\");
    let _ = writeln!(out, "\\midrule");
    let _ = writeln!(out, "\\endfirsthead");
    let _ = writeln!(out, "\\toprule");
    let _ = writeln!(out, "\\textbf{{Source}} & \\textbf{{Issue}} & \\textbf{{Coverage}} & \\textbf{{Rule}} & \\textbf{{Gap / next action}} \\\\");
    let _ = writeln!(out, "\\midrule");
    let _ = writeln!(out, "\\endhead");
    for issue in &report.issues {
        let color = match issue.coverage.as_str() {
            "absolute" => "green!18",
            "partial" => "yellow!28",
            _ => "red!18",
        };
        let rule = issue.rule_ids.join(", ");
        let source_refs = issue.source_refs.join(", ");
        let gap = if issue.coverage == "absolute" {
            issue.coverage_reason.clone()
        } else {
            format!("{}; next: {}", issue.gap, issue.next_action)
        };
        let _ = writeln!(
            out,
            "\\rowcolor{{{}}}\\texttt{{{}}} & {} & \\textbf{{{}}} & \\texttt{{{}}} & {} \\\\",
            color,
            tex_escape(&source_refs),
            tex_escape(&issue.name),
            tex_escape(&issue.coverage),
            tex_escape(&rule),
            tex_escape(&gap)
        );
    }
    let _ = writeln!(out, "\\bottomrule");
    let _ = writeln!(out, "\\end{{longtable}}");
    out
}

fn md_escape(value: &str) -> String {
    value.replace('|', "\\|")
}

fn tex_escape(value: &str) -> String {
    value
        .replace('\\', "\\textbackslash{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
}
