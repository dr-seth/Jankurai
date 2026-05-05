use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as GitProcess;
use std::time::{SystemTime, UNIX_EPOCH};

pub const PROOFMARK_SCHEMA_VERSION: &str = "1.0.0";
pub const PROOFMARK_STANDARD_VERSION: &str = "0.7.0";
pub const PROOFMARK_AUDITOR_VERSION: &str = "0.7.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofMarkMode {
    Advisory,
    Required,
}

impl ProofMarkMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Advisory => "advisory",
            Self::Required => "required",
        }
    }
}

impl std::str::FromStr for ProofMarkMode {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "advisory" => Ok(Self::Advisory),
            "required" => Ok(Self::Required),
            other => anyhow::bail!("unknown proofmark mode `{other}`"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProofMarkRequest {
    pub repo_root: PathBuf,
    pub changed_paths: Vec<PathBuf>,
    pub changed_from: Option<String>,
    pub obligations_path: Option<PathBuf>,
    pub coverage_path: Option<PathBuf>,
    pub mutation_path: Option<PathBuf>,
    pub negative_proofs: Vec<String>,
    pub mode: ProofMarkMode,
}

#[derive(Debug, Clone)]
pub struct ProofMarkOutput {
    pub receipt: ProofMarkReceipt,
    pub proof_receipt: StandardProofReceipt,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMarkReceipt {
    pub schema_version: String,
    pub standard_version: String,
    pub generated_at: String,
    pub repo_root: String,
    pub git_head: String,
    pub mode: String,
    pub changed_paths: Vec<String>,
    pub changed_units: Vec<ChangedUnit>,
    pub coverage: CoverageSummary,
    pub mutation: MutationSummary,
    pub obligation_results: Vec<ObligationResult>,
    pub satisfied_obligations: Vec<String>,
    pub summary: ProofMarkSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedUnit {
    pub path: String,
    pub unit: String,
    pub changed_lines: Vec<u32>,
    pub covered_changed_lines: Vec<u32>,
    pub uncovered_changed_lines: Vec<u32>,
    pub coverage_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub source: String,
    pub changed_line_count: usize,
    pub covered_changed_line_count: usize,
    pub uncovered_changed_line_count: usize,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationSummary {
    pub source: String,
    pub status: String,
    pub killed: usize,
    pub survived: usize,
    pub timeout: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObligationResult {
    pub obligation_id: String,
    pub path: String,
    pub rule_ids: Vec<String>,
    pub required_lanes: Vec<String>,
    pub status: String,
    pub coverage_status: String,
    pub mutation_status: String,
    pub negative_proof_status: String,
    pub evidence: Vec<String>,
    pub residual_risk: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMarkSummary {
    pub total_obligations: usize,
    pub satisfied_obligations: usize,
    pub review_obligations: usize,
    pub changed_units: usize,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardProofReceipt {
    pub schema_version: String,
    pub standard_version: String,
    pub auditor_version: String,
    pub receipt_id: String,
    pub lane: String,
    pub command: String,
    pub exit_code: i32,
    pub elapsed_ms: u128,
    pub artifacts: Vec<String>,
    pub changed_paths: Vec<String>,
    pub generated_at: String,
    pub repo_root: String,
    pub git_head: String,
    pub dirty_worktree: bool,
    pub rules_covered: Vec<RuleCoverage>,
    pub extensions: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCoverage {
    pub rule_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ProofBindObligations {
    #[serde(default)]
    obligations: Vec<ProofBindObligation>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ProofBindObligation {
    obligation_id: String,
    path: String,
    #[serde(default)]
    rule_ids: Vec<String>,
    #[serde(default)]
    required_lanes: Vec<String>,
    #[serde(default)]
    required_receipt_kinds: Vec<String>,
}

pub fn build_proofmark(request: ProofMarkRequest) -> Result<ProofMarkOutput> {
    let started = SystemTime::now();
    let repo = request.repo_root;
    let changed_paths = resolve_changed_paths(
        &repo,
        &request.changed_paths,
        request.changed_from.as_deref(),
    )?;
    let coverage = load_coverage(&repo, request.coverage_path.as_deref())?;
    let mutation = load_mutation(&repo, request.mutation_path.as_deref())?;
    let changed_lines =
        changed_lines_for_paths(&repo, request.changed_from.as_deref(), &changed_paths);
    let changed_units = changed_paths
        .iter()
        .filter(|path| path.ends_with(".rs"))
        .map(|path| changed_unit(path, changed_lines.get(path), &coverage))
        .collect::<Vec<_>>();

    let obligations = load_obligations(&repo, request.obligations_path.as_deref())?;
    let negative_proofs = request
        .negative_proofs
        .iter()
        .map(|item| item.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let obligation_results = obligations
        .iter()
        .filter(|obligation| {
            obligation
                .required_lanes
                .iter()
                .any(|lane| lane == "proofmark-rust")
                || obligation.path.ends_with(".rs")
        })
        .map(|obligation| {
            obligation_result(obligation, &changed_units, &mutation, &negative_proofs)
        })
        .collect::<Vec<_>>();
    let satisfied_obligations = obligation_results
        .iter()
        .filter(|result| result.status == "pass")
        .map(|result| result.obligation_id.clone())
        .collect::<Vec<_>>();
    let coverage_summary = coverage_summary(
        request.coverage_path.as_deref(),
        &changed_units,
        coverage.loaded,
    );
    let summary = proofmark_summary(&changed_units, &obligation_results, request.mode);
    let generated_at = unix_seconds();
    let git_head = match git_output(&repo, &["rev-parse", "--short", "HEAD"]) {
        Some(head) => head,
        None => "unknown".into(),
    };
    let receipt = ProofMarkReceipt {
        schema_version: PROOFMARK_SCHEMA_VERSION.into(),
        standard_version: PROOFMARK_STANDARD_VERSION.into(),
        generated_at: generated_at.clone(),
        repo_root: repo.display().to_string(),
        git_head: git_head.clone(),
        mode: request.mode.as_str().into(),
        changed_paths: changed_paths.clone(),
        changed_units,
        coverage: coverage_summary,
        mutation,
        obligation_results,
        satisfied_obligations,
        summary,
    };
    let proof_receipt = standard_proof_receipt(
        &repo,
        &receipt,
        elapsed_ms(started),
        &generated_at,
        &git_head,
    );
    let markdown = render_markdown(&receipt);
    Ok(ProofMarkOutput {
        receipt,
        proof_receipt,
        markdown,
    })
}

fn obligation_result(
    obligation: &ProofBindObligation,
    units: &[ChangedUnit],
    mutation: &MutationSummary,
    negative_proofs: &BTreeSet<String>,
) -> ObligationResult {
    let matching_units = units
        .iter()
        .filter(|unit| unit.path == obligation.path)
        .collect::<Vec<_>>();
    let coverage_pass = !matching_units.is_empty()
        && matching_units
            .iter()
            .all(|unit| unit.coverage_status == "pass");
    let coverage_status = if matching_units.is_empty() {
        "unavailable"
    } else if coverage_pass {
        "pass"
    } else {
        "review"
    };
    let boundary_sensitive = obligation.rule_ids.iter().any(|rule| {
        matches!(
            rule.as_str(),
            "HLT-021-DESTRUCTIVE-MIGRATION"
                | "HLT-022-AUTHZ-ISOLATION-GAP"
                | "HLT-023-INPUT-BOUNDARY-GAP"
                | "HLT-024-AGENT-TOOL-SUPPLY-GAP"
        )
    }) || obligation
        .required_receipt_kinds
        .iter()
        .any(|kind| kind == "negative-behavior-proof");
    let mutation_status = mutation.status.clone();
    let negative_proof_status = if !boundary_sensitive {
        "not_required".to_string()
    } else if negative_proofs.contains(&obligation.obligation_id.to_ascii_lowercase())
        || obligation
            .rule_ids
            .iter()
            .any(|rule| negative_proofs.contains(&rule.to_ascii_lowercase()))
        || negative_proofs.contains(&obligation.path.to_ascii_lowercase())
    {
        "present".to_string()
    } else {
        "missing".to_string()
    };
    let mutation_ok = mutation.status == "pass" || mutation.status == "unavailable";
    let negative_ok = negative_proof_status != "missing";
    let status = if coverage_pass && mutation_ok && negative_ok {
        "pass"
    } else {
        "review"
    };
    let mut evidence = Vec::new();
    if coverage_pass {
        evidence.push("all changed Rust lines are covered by supplied coverage evidence".into());
    }
    if mutation.status == "pass" {
        evidence.push("mutation evidence has no survived in-diff mutants".into());
    }
    if negative_proof_status == "present" {
        evidence.push("negative proof marker matched obligation, rule, or path".into());
    }
    let mut residual_risk = Vec::new();
    if coverage_status != "pass" {
        residual_risk.push("changed-line coverage is missing or incomplete".into());
    }
    if mutation.status == "unavailable" {
        residual_risk.push("focused mutation evidence was not supplied".into());
    } else if mutation.status != "pass" {
        residual_risk.push("mutation evidence reports survived or timed-out mutants".into());
    }
    if negative_proof_status == "missing" {
        residual_risk.push("boundary-sensitive change lacks negative behavior proof".into());
    }
    ObligationResult {
        obligation_id: obligation.obligation_id.clone(),
        path: obligation.path.clone(),
        rule_ids: obligation.rule_ids.clone(),
        required_lanes: obligation.required_lanes.clone(),
        status: status.into(),
        coverage_status: coverage_status.into(),
        mutation_status,
        negative_proof_status,
        evidence,
        residual_risk,
    }
}

fn standard_proof_receipt(
    repo: &Path,
    receipt: &ProofMarkReceipt,
    elapsed_ms: u128,
    generated_at: &str,
    git_head: &str,
) -> StandardProofReceipt {
    let mut rules = BTreeMap::<String, String>::new();
    for result in &receipt.obligation_results {
        for rule in &result.rule_ids {
            let status = if result.status == "pass" {
                "covered"
            } else {
                "review"
            };
            rules
                .entry(rule.clone())
                .and_modify(|existing| {
                    if *existing == "covered" && status != "covered" {
                        *existing = status.into();
                    }
                })
                .or_insert_with(|| status.into());
        }
    }
    let mut extensions = Map::new();
    extensions.insert(
        "proofmark".into(),
        json!({
            "schema_version": receipt.schema_version,
            "changed_units": receipt.changed_units,
            "coverage": receipt.coverage,
            "mutation": receipt.mutation,
            "obligation_results": receipt.obligation_results,
            "satisfied_obligations": receipt.satisfied_obligations,
            "summary": receipt.summary,
        }),
    );
    StandardProofReceipt {
        schema_version: "1.0.0".into(),
        standard_version: PROOFMARK_STANDARD_VERSION.into(),
        auditor_version: PROOFMARK_AUDITOR_VERSION.into(),
        receipt_id: format!("proofmark-rust-{}", generated_at),
        lane: "proofmark-rust".into(),
        command: "jankurai proofmark rust".into(),
        exit_code: 0,
        elapsed_ms,
        artifacts: vec![
            "target/jankurai/proofmark/proofmark-receipt.json".into(),
            "target/jankurai/proofmark/proofmark.md".into(),
        ],
        changed_paths: receipt.changed_paths.clone(),
        generated_at: generated_at.into(),
        repo_root: repo.display().to_string(),
        git_head: git_head.into(),
        dirty_worktree: git_dirty(repo),
        rules_covered: rules
            .into_iter()
            .map(|(rule_id, status)| RuleCoverage { rule_id, status })
            .collect(),
        extensions,
    }
}

fn proofmark_summary(
    units: &[ChangedUnit],
    results: &[ObligationResult],
    mode: ProofMarkMode,
) -> ProofMarkSummary {
    let satisfied = results
        .iter()
        .filter(|result| result.status == "pass")
        .count();
    let review = results.len().saturating_sub(satisfied);
    let verdict = if review == 0 {
        "pass"
    } else if mode == ProofMarkMode::Required {
        "block"
    } else {
        "review"
    };
    ProofMarkSummary {
        total_obligations: results.len(),
        satisfied_obligations: satisfied,
        review_obligations: review,
        changed_units: units.len(),
        verdict: verdict.into(),
    }
}

fn changed_unit(
    path: &str,
    changed_lines: Option<&BTreeSet<u32>>,
    coverage: &CoverageData,
) -> ChangedUnit {
    let changed_lines = match changed_lines {
        Some(lines) => lines.clone(),
        None => BTreeSet::from([1]),
    }
    .into_iter()
    .collect::<Vec<_>>();
    let covered = coverage.covered_lines(path);
    let mut covered_changed_lines = Vec::new();
    let mut uncovered_changed_lines = Vec::new();
    for line in &changed_lines {
        if covered.contains(line) {
            covered_changed_lines.push(*line);
        } else {
            uncovered_changed_lines.push(*line);
        }
    }
    let coverage_status = if !coverage.loaded {
        "unavailable"
    } else if uncovered_changed_lines.is_empty() {
        "pass"
    } else {
        "review"
    };
    ChangedUnit {
        path: path.into(),
        unit: path_symbol(path),
        changed_lines,
        covered_changed_lines,
        uncovered_changed_lines,
        coverage_status: coverage_status.into(),
    }
}

fn coverage_summary(source: Option<&Path>, units: &[ChangedUnit], loaded: bool) -> CoverageSummary {
    let changed_line_count = units.iter().map(|unit| unit.changed_lines.len()).sum();
    let covered_changed_line_count = units
        .iter()
        .map(|unit| unit.covered_changed_lines.len())
        .sum();
    let uncovered_changed_line_count = units
        .iter()
        .map(|unit| unit.uncovered_changed_lines.len())
        .sum();
    let status = if !loaded {
        "unavailable"
    } else if uncovered_changed_line_count == 0 {
        "pass"
    } else {
        "review"
    };
    CoverageSummary {
        source: match source {
            Some(path) => path.display().to_string(),
            None => "unavailable".into(),
        },
        changed_line_count,
        covered_changed_line_count,
        uncovered_changed_line_count,
        status: status.into(),
    }
}

#[derive(Debug, Clone, Default)]
struct CoverageData {
    loaded: bool,
    files: BTreeMap<String, BTreeSet<u32>>,
}

impl CoverageData {
    fn covered_lines(&self, path: &str) -> BTreeSet<u32> {
        let mut out = BTreeSet::new();
        for (file, lines) in &self.files {
            if file == path || file.ends_with(&format!("/{path}")) || path.ends_with(file) {
                out.extend(lines.iter().copied());
            }
        }
        out
    }
}

fn load_coverage(repo: &Path, path: Option<&Path>) -> Result<CoverageData> {
    let Some(path) = path else {
        return Ok(CoverageData::default());
    };
    let path = resolve(repo, path);
    let text =
        fs::read_to_string(&path).with_context(|| format!("read coverage {}", path.display()))?;
    if text.trim_start().starts_with('{') {
        return Ok(load_json_coverage(&text));
    }
    Ok(load_lcov(&text))
}

fn load_lcov(text: &str) -> CoverageData {
    let mut data = CoverageData {
        loaded: true,
        files: BTreeMap::new(),
    };
    let mut current: Option<String> = None;
    for line in text.lines() {
        if let Some(file) = line.strip_prefix("SF:") {
            current = Some(file.trim().replace('\\', "/"));
        } else if let Some(rest) = line.strip_prefix("DA:") {
            let Some(file) = current.clone() else {
                continue;
            };
            let mut parts = rest.split(',');
            let line_no = parts.next().and_then(|value| value.parse::<u32>().ok());
            let hits = parts.next().and_then(|value| value.parse::<u64>().ok());
            if let (Some(line_no), Some(hits)) = (line_no, hits) {
                if hits > 0 {
                    data.files.entry(file).or_default().insert(line_no);
                }
            }
        }
    }
    data
}

fn load_json_coverage(text: &str) -> CoverageData {
    let mut data = CoverageData {
        loaded: true,
        files: BTreeMap::new(),
    };
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        data.loaded = false;
        return data;
    };
    if let Some(files) = value.get("files").and_then(Value::as_array) {
        for file in files {
            let filename_value = match file.get("filename") {
                Some(value) => Some(value),
                None => file.get("path"),
            };
            let Some(filename) = filename_value.and_then(Value::as_str) else {
                continue;
            };
            if let Some(lines) = file.get("covered_lines").and_then(Value::as_array) {
                for line in lines.iter().filter_map(Value::as_u64) {
                    data.files
                        .entry(filename.replace('\\', "/"))
                        .or_default()
                        .insert(line as u32);
                }
            }
        }
    }
    data
}

fn load_mutation(repo: &Path, path: Option<&Path>) -> Result<MutationSummary> {
    let Some(path) = path else {
        return Ok(MutationSummary {
            source: "unavailable".into(),
            status: "unavailable".into(),
            killed: 0,
            survived: 0,
            timeout: 0,
        });
    };
    let path = resolve(repo, path);
    let text =
        fs::read_to_string(&path).with_context(|| format!("read mutation {}", path.display()))?;
    let value: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(_) => json!({}),
    };
    let killed = number_at(&value, &["killed", "caught", "success"]);
    let survived = number_at(&value, &["survived", "missed", "unmutated"]);
    let timeout = number_at(&value, &["timeout", "timed_out"]);
    Ok(MutationSummary {
        source: path.display().to_string(),
        status: if survived == 0 && timeout == 0 {
            "pass"
        } else {
            "review"
        }
        .into(),
        killed,
        survived,
        timeout,
    })
}

fn number_at(value: &Value, keys: &[&str]) -> usize {
    for key in keys {
        if let Some(number) = value.get(*key).and_then(Value::as_u64) {
            return number as usize;
        }
        if let Some(number) = value
            .get("summary")
            .and_then(|summary| summary.get(*key))
            .and_then(Value::as_u64)
        {
            return number as usize;
        }
    }
    0
}

fn load_obligations(repo: &Path, path: Option<&Path>) -> Result<Vec<ProofBindObligation>> {
    let Some(path) = path else {
        return Ok(vec![]);
    };
    let path = resolve(repo, path);
    if !path.exists() {
        return Ok(vec![]);
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("read obligations {}", path.display()))?;
    let parsed: ProofBindObligations = serde_json::from_str(&text)
        .with_context(|| format!("parse obligations {}", path.display()))?;
    Ok(parsed.obligations)
}

fn changed_lines_for_paths(
    repo: &Path,
    changed_from: Option<&str>,
    paths: &[String],
) -> BTreeMap<String, BTreeSet<u32>> {
    let mut out = BTreeMap::new();
    for path in paths {
        if !path.ends_with(".rs") {
            continue;
        }
        let lines = changed_lines_from_git(repo, changed_from, path).unwrap_or(Default::default());
        if lines.is_empty() {
            out.insert(path.clone(), BTreeSet::from([1]));
        } else {
            out.insert(path.clone(), lines);
        }
    }
    out
}

fn changed_lines_from_git(
    repo: &Path,
    changed_from: Option<&str>,
    path: &str,
) -> Result<BTreeSet<u32>> {
    let mut command = GitProcess::new("git");
    command.current_dir(repo).arg("diff").arg("--unified=0");
    if let Some(base) = changed_from {
        command.arg(format!("{base}...HEAD"));
    }
    command.arg("--").arg(path);
    let output = command.output()?;
    if !output.status.success() {
        return Ok(BTreeSet::new());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(parse_unified_diff_changed_lines(&text))
}

fn parse_unified_diff_changed_lines(diff: &str) -> BTreeSet<u32> {
    let mut out = BTreeSet::new();
    let mut current_new_line = 0u32;
    for line in diff.lines() {
        if line.starts_with("@@") {
            if let Some(start) = parse_hunk_new_start(line) {
                current_new_line = start;
            }
            continue;
        }
        if current_new_line == 0 {
            continue;
        }
        if line.starts_with('+') && !line.starts_with("+++") {
            out.insert(current_new_line);
            current_new_line = current_new_line.saturating_add(1);
        } else if line.starts_with('-') && !line.starts_with("---") {
            continue;
        } else {
            current_new_line = current_new_line.saturating_add(1);
        }
    }
    out
}

fn parse_hunk_new_start(line: &str) -> Option<u32> {
    let plus = line.find('+')?;
    let rest = &line[plus + 1..];
    let number = rest.split(|c: char| c == ',' || c.is_whitespace()).next()?;
    number.parse().ok()
}

fn resolve_changed_paths(
    repo: &Path,
    changed: &[PathBuf],
    changed_from: Option<&str>,
) -> Result<Vec<String>> {
    let paths = if let Some(base) = changed_from {
        changed_paths_from_git(repo, base)?
    } else if changed.is_empty() {
        local_changed_paths_from_git(repo)?
    } else {
        changed
            .iter()
            .filter_map(|path| normalize_changed_path(repo, path))
            .collect()
    };
    let mut paths = paths
        .into_iter()
        .filter(|path| !path.trim().is_empty())
        .filter(|path| path.ends_with(".rs"))
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn changed_paths_from_git(repo: &Path, base: &str) -> Result<Vec<String>> {
    let refspec = format!("{base}...HEAD");
    let output = GitProcess::new("git")
        .args(["diff", "--name-only", refspec.as_str()])
        .current_dir(repo)
        .output()
        .with_context(|| format!("run git diff for {base}"))?;
    if !output.status.success() {
        return Ok(vec![]);
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn local_changed_paths_from_git(repo: &Path) -> Result<Vec<String>> {
    let output = GitProcess::new("git")
        .args(["diff", "--name-only"])
        .current_dir(repo)
        .output();
    let Ok(output) = output else {
        return Ok(vec![]);
    };
    if !output.status.success() {
        return Ok(vec![]);
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn normalize_changed_path(repo: &Path, path: &Path) -> Option<String> {
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo.join(path)
    };
    candidate
        .strip_prefix(repo)
        .ok()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

pub fn render_markdown(receipt: &ProofMarkReceipt) -> String {
    let mut out = String::new();
    out.push_str("# jankurai ProofMark\n\n");
    out.push_str(&format!("- mode: `{}`\n", receipt.mode));
    out.push_str(&format!(
        "- changed units: `{}`\n",
        receipt.summary.changed_units
    ));
    out.push_str(&format!(
        "- obligations: total=`{}` satisfied=`{}` review=`{}` verdict=`{}`\n",
        receipt.summary.total_obligations,
        receipt.summary.satisfied_obligations,
        receipt.summary.review_obligations,
        receipt.summary.verdict
    ));
    out.push_str(&format!(
        "- coverage: status=`{}` changed=`{}` covered=`{}` uncovered=`{}`\n",
        receipt.coverage.status,
        receipt.coverage.changed_line_count,
        receipt.coverage.covered_changed_line_count,
        receipt.coverage.uncovered_changed_line_count
    ));
    out.push_str(&format!(
        "- mutation: status=`{}` killed=`{}` survived=`{}` timeout=`{}`\n",
        receipt.mutation.status,
        receipt.mutation.killed,
        receipt.mutation.survived,
        receipt.mutation.timeout
    ));
    out.push_str("\n## Obligation Results\n");
    if receipt.obligation_results.is_empty() {
        out.push_str("- none\n");
    } else {
        for result in &receipt.obligation_results {
            out.push_str(&format!(
                "- `{}` status=`{}` coverage=`{}` mutation=`{}` negative=`{}`\n",
                result.path,
                result.status,
                result.coverage_status,
                result.mutation_status,
                result.negative_proof_status
            ));
        }
    }
    out
}

fn path_symbol(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("file")
        .to_string()
}

fn resolve(repo: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo.join(path)
    }
}

fn git_output(repo: &Path, args: &[&str]) -> Option<String> {
    GitProcess::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn git_dirty(repo: &Path) -> bool {
    GitProcess::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo)
        .output()
        .ok()
        .map(|out| !out.stdout.is_empty())
        .unwrap_or(false)
}

fn unix_seconds() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Default::default())
        .as_secs()
        .to_string()
}

fn elapsed_ms(started: SystemTime) -> u128 {
    started.elapsed().unwrap_or(Default::default()).as_millis()
}
