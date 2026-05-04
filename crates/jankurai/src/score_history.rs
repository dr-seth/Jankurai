use crate::model::Report;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn append_score_history(
    repo: &Path,
    report: &Report,
    json_path: &str,
    md_path: &str,
    history_path: &str,
    csv_path: Option<&str>,
) -> Result<PathBuf> {
    let history_path = resolve_output_path(repo, history_path);
    if let Some(parent) = history_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let decision = report.decision.as_ref();
    let entry = json!({
        "schema_version": "1.0.0",
        "generated_at": report.generated_at,
        "run_id": report.run_id,
        "branch": git_output(repo, &["rev-parse", "--abbrev-ref", "HEAD"]),
        "commit": report.git.as_ref().and_then(|git| git.head.clone()),
        "dirty_worktree": report.dirty_worktree,
        "score": report.score,
        "raw_score": report.raw_score,
        "caps_applied": report.caps_applied,
        "finding_count": report.findings.len(),
        "hard_findings": decision.map(|d| d.hard_findings).unwrap_or(0),
        "soft_findings": decision.map(|d| d.soft_findings).unwrap_or(0),
        "decision": decision.map(|d| d.status.clone()),
        "minimum_score": decision.map(|d| d.minimum_score),
        "scope": report.scope.mode,
        "changed_paths": report.scope.paths,
        "report_fingerprint": report.report_fingerprint,
        "repo_score_json_path": display_path(repo, json_path),
        "repo_score_md_path": display_path(repo, md_path),
    });

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history_path)
        .with_context(|| format!("open {}", history_path.display()))?;
    writeln!(file, "{}", serde_json::to_string(&entry)?)
        .with_context(|| format!("append {}", history_path.display()))?;

    if let Some(csv_path) = csv_path {
        append_score_history_csv(repo, &history_path, csv_path, &entry)?;
    }

    Ok(history_path)
}

fn append_score_history_csv(
    repo: &Path,
    history_path: &Path,
    csv_path: &str,
    entry: &Value,
) -> Result<()> {
    let csv_path = resolve_output_path(repo, csv_path);
    if let Some(parent) = csv_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let needs_header = !csv_path.exists() || csv_path.metadata().map(|m| m.len()).unwrap_or(0) == 0;
    let index = fs::read_to_string(history_path)
        .with_context(|| format!("read {}", history_path.display()))?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    let row = csv_row(index, entry);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&csv_path)
        .with_context(|| format!("open {}", csv_path.display()))?;
    if needs_header {
        writeln!(file, "{}", csv_header())
            .with_context(|| format!("write {}", csv_path.display()))?;
    }
    writeln!(file, "{}", row).with_context(|| format!("append {}", csv_path.display()))?;
    Ok(())
}

fn csv_header() -> &'static str {
    "index,generated_at,branch,commit,dirty_worktree,score,raw_score,caps,finding_count,hard_findings,soft_findings,decision,minimum_score,scope,report_fingerprint"
}

fn csv_row(index: usize, value: &Value) -> String {
    let row = [
        index.to_string(),
        string_field(value, "generated_at"),
        string_field(value, "branch"),
        string_field(value, "commit"),
        bool_field(value, "dirty_worktree"),
        int_field(value, "score"),
        int_field(value, "raw_score"),
        array_len_field(value, "caps_applied"),
        int_field(value, "finding_count"),
        int_field(value, "hard_findings"),
        int_field(value, "soft_findings"),
        string_field(value, "decision"),
        int_field(value, "minimum_score"),
        string_field(value, "scope"),
        string_field(value, "report_fingerprint"),
    ];
    row.iter()
        .map(|field| csv_escape(field))
        .collect::<Vec<_>>()
        .join(",")
}

fn resolve_output_path(repo: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo.join(path)
    }
}

fn display_path(repo: &Path, path: &str) -> String {
    let path = resolve_output_path(repo, path);
    path.strip_prefix(repo)
        .unwrap_or(&path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn string_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn int_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
        .unwrap_or_default()
}

fn bool_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_bool)
        .map(|value| value.to_string())
        .unwrap_or_default()
}

fn array_len_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|value| value.len().to_string())
        .unwrap_or_default()
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn git_output(repo: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}
