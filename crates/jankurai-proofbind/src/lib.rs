use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as GitProcess;
use std::time::{SystemTime, UNIX_EPOCH};

mod receipts;
use receipts::{load_receipts, receipt_satisfies, ReceiptEvidence};

pub const PROOFBIND_SCHEMA_VERSION: &str = "1.0.0";
pub const PROOFBIND_STANDARD_VERSION: &str = "0.7.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofBindMode {
    Advisory,
    Required,
}

impl ProofBindMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Advisory => "advisory",
            Self::Required => "required",
        }
    }
}

impl std::str::FromStr for ProofBindMode {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "advisory" => Ok(Self::Advisory),
            "required" => Ok(Self::Required),
            other => anyhow::bail!("unknown proofbind mode `{other}`"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProofBindRequest {
    pub repo_root: PathBuf,
    pub changed_paths: Vec<PathBuf>,
    pub changed_from: Option<String>,
    pub mode: ProofBindMode,
    pub proof_receipts: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ProofBindOutput {
    pub witness: SurfaceWitness,
    pub obligations: ProofBindObligations,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceWitness {
    pub schema_version: String,
    pub standard_version: String,
    pub generated_at: String,
    pub repo_root: String,
    pub git_head: String,
    pub mode: String,
    pub changed_paths: Vec<String>,
    pub surfaces: Vec<ChangedSurface>,
    pub summary: SurfaceSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SurfaceSummary {
    pub changed_surface_count: usize,
    pub high_or_critical_surface_count: usize,
    pub by_surface_type: BTreeMap<String, usize>,
    pub by_owner: BTreeMap<String, usize>,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedSurface {
    pub surface_id: String,
    pub path: String,
    pub symbol: String,
    pub surface_type: String,
    pub severity: String,
    pub risk_tags: Vec<String>,
    pub owner: String,
    pub owner_route: String,
    pub test_route: String,
    pub proof_lane: String,
    pub required_rules: Vec<String>,
    pub required_lanes: Vec<String>,
    pub repair_tasks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofBindObligations {
    pub schema_version: String,
    pub standard_version: String,
    pub generated_at: String,
    pub repo_root: String,
    pub git_head: String,
    pub mode: String,
    pub obligations: Vec<ProofObligation>,
    pub summary: ObligationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObligationSummary {
    pub total: usize,
    pub satisfied: usize,
    pub missing: usize,
    pub high_or_critical_missing: usize,
    pub changed_surface_count: usize,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofObligation {
    pub obligation_id: String,
    pub surface_id: String,
    pub path: String,
    pub symbol: String,
    pub surface_type: String,
    pub severity: String,
    pub risk_tags: Vec<String>,
    pub rule_ids: Vec<String>,
    pub required_lanes: Vec<String>,
    pub required_receipt_kinds: Vec<String>,
    pub repair_task: String,
    pub satisfied: bool,
    pub status: String,
    pub receipt_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct OwnerMap {
    #[serde(default)]
    owners: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct TestMap {
    #[serde(default)]
    tests: BTreeMap<String, TestSpec>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct TestSpec {
    command: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ProofLanes {
    #[serde(default)]
    lane: Vec<ProofLane>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ProofLane {
    name: String,
    command: String,
}

#[derive(Debug, Clone, Default)]
struct Catalog {
    owner_map: OwnerMap,
    test_map: TestMap,
    proof_lanes: ProofLanes,
}

pub fn build_proofbind(request: ProofBindRequest) -> Result<ProofBindOutput> {
    let repo = request.repo_root;
    let changed_paths = resolve_changed_paths(
        &repo,
        &request.changed_paths,
        request.changed_from.as_deref(),
    )?;
    let catalog = Catalog::load(&repo);
    let mut surfaces = Vec::new();
    for path in &changed_paths {
        surfaces.extend(classify_changed_path(&repo, &catalog, path)?);
    }
    surfaces.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.surface_type.cmp(&b.surface_type))
            .then(a.symbol.cmp(&b.symbol))
    });
    surfaces.dedup_by(|a, b| a.surface_id == b.surface_id);

    let receipts = load_receipts(&repo, request.proof_receipts.as_deref())?;
    let mut obligations = surfaces
        .iter()
        .map(|surface| obligation_for_surface(surface, &receipts))
        .collect::<Vec<_>>();
    obligations.sort_by(|a, b| a.obligation_id.cmp(&b.obligation_id));

    let generated_at = unix_seconds();
    let git_head = match git_output(&repo, &["rev-parse", "--short", "HEAD"]) {
        Some(head) => head,
        None => "unknown".into(),
    };
    let witness_summary = surface_summary(&surfaces);
    let obligation_summary = obligation_summary(&surfaces, &obligations, request.mode);
    let witness = SurfaceWitness {
        schema_version: PROOFBIND_SCHEMA_VERSION.into(),
        standard_version: PROOFBIND_STANDARD_VERSION.into(),
        generated_at: generated_at.clone(),
        repo_root: repo.display().to_string(),
        git_head: git_head.clone(),
        mode: request.mode.as_str().into(),
        changed_paths,
        surfaces,
        summary: witness_summary,
    };
    let obligations = ProofBindObligations {
        schema_version: PROOFBIND_SCHEMA_VERSION.into(),
        standard_version: PROOFBIND_STANDARD_VERSION.into(),
        generated_at,
        repo_root: repo.display().to_string(),
        git_head,
        mode: request.mode.as_str().into(),
        obligations,
        summary: obligation_summary,
    };
    let markdown = render_markdown(&witness, &obligations);
    Ok(ProofBindOutput {
        witness,
        obligations,
        markdown,
    })
}

impl Catalog {
    fn load(repo: &Path) -> Self {
        Self {
            owner_map: read_json(repo.join("agent/owner-map.json")).unwrap_or(Default::default()),
            test_map: read_json(repo.join("agent/test-map.json")).unwrap_or(Default::default()),
            proof_lanes: read_toml(repo.join("agent/proof-lanes.toml"))
                .unwrap_or(Default::default()),
        }
    }

    fn owner_for_path(&self, path: &str) -> (String, String) {
        let matched = self
            .owner_map
            .owners
            .iter()
            .filter(|(prefix, _)| prefix_matches(prefix, path))
            .max_by(|(a, _), (b, _)| a.len().cmp(&b.len()).then(a.cmp(b)))
            .map(|(prefix, owner)| (owner.clone(), normalize_prefix(prefix)));
        match matched {
            Some(route) => route,
            None => ("unmapped".into(), "unmapped".into()),
        }
    }

    fn test_for_path(&self, path: &str) -> (String, String) {
        let matched = self
            .test_map
            .tests
            .iter()
            .filter(|(prefix, _)| prefix_matches(prefix, path))
            .max_by(|(a, _), (b, _)| a.len().cmp(&b.len()).then(a.cmp(b)))
            .map(|(prefix, spec)| {
                let route = normalize_prefix(prefix);
                let lane = self
                    .proof_lanes
                    .lane
                    .iter()
                    .find(|lane| lane.command.trim() == spec.command.trim())
                    .map(|lane| lane.name.clone());
                let lane = match lane {
                    Some(lane) => lane,
                    None => "test-map".into(),
                };
                (route, lane)
            });
        match matched {
            Some(route) => route,
            None => ("unmapped".into(), "unmapped".into()),
        }
    }
}

fn classify_changed_path(
    repo: &Path,
    catalog: &Catalog,
    path: &str,
) -> Result<Vec<ChangedSurface>> {
    let full_path = repo.join(path);
    let text = fs::read_to_string(&full_path).unwrap_or(String::new());
    let lower_path = path.to_ascii_lowercase();
    let lower_text = text.to_ascii_lowercase();
    let mut surfaces = Vec::new();

    if lower_path.ends_with(".rs") {
        for symbol in rust_public_symbols(&text) {
            surfaces.push(surface(
                catalog,
                path,
                &symbol,
                "rust_public_api",
                "medium",
                vec!["public_api"],
                vec!["HLT-007-HANDWRITTEN-CONTRACT"],
                vec!["contract", "proofmark-rust"],
            ));
        }
        if lower_path.ends_with("main.rs")
            || lower_path.contains("/commands/")
            || lower_text.contains("subcommand")
        {
            let symbol = path_symbol(path);
            surfaces.push(surface(
                catalog,
                path,
                &symbol,
                "cli_command",
                "high",
                vec!["tool_surface", "operator_boundary"],
                vec!["HLT-024-AGENT-TOOL-SUPPLY-GAP"],
                vec!["security", "proofmark-rust"],
            ));
        }
        if contains_authz_marker(&lower_path, &lower_text) {
            surfaces.push(surface(
                catalog,
                path,
                "authz",
                "authz_boundary",
                "critical",
                vec![
                    "authorization",
                    "tenant_isolation",
                    "negative_proof_required",
                ],
                vec!["HLT-022-AUTHZ-ISOLATION-GAP"],
                vec!["security", "proofmark-rust"],
            ));
        }
        if contains_input_marker(&lower_path, &lower_text) {
            surfaces.push(surface(
                catalog,
                path,
                "input",
                "input_boundary",
                "high",
                vec!["input_validation", "negative_proof_required"],
                vec!["HLT-023-INPUT-BOUNDARY-GAP"],
                vec!["security", "proofmark-rust"],
            ));
        }
        if contains_process_sink(&lower_text) {
            surfaces.push(surface(
                catalog,
                path,
                "process_sink",
                "unsafe_or_process_sink",
                "high",
                vec!["process", "filesystem", "unsafe_sink"],
                vec!["HLT-023-INPUT-BOUNDARY-GAP"],
                vec!["security", "proofmark-rust"],
            ));
        }
    }

    if lower_path.ends_with(".sql") {
        let destructive = contains_destructive_sql(&lower_text);
        surfaces.push(surface(
            catalog,
            path,
            "sql",
            "sql_query",
            if destructive { "critical" } else { "high" },
            if destructive {
                vec!["sql", "destructive"]
            } else {
                vec!["sql"]
            },
            if destructive {
                vec!["HLT-021-DESTRUCTIVE-MIGRATION"]
            } else {
                vec!["HLT-006-DIRECT-DB-WRONG-LAYER"]
            },
            if destructive {
                vec!["db-migration-analyze"]
            } else {
                vec!["db"]
            },
        ));
        if lower_path.contains("migration") || lower_path.starts_with("db/") {
            surfaces.push(surface(
                catalog,
                path,
                "migration",
                "db_migration",
                if destructive { "critical" } else { "medium" },
                if destructive {
                    vec!["migration", "destructive"]
                } else {
                    vec!["migration"]
                },
                if destructive {
                    vec!["HLT-021-DESTRUCTIVE-MIGRATION"]
                } else {
                    vec!["HLT-006-DIRECT-DB-WRONG-LAYER"]
                },
                vec!["db-migration-analyze"],
            ));
        }
    }

    if is_agent_tool_surface(&lower_path, &lower_text) {
        surfaces.push(surface(
            catalog,
            path,
            "tool",
            if lower_text.contains("mcp") || lower_path.contains("mcp") {
                "mcp_tool"
            } else {
                "cli_command"
            },
            "high",
            vec!["agent_tool_supply", "tool_authority"],
            vec!["HLT-024-AGENT-TOOL-SUPPLY-GAP"],
            vec!["security"],
        ));
    }

    if surfaces.is_empty() {
        surfaces.push(surface(
            catalog,
            path,
            &path_symbol(path),
            "business_invariant",
            "medium",
            vec!["changed_behavior"],
            vec!["HLT-008-FALSE-GREEN-RISK"],
            vec!["proofmark-rust"],
        ));
    }

    Ok(surfaces)
}

fn surface(
    catalog: &Catalog,
    path: &str,
    symbol: &str,
    surface_type: &str,
    severity: &str,
    risk_tags: Vec<&str>,
    required_rules: Vec<&str>,
    required_lanes: Vec<&str>,
) -> ChangedSurface {
    let (owner, owner_route) = catalog.owner_for_path(path);
    let (test_route, proof_lane) = catalog.test_for_path(path);
    let risk_tags = risk_tags
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let required_rules = required_rules
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let required_lanes = required_lanes
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    ChangedSurface {
        surface_id: surface_id(surface_type, path, symbol),
        path: path.into(),
        symbol: symbol.into(),
        surface_type: surface_type.into(),
        severity: severity.into(),
        risk_tags,
        owner,
        owner_route,
        test_route,
        proof_lane,
        required_rules,
        required_lanes,
        repair_tasks: repair_tasks(surface_type, severity),
    }
}

fn obligation_for_surface(
    surface: &ChangedSurface,
    receipts: &[ReceiptEvidence],
) -> ProofObligation {
    let obligation_id = format!(
        "obligation:{}:{}",
        match surface.required_rules.first() {
            Some(rule) => rule.clone(),
            None => "HLT-008-FALSE-GREEN-RISK".into(),
        },
        surface.surface_id
    );
    let mut receipt_paths = Vec::new();
    for receipt in receipts {
        if receipt_satisfies(&obligation_id, surface, receipt) {
            receipt_paths.push(receipt.path.clone());
        }
    }
    receipt_paths.sort();
    receipt_paths.dedup();
    let satisfied = !receipt_paths.is_empty();
    ProofObligation {
        obligation_id,
        surface_id: surface.surface_id.clone(),
        path: surface.path.clone(),
        symbol: surface.symbol.clone(),
        surface_type: surface.surface_type.clone(),
        severity: surface.severity.clone(),
        risk_tags: surface.risk_tags.clone(),
        rule_ids: surface.required_rules.clone(),
        required_lanes: surface.required_lanes.clone(),
        required_receipt_kinds: required_receipt_kinds(surface),
        repair_task: match surface.repair_tasks.first() {
            Some(task) => task.clone(),
            None => "attach a focused proof receipt for the changed surface".into(),
        },
        satisfied,
        status: if satisfied { "satisfied" } else { "missing" }.into(),
        receipt_paths,
    }
}

fn required_receipt_kinds(surface: &ChangedSurface) -> Vec<String> {
    let mut kinds = vec!["proof-receipt".to_string()];
    if surface
        .required_lanes
        .iter()
        .any(|lane| lane == "proofmark-rust")
    {
        kinds.push("proofmark".into());
    }
    if surface
        .risk_tags
        .iter()
        .any(|tag| tag == "negative_proof_required")
    {
        kinds.push("negative-behavior-proof".into());
    }
    kinds
}

fn repair_tasks(surface_type: &str, severity: &str) -> Vec<String> {
    let task = match surface_type {
        "authz_boundary" => {
            "add negative authorization or tenant-isolation proof and attach a proofmark receipt"
        }
        "input_boundary" => {
            "add malformed-input or unsafe-sink negative proof and attach a proofmark receipt"
        }
        "db_migration" => {
            "run migration analysis and document rollback/backfill/lock evidence for destructive SQL"
        }
        "sql_query" => "prove the SQL boundary with migration or adapter evidence",
        "cli_command" | "mcp_tool" => {
            "prove the tool surface with supply-chain review and changed-behavior receipt"
        }
        "rust_public_api" => "prove public API compatibility and changed-line behavior",
        "unsafe_or_process_sink" => {
            "prove the process/filesystem sink with negative input evidence"
        }
        _ => "attach a focused proof receipt for the changed behavior",
    };
    let mut tasks = vec![task.to_string()];
    if matches!(severity, "high" | "critical") {
        tasks.push("do not merge as hard proof until the obligation is satisfied or waived".into());
    }
    tasks
}

fn surface_summary(surfaces: &[ChangedSurface]) -> SurfaceSummary {
    let mut summary = SurfaceSummary {
        changed_surface_count: surfaces.len(),
        high_or_critical_surface_count: surfaces
            .iter()
            .filter(|surface| matches!(surface.severity.as_str(), "high" | "critical"))
            .count(),
        by_surface_type: BTreeMap::new(),
        by_owner: BTreeMap::new(),
        verdict: "pass".into(),
    };
    for surface in surfaces {
        *summary
            .by_surface_type
            .entry(surface.surface_type.clone())
            .or_default() += 1;
        *summary.by_owner.entry(surface.owner.clone()).or_default() += 1;
    }
    if summary.high_or_critical_surface_count > 0 {
        summary.verdict = "review".into();
    }
    summary
}

fn obligation_summary(
    surfaces: &[ChangedSurface],
    obligations: &[ProofObligation],
    mode: ProofBindMode,
) -> ObligationSummary {
    let satisfied = obligations
        .iter()
        .filter(|obligation| obligation.satisfied)
        .count();
    let missing = obligations.len().saturating_sub(satisfied);
    let high_or_critical_missing = obligations
        .iter()
        .filter(|obligation| {
            !obligation.satisfied && matches!(obligation.severity.as_str(), "high" | "critical")
        })
        .count();
    let verdict = if missing == 0 {
        "pass"
    } else if mode == ProofBindMode::Required && high_or_critical_missing > 0 {
        "block"
    } else {
        "review"
    };
    ObligationSummary {
        total: obligations.len(),
        satisfied,
        missing,
        high_or_critical_missing,
        changed_surface_count: surfaces.len(),
        verdict: verdict.into(),
    }
}

pub fn render_markdown(witness: &SurfaceWitness, obligations: &ProofBindObligations) -> String {
    let mut out = String::new();
    out.push_str("# jankurai ProofBind\n\n");
    out.push_str(&format!("- mode: `{}`\n", witness.mode));
    out.push_str(&format!(
        "- changed surfaces: `{}`\n",
        witness.summary.changed_surface_count
    ));
    out.push_str(&format!(
        "- high/critical surfaces: `{}`\n",
        witness.summary.high_or_critical_surface_count
    ));
    out.push_str(&format!(
        "- obligations: total=`{}` satisfied=`{}` missing=`{}` high_or_critical_missing=`{}` verdict=`{}`\n",
        obligations.summary.total,
        obligations.summary.satisfied,
        obligations.summary.missing,
        obligations.summary.high_or_critical_missing,
        obligations.summary.verdict
    ));
    out.push_str("\n## Surfaces\n");
    if witness.surfaces.is_empty() {
        out.push_str("- none\n");
    } else {
        for surface in &witness.surfaces {
            out.push_str(&format!(
                "- `{}` type=`{}` severity=`{}` owner=`{}` rules=`{}` lanes=`{}`\n",
                surface.path,
                surface.surface_type,
                surface.severity,
                surface.owner,
                surface.required_rules.join(","),
                surface.required_lanes.join(",")
            ));
        }
    }
    out.push_str("\n## Missing Obligations\n");
    let mut any = false;
    for obligation in obligations
        .obligations
        .iter()
        .filter(|obligation| !obligation.satisfied)
    {
        any = true;
        out.push_str(&format!(
            "- `{}` `{}` severity=`{}` repair=`{}`\n",
            obligation.path, obligation.surface_type, obligation.severity, obligation.repair_task
        ));
    }
    if !any {
        out.push_str("- none\n");
    }
    out
}

fn rust_public_symbols(text: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("pub ") else {
            continue;
        };
        if rest.starts_with("fn ")
            || rest.starts_with("async fn ")
            || rest.starts_with("struct ")
            || rest.starts_with("enum ")
            || rest.starts_with("trait ")
            || rest.starts_with("mod ")
            || rest.starts_with("type ")
        {
            if let Some(name) = rust_symbol_name(rest) {
                symbols.push(name);
            }
        }
    }
    symbols.sort();
    symbols.dedup();
    symbols.truncate(24);
    symbols
}

fn rust_symbol_name(rest: &str) -> Option<String> {
    let rest = rest.strip_prefix("async ").unwrap_or(rest);
    let mut matched = None;
    for prefix in ["fn ", "struct ", "enum ", "trait ", "mod ", "type "] {
        if let Some(value) = rest.strip_prefix(prefix) {
            matched = Some(value);
            break;
        }
    }
    let rest = matched?;
    let name = rest
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .next()?
        .trim();
    if name.is_empty() {
        None
    } else {
        Some(name.into())
    }
}

fn contains_authz_marker(path: &str, text: &str) -> bool {
    path.contains("auth")
        || text.contains("authorize")
        || text.contains("authorization")
        || text.contains("permission")
        || text.contains("tenant_id")
        || text.contains("owner_id")
        || text.contains("role")
}

fn contains_input_marker(path: &str, text: &str) -> bool {
    path.contains("parser")
        || path.contains("request")
        || text.contains("from_str")
        || text.contains("parse(")
        || text.contains("deserialize")
        || text.contains("inner_html")
        || text.contains(&["select", " * from"].concat())
        || text.contains("format!(\"select")
}

fn contains_process_sink(text: &str) -> bool {
    text.contains("unsafe ")
        || text.contains(&["command", "::new"].concat())
        || text.contains("std::process")
        || text.contains("remove_file")
        || text.contains("remove_dir")
        || text.contains("fs::write")
}

fn contains_destructive_sql(text: &str) -> bool {
    [
        "drop table",
        "drop column",
        "truncate",
        "delete from",
        "alter table",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn is_agent_tool_surface(path: &str, text: &str) -> bool {
    path.starts_with("agent/")
        || path.starts_with(".agents/")
        || path.starts_with(".cursor/")
        || path.starts_with(".github/workflows/")
        || path.contains("mcp")
        || text.contains("mcp")
        || text.contains("tool")
}

fn path_symbol(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("file")
        .to_string()
}

fn surface_id(surface_type: &str, path: &str, symbol: &str) -> String {
    format!(
        "surface:{}:{}:{}",
        sanitize(surface_type),
        sanitize(path),
        sanitize(symbol)
    )
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                ':'
            }
        })
        .collect::<String>()
        .trim_matches(':')
        .to_string()
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
        .filter(|path| !path.starts_with("target/"))
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

fn read_json<T: for<'de> Deserialize<'de>>(path: PathBuf) -> Option<T> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn read_toml<T: for<'de> Deserialize<'de>>(path: PathBuf) -> Option<T> {
    let text = fs::read_to_string(path).ok()?;
    toml::from_str(&text).ok()
}

fn prefix_matches(prefix: &str, path: &str) -> bool {
    let prefix = prefix.trim().trim_matches('/');
    path == prefix || path.starts_with(&format!("{prefix}/"))
}

fn normalize_prefix(prefix: &str) -> String {
    prefix.trim().trim_end_matches('/').to_string()
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

fn unix_seconds() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Default::default())
        .as_secs()
        .to_string()
}
