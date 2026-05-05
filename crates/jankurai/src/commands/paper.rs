use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::PathBuf;

pub struct PublicRepoScoresArgs {
    pub source: PathBuf,
    pub out: PathBuf,
}

const REQUIRED_TOP_LEVEL_KEYS: &[&str] = &[
    "run_root",
    "generated_at",
    "jankurai_version",
    "repo_count",
    "successful",
    "failed",
    "rows",
];

const REQUIRED_ROW_KEYS: &[&str] = &[
    "rank",
    "repo",
    "score",
    "finding_count",
    "hard_findings",
    "status",
    "weak_dimensions",
];

pub fn run_public_repo_scores(args: PublicRepoScoresArgs) -> Result<()> {
    let data = load_source(&args.source)?;
    let output = render(&data, &args.source, &args.out)?;
    if let Some(parent) = args.out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&args.out, output)?;
    Ok(())
}

fn load_source(path: &PathBuf) -> Result<Value> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let data: Value =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    let Some(object) = data.as_object() else {
        bail!("{} must contain a JSON object", path.display());
    };
    for key in REQUIRED_TOP_LEVEL_KEYS {
        if !object.contains_key(*key) {
            bail!("{} is missing top-level key `{}`", path.display(), key);
        }
    }
    let rows = rows(&data)?;
    if rows.len() as u64 != integer(&data["repo_count"])? {
        bail!(
            "{} repo_count={} but rows={}",
            path.display(),
            integer(&data["repo_count"])?,
            rows.len()
        );
    }
    for (idx, row) in rows.iter().enumerate() {
        let Some(row_object) = row.as_object() else {
            bail!("{} row {} must be an object", path.display(), idx + 1);
        };
        for key in REQUIRED_ROW_KEYS {
            if !row_object.contains_key(*key) {
                bail!(
                    "{} row {} is missing key `{}`",
                    path.display(),
                    idx + 1,
                    key
                );
            }
        }
        if !row["weak_dimensions"].is_array() {
            bail!(
                "{} row {} weak_dimensions must be an array",
                path.display(),
                idx + 1
            );
        }
    }
    Ok(data)
}

fn rows(data: &Value) -> Result<&Vec<Value>> {
    data["rows"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("rows must be a list"))
}

fn text(value: &Value) -> Result<&str> {
    value
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("expected string"))
}

fn integer(value: &Value) -> Result<u64> {
    value
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("expected unsigned integer"))
}

fn tex_escape(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '\\' => r"\textbackslash{}".into(),
            '&' => r"\&".into(),
            '%' => r"\%".into(),
            '$' => r"\$".into(),
            '#' => r"\#".into(),
            '_' => r"\_".into(),
            '{' => r"\{".into(),
            '}' => r"\}".into(),
            '~' => r"\textasciitilde{}".into(),
            '^' => r"\textasciicircum{}".into(),
            _ => ch.to_string(),
        })
        .collect()
}

fn fmt_int(value: u64) -> String {
    let text = value.to_string();
    let mut out = String::new();
    for (idx, ch) in text.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn fmt_float(value: f64) -> String {
    format!("{value:.1}")
}

fn ranked_rows(rows: &[Value]) -> Vec<&Value> {
    let mut ranked = rows.iter().collect::<Vec<_>>();
    ranked.sort_by(|a, b| {
        integer(&b["score"])
            .unwrap_or(0)
            .cmp(&integer(&a["score"]).unwrap_or(0))
            .then(
                integer(&a["finding_count"])
                    .unwrap_or(0)
                    .cmp(&integer(&b["finding_count"]).unwrap_or(0)),
            )
            .then(
                text(&a["repo"])
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .cmp(&text(&b["repo"]).unwrap_or_default().to_ascii_lowercase()),
            )
    });
    ranked
}

fn dimension_label(name: &str) -> String {
    match name {
        "Jankurai tool adoption and CI replacement" => "audit CI replacement".into(),
        "Code shape and semantic surface" => "code shape/semantic surface".into(),
        "Context economy and agent instructions" => "context routing".into(),
        "Python containment and polyglot hygiene" => r"Python/\allowbreak{}polyglot hygiene".into(),
        "Proof lanes and test routing" => "proof routing".into(),
        "Security and supply-chain posture" => "security/supply chain".into(),
        "Build speed signals" => "build speed".into(),
        _ => tex_escape(name),
    }
}

fn dimension_labels(row: &Value) -> String {
    let labels = row["weak_dimensions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("name").and_then(Value::as_str))
        .map(dimension_label)
        .collect::<Vec<_>>();
    if labels.is_empty() {
        "none reported".into()
    } else {
        labels.join("; ")
    }
}

fn repo_path(repo: &str) -> String {
    format!(r"\path{{{repo}}}")
}

fn score_cell(score: u64, best_observed: bool) -> String {
    if best_observed {
        format!(r"\JKScoreCellBest{{{score}}}")
    } else {
        format!(r"\JKScoreCell{{{score}}}")
    }
}

fn aggregate_rows(data: &Value) -> Result<Vec<(String, String)>> {
    let rows = rows(data)?;
    let mut scores = rows
        .iter()
        .map(|row| integer(&row["score"]))
        .collect::<Result<Vec<_>>>()?;
    scores.sort_unstable();
    let findings_total = rows
        .iter()
        .map(|row| integer(&row["finding_count"]))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<u64>();
    let hard_total = rows
        .iter()
        .map(|row| integer(&row["hard_findings"]))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<u64>();
    let hard_share = if findings_total == 0 {
        0.0
    } else {
        hard_total as f64 / findings_total as f64 * 100.0
    };
    let average = scores.iter().sum::<u64>() as f64 / scores.len() as f64;
    let upper_middle = scores[scores.len() / 2];
    Ok(vec![
        (
            "Repositories scanned".into(),
            fmt_int(integer(&data["repo_count"])?),
        ),
        (
            "Successful scans".into(),
            fmt_int(integer(&data["successful"])?),
        ),
        ("Failed scans".into(), fmt_int(integer(&data["failed"])?)),
        (
            "Minimum score".into(),
            fmt_int(*scores.first().unwrap_or(&0)),
        ),
        (
            "Maximum score".into(),
            format!("{} (best observed)", fmt_int(*scores.last().unwrap_or(&0))),
        ),
        ("Average score".into(), fmt_float(average)),
        ("Median score (upper middle)".into(), fmt_int(upper_middle)),
        ("Findings total".into(), fmt_int(findings_total)),
        ("Hard findings total".into(), fmt_int(hard_total)),
        (
            "Hard findings share".into(),
            format!("{}\\%", fmt_float(hard_share)),
        ),
    ])
}

fn weak_dimension_rows(rows: &[Value]) -> Vec<(&'static str, u64)> {
    let mut counter = BTreeMap::<String, u64>::new();
    for row in rows {
        for item in row["weak_dimensions"].as_array().into_iter().flatten() {
            if let Some(name) = item.get("name").and_then(Value::as_str) {
                *counter.entry(name.to_string()).or_default() += 1;
            }
        }
    }
    [
        "Jankurai tool adoption and CI replacement",
        "Code shape and semantic surface",
        "Context economy and agent instructions",
        "Python containment and polyglot hygiene",
        "Proof lanes and test routing",
    ]
    .into_iter()
    .map(|name| (name, counter.get(name).copied().unwrap_or(0)))
    .collect()
}

fn render_top_table(rows: &[Value]) -> Result<String> {
    let ranked = ranked_rows(rows);
    let mut out = String::new();
    for line in [
        r"\newcommand{\PublicRepoScoreTopTable}{%",
        r"\begin{table*}[t]",
        r"\caption{Top observed advisory public-repository scores. The top score is best observed in this run, not a certification threshold.}",
        r"\label{tab:public-repo-top-scores}",
        r"\centering",
        r"\scriptsize",
        r"\setlength{\tabcolsep}{3pt}",
        r"\begin{tabularx}{\textwidth}{R{0.045\textwidth} L{0.245\textwidth} R{0.105\textwidth} R{0.075\textwidth} R{0.065\textwidth} Y}",
        r"\toprule",
        r"\JKTableHeader",
        r"\textbf{Rank} & \textbf{Repo} & \textbf{Score} & \textbf{Findings} & \textbf{Hard} & \textbf{Dominant gaps} \\",
        r"\midrule",
        r"\JKDenseRows",
    ] {
        writeln!(out, "{line}")?;
    }
    for (rank, row) in ranked.iter().take(10).enumerate() {
        writeln!(
            out,
            "{} & {} & {} & {} & {} & {} \\\\",
            rank + 1,
            repo_path(text(&row["repo"])?),
            score_cell(integer(&row["score"])?, rank == 0),
            fmt_int(integer(&row["finding_count"])?),
            fmt_int(integer(&row["hard_findings"])?),
            dimension_labels(row)
        )?;
    }
    for line in [
        r"\bottomrule",
        r"\end{tabularx}",
        r"\JKResetRows",
        r"\end{table*}",
        r"}",
    ] {
        writeln!(out, "{line}")?;
    }
    Ok(out.trim_end().into())
}

fn render_aggregate_table(data: &Value) -> Result<String> {
    let mut out = String::new();
    for line in [
        r"\newcommand{\PublicRepoScoreAggregateTable}{%",
        r"\begin{table}[t]",
        r"\caption{Aggregate advisory scan posture.}",
        r"\label{tab:public-repo-aggregate}",
        r"\centering",
        r"\scriptsize",
        r"\setlength{\tabcolsep}{4pt}",
        r"\begin{tabularx}{\columnwidth}{L{0.60\columnwidth} R{0.28\columnwidth}}",
        r"\toprule",
        r"\JKTableHeader",
        r"\textbf{Metric} & \textbf{Value} \\",
        r"\midrule",
        r"\JKDenseRows",
    ] {
        writeln!(out, "{line}")?;
    }
    for (label, value) in aggregate_rows(data)? {
        writeln!(out, "{} & {} \\\\", tex_escape(&label), value)?;
    }
    for line in [
        r"\bottomrule",
        r"\end{tabularx}",
        r"\JKResetRows",
        r"\end{table}",
        r"}",
    ] {
        writeln!(out, "{line}")?;
    }
    Ok(out.trim_end().into())
}

fn render_weak_dimension_table(rows: &[Value]) -> Result<String> {
    let mut out = String::new();
    for line in [
        r"\newcommand{\PublicRepoWeakDimensionTable}{%",
        r"\begin{table}[t]",
        r"\caption{Most frequent weak-dimension sets in the advisory scan.}",
        r"\label{tab:public-repo-weak-dimensions}",
        r"\centering",
        r"\scriptsize",
        r"\setlength{\tabcolsep}{4pt}",
        r"\begin{tabularx}{\columnwidth}{Y R{0.18\columnwidth}}",
        r"\toprule",
        r"\JKTableHeader",
        r"\textbf{Weak dimension} & \textbf{Repos} \\",
        r"\midrule",
        r"\JKDenseRows",
    ] {
        writeln!(out, "{line}")?;
    }
    for (label, count) in weak_dimension_rows(rows) {
        writeln!(out, "{} & {}/{} \\\\", tex_escape(label), count, rows.len())?;
    }
    for line in [
        r"\bottomrule",
        r"\end{tabularx}",
        r"\JKResetRows",
        r"\end{table}",
        r"}",
    ] {
        writeln!(out, "{line}")?;
    }
    Ok(out.trim_end().into())
}

fn render_appendix_table(rows: &[Value]) -> Result<String> {
    let ranked = ranked_rows(rows);
    let mut out = String::new();
    for line in [
        r"\newcommand{\PublicRepoScoreAppendixTable}{%",
        r"\begingroup",
        r"\scriptsize",
        r"\setlength{\tabcolsep}{3pt}",
        r"\setlength{\LTleft}{0pt}",
        r"\setlength{\LTright}{0pt}",
        r"\JKDenseRows",
        r"\begin{longtable}{@{}R{0.040\textwidth} R{0.045\textwidth} L{0.235\textwidth} R{0.055\textwidth} R{0.070\textwidth} R{0.065\textwidth} L{0.355\textwidth}@{}}",
        r"\caption{Full 30-repository advisory scoring run.}\label{tab:public-repo-full}\\",
        r"\toprule",
        r"\JKTableHeader",
        r"\textbf{Rank} & \textbf{Run \#} & \textbf{Repo} & \textbf{Score} & \textbf{Findings} & \textbf{Hard} & \textbf{Dominant gaps} \\",
        r"\midrule",
        r"\endfirsthead",
        r"\toprule",
        r"\JKTableHeader",
        r"\textbf{Rank} & \textbf{Run \#} & \textbf{Repo} & \textbf{Score} & \textbf{Findings} & \textbf{Hard} & \textbf{Dominant gaps} \\",
        r"\midrule",
        r"\endhead",
    ] {
        writeln!(out, "{line}")?;
    }
    for (rank, row) in ranked.iter().enumerate() {
        writeln!(
            out,
            "{} & {} & {} & {} & {} & {} & {} \\\\",
            rank + 1,
            integer(&row["rank"])?,
            repo_path(text(&row["repo"])?),
            score_cell(integer(&row["score"])?, rank == 0),
            fmt_int(integer(&row["finding_count"])?),
            fmt_int(integer(&row["hard_findings"])?),
            dimension_labels(row)
        )?;
    }
    for line in [
        r"\bottomrule",
        r"\end{longtable}",
        r"\JKResetRows",
        r"\endgroup",
        r"}",
    ] {
        writeln!(out, "{line}")?;
    }
    Ok(out.trim_end().into())
}

fn render(data: &Value, source: &PathBuf, out: &PathBuf) -> Result<String> {
    let source_posix = source.to_string_lossy().replace('\\', "/");
    let out_posix = out.to_string_lossy().replace('\\', "/");
    let command = format!(
        "cargo run -p jankurai -- paper public-repo-scores --source {source_posix} --out {out_posix}"
    );
    Ok(format!(
        "% Generated by: cargo run -p jankurai -- paper public-repo-scores\n% Source: {source_posix}\n% Command: {command}\n% DO NOT EDIT BY HAND.\n% Run root: {}\n% Generated at: {}\n% Jankurai version: {}\n\n{}\n\n{}\n\n{}\n\n{}\n",
        text(&data["run_root"])?,
        text(&data["generated_at"])?,
        text(&data["jankurai_version"])?,
        render_top_table(rows(data)?)?,
        render_aggregate_table(data)?,
        render_weak_dimension_table(rows(data)?)?,
        render_appendix_table(rows(data)?)?,
    ))
}
