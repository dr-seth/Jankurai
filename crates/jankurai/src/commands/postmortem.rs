use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

use crate::validation::{self, ArtifactSchema};

#[derive(Debug, Clone)]
pub struct PostmortemRecordArgs {
    pub repo: PathBuf,
    pub input: String,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PostmortemListArgs {
    pub repo: PathBuf,
    pub root: String,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PostmortemShowArgs {
    pub repo: PathBuf,
    pub root: String,
    pub postmortem_id: String,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PostmortemReadArgs {
    pub repo: PathBuf,
    pub path: String,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureMode {
    AspirationalSpec,
    EnvPrerequisite,
    InteropRuntime,
    EquivalenceGap,
    CutoverRollback,
    PerfRegression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmortemEntry {
    pub schema_version: String,
    pub postmortem_id: String,
    pub title: String,
    pub owner: String,
    pub failure_mode: FailureMode,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocker_type: Option<FailureMode>,
    pub summary: String,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub actions: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct PostmortemEntrySummary {
    postmortem_id: String,
    title: String,
    owner: String,
    failure_mode: FailureMode,
    severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocker_type: Option<FailureMode>,
    summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct PostmortemRecordReport {
    schema_version: String,
    command: String,
    status: String,
    repo: String,
    root: String,
    record_path: String,
    record: PostmortemEntry,
    notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct PostmortemListReport {
    schema_version: String,
    command: String,
    status: String,
    repo: String,
    root: String,
    records_total: usize,
    records: Vec<PostmortemEntrySummary>,
    notes: Vec<String>,
}

/// The six valid `failure_mode` enum values (QO postmortem schema).
const FAILURE_MODE_VALUES: &[&str] = &[
    "aspirational-spec",
    "env-prerequisite",
    "interop-runtime",
    "equivalence-gap",
    "cutover-rollback",
    "perf-regression",
];

/// The three valid `outcome` enum values.
const OUTCOME_VALUES: &[&str] = &["blocked", "rolled_back", "shipped_with_caveats"];

/// A validated QO-schema postmortem document (`[postmortem]`/`[evidence]`/
/// `[lessons]`/`[follow_ups]`). Distinct from the legacy `PostmortemEntry`
/// (schema_version/postmortem_id/title/owner) which `list`/`show`/`read`
/// still use — kept side-by-side for back-compat.
pub struct PostmortemDoc {
    pub date: String,
    pub slice: String,
    pub failure_mode: String,
    pub outcome: String,
    /// The whole parsed table, used for the canonical round-trip hash.
    value: toml::Value,
}

/// Derive the on-disk id: `<YYYY-MM>-<slice with trailing "-runner" trimmed>`.
/// `date="2026-05-13"`, `slice="phase-3d-atlas-runner"` → `2026-05-phase-3d-atlas`.
pub fn derive_postmortem_id(doc: &PostmortemDoc) -> String {
    let ym = doc.date.get(..7).unwrap_or(&doc.date);
    let slug = doc
        .slice
        .strip_suffix("-runner")
        .unwrap_or(&doc.slice)
        .to_string();
    format!("{ym}-{slug}")
}

/// True iff `text` parses as a QO-schema postmortem (a top-level
/// `[postmortem]` table). The legacy `PostmortemEntry` reader cannot parse
/// these, so `list`/`show`/`read` must skip/reject them rather than fail.
pub fn is_qo_schema_text(text: &str) -> bool {
    toml::from_str::<toml::Value>(text)
        .ok()
        .and_then(|v| v.get("postmortem").map(|p| p.is_table()))
        .unwrap_or(false)
}

/// The derived id becomes a filename under `.jankurai/postmortems/`. It is
/// built from attacker-controllable `slice`/`date` fields, so it must be a
/// single safe path component — no separators, no `..`, no root/prefix.
fn ensure_safe_record_id(id: &str) -> Result<()> {
    let mut comps = Path::new(id).components();
    match (comps.next(), comps.next()) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => bail!(
            "derived postmortem id `{id}` is not a safe filename component \
             (path separators or traversal in slice/date)"
        ),
    }
}

/// Canonical hash: re-serialize the parsed TOML with stable key ordering and
/// sha256 it. Comment/whitespace differences in the source do not affect it,
/// so round-trip equality is content-equivalence, not byte-equality.
pub fn canonical_hash(doc: &PostmortemDoc) -> String {
    use sha2::{Digest, Sha256};
    // serde_json with sorted keys gives a deterministic canonical form.
    let json = canonical_json(&doc.value);
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn canonical_json(value: &toml::Value) -> String {
    // toml::Value -> serde_json::Value, then serialize with BTree-sorted keys.
    fn conv(v: &toml::Value) -> serde_json::Value {
        match v {
            toml::Value::String(s) => serde_json::Value::String(s.clone()),
            toml::Value::Integer(i) => serde_json::Value::from(*i),
            toml::Value::Float(f) => serde_json::Value::from(*f),
            toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
            toml::Value::Datetime(d) => serde_json::Value::String(d.to_string()),
            toml::Value::Array(a) => serde_json::Value::Array(a.iter().map(conv).collect()),
            toml::Value::Table(t) => {
                let mut map = serde_json::Map::new();
                let mut keys: Vec<&String> = t.keys().collect();
                keys.sort();
                for k in keys {
                    map.insert(k.clone(), conv(&t[k]));
                }
                serde_json::Value::Object(map)
            }
        }
    }
    serde_json::to_string(&conv(value)).unwrap_or_default()
}

/// Validate the QO postmortem schema with precise, spec-mandated
/// diagnostics. Parses with a real TOML parser first (Rule 9: a commented
/// `# failure_mode = ...` is *absent*, never satisfies the required check).
/// Returns `Err(diagnostic)` — the string is printed verbatim then exit 1.
pub fn validate_postmortem_doc(text: &str) -> std::result::Result<PostmortemDoc, String> {
    let value: toml::Value =
        toml::from_str(text).map_err(|e| format!("postmortem TOML parse error: {e}"))?;
    let pm = value
        .get("postmortem")
        .and_then(|v| v.as_table())
        .ok_or_else(|| "missing required section: [postmortem]".to_string())?;

    let get_str = |k: &str| pm.get(k).and_then(|v| v.as_str()).map(str::to_string);

    let failure_mode = match get_str("failure_mode") {
        None => return Err("missing required field: failure_mode".to_string()),
        Some(v) if !FAILURE_MODE_VALUES.contains(&v.as_str()) => {
            return Err(format!(
                "invalid enum value for failure_mode: `{v}`\n  valid values: {}",
                FAILURE_MODE_VALUES.join(", ")
            ));
        }
        Some(v) => v,
    };

    let outcome = match get_str("outcome") {
        None => return Err("missing required field: outcome".to_string()),
        Some(v) if !OUTCOME_VALUES.contains(&v.as_str()) => {
            let suggestion = OUTCOME_VALUES
                .iter()
                .find(|cand| v.starts_with(*cand) || cand.starts_with(v.as_str()))
                .map(|c| format!("\n  did you mean: {c}"))
                .unwrap_or_default();
            return Err(format!("invalid enum value for outcome: `{v}`{suggestion}"));
        }
        Some(v) => v,
    };

    // Accept both a quoted string and an idiomatic unquoted TOML date
    // literal (`date = 2026-05-13`), which the parser yields as a Datetime.
    // `canonical_json` already stringifies Datetime, so the hash stays stable.
    let date = pm
        .get("date")
        .and_then(|v| {
            v.as_str()
                .map(str::to_string)
                .or_else(|| v.as_datetime().map(|d| d.to_string()))
        })
        .ok_or_else(|| "missing required field: date".to_string())?;
    let slice = get_str("slice").ok_or_else(|| "missing required field: slice".to_string())?;

    // `[lessons]` must be a structurally present section (commented = absent).
    if value.get("lessons").and_then(|v| v.as_table()).is_none() {
        return Err("missing required section: [lessons]".to_string());
    }

    Ok(PostmortemDoc {
        date,
        slice,
        failure_mode,
        outcome,
        value,
    })
}

pub fn run_record(args: PostmortemRecordArgs) -> Result<()> {
    let repo = canonicalize_repo(&args.repo)?;
    let input_path = resolve_repo_relative_existing(&repo, &args.input)?;
    let input_text = fs::read_to_string(&input_path)
        .with_context(|| format!("read {}", input_path.display()))?;

    // Schema dispatch: a top-level `[postmortem]` table is the QO slice
    // postmortem schema (ARY-2032); anything else is the legacy
    // schema_version/postmortem_id entry, kept for back-compat.
    if is_qo_schema_text(&input_text) {
        return run_record_qo(&repo, &input_text, &args);
    }
    run_record_legacy(&repo, input_text, &args)
}

fn run_record_legacy(repo: &Path, input_text: String, args: &PostmortemRecordArgs) -> Result<()> {
    let entry = parse_entry(repo, &input_text)?;
    let record_path = match args.out.as_deref() {
        Some(out) => PathBuf::from(out),
        None => {
            ensure_safe_record_id(&entry.postmortem_id)?;
            postmortem_root(repo).join(format!("{}.toml", entry.postmortem_id))
        }
    };
    if let Some(parent) = record_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&record_path, &input_text)?;
    let report = PostmortemRecordReport {
        schema_version: entry.schema_version.clone(),
        command: "jankurai postmortem record".to_string(),
        status: "complete".to_string(),
        repo: repo.display().to_string(),
        root: postmortem_root(repo).display().to_string(),
        record_path: record_path.display().to_string(),
        record: entry,
        notes: vec!["postmortem record written only when explicitly requested".to_string()],
    };
    if let Some(path) = args.md.as_deref() {
        crate::render::write_markdown(path, &render_record_markdown(&report))?;
    }
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn run_record_qo(repo: &Path, input_text: &str, args: &PostmortemRecordArgs) -> Result<()> {
    let doc = match validate_postmortem_doc(input_text) {
        Ok(doc) => doc,
        Err(diagnostic) => {
            eprintln!("{diagnostic}");
            bail!("postmortem validation failed");
        }
    };

    let id = derive_postmortem_id(&doc);
    let record_path = match args.out.as_deref() {
        Some(out) => PathBuf::from(out),
        None => {
            // No explicit --out: the id is interpolated straight into the
            // on-disk path, so reject any separator/traversal before write.
            ensure_safe_record_id(&id)?;
            postmortem_root(repo).join(format!("{id}.toml"))
        }
    };
    if let Some(parent) = record_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&record_path, input_text)?;

    // Rule 9 round-trip: re-read, re-validate, canonical-hash must match.
    let written = fs::read_to_string(&record_path)
        .with_context(|| format!("re-read {}", record_path.display()))?;
    let reparsed = validate_postmortem_doc(&written)
        .map_err(|d| anyhow::anyhow!("round-trip re-validation failed: {d}"))?;
    let h_in = canonical_hash(&doc);
    let h_out = canonical_hash(&reparsed);
    if h_in != h_out {
        bail!("round-trip canonical-hash mismatch: {h_in} != {h_out}");
    }

    let rel = record_path
        .strip_prefix(repo)
        .unwrap_or(&record_path)
        .display()
        .to_string();
    println!("written to {rel}");
    println!("canonical-hash: {h_in}");
    println!(
        "postmortem: {} / {} / {}",
        doc.failure_mode, doc.outcome, id
    );
    if let Some(path) = args.md.as_deref() {
        crate::render::write_markdown(
            path,
            &format!(
                "# postmortem record\n\n- id: `{id}`\n- failure_mode: `{}`\n- outcome: `{}`\n- canonical-hash: `{h_in}`\n",
                doc.failure_mode, doc.outcome
            ),
        )?;
    }
    Ok(())
}

pub fn run_list(args: PostmortemListArgs) -> Result<()> {
    let repo = canonicalize_repo(&args.repo)?;
    let root = resolve_repo_relative(&repo, &args.root)?;
    let records = collect_entries(&repo, &root)?;
    let report = PostmortemListReport {
        schema_version: "1.0.0".to_string(),
        command: "jankurai postmortem list".to_string(),
        status: "complete".to_string(),
        repo: repo.display().to_string(),
        root: root.display().to_string(),
        records_total: records.len(),
        records: records.into_iter().map(summary_from_entry).collect(),
        notes: vec![
            "list and show are read-only views over durable postmortem records".to_string(),
        ],
    };
    write_report(
        &report,
        args.out.as_deref(),
        args.md.as_deref(),
        &render_list_markdown(&report),
    )?;
    Ok(())
}

pub fn run_show(args: PostmortemShowArgs) -> Result<()> {
    let repo = canonicalize_repo(&args.repo)?;
    let root = resolve_repo_relative(&repo, &args.root)?;
    let record_path = root.join(format!("{}.toml", args.postmortem_id));
    let record = read_entry(&repo, &record_path)?;
    let report = PostmortemRecordReport {
        schema_version: record.schema_version.clone(),
        command: "jankurai postmortem show".to_string(),
        status: "complete".to_string(),
        repo: repo.display().to_string(),
        root: root.display().to_string(),
        record_path: record_path.display().to_string(),
        record,
        notes: vec!["show reads an existing durable record without mutating it".to_string()],
    };
    write_report(
        &report,
        args.out.as_deref(),
        args.md.as_deref(),
        &render_record_markdown(&report),
    )?;
    Ok(())
}

pub fn run_read(args: PostmortemReadArgs) -> Result<()> {
    let repo = canonicalize_repo(&args.repo)?;
    let record_path = resolve_repo_relative_existing(&repo, &args.path)?;
    let record = read_entry(&repo, &record_path)?;
    let report = PostmortemRecordReport {
        schema_version: record.schema_version.clone(),
        command: "jankurai postmortem read".to_string(),
        status: "complete".to_string(),
        repo: repo.display().to_string(),
        root: postmortem_root(&repo).display().to_string(),
        record_path: record_path.display().to_string(),
        record,
        notes: vec!["read inspects an arbitrary postmortem record without writing".to_string()],
    };
    write_report(
        &report,
        args.out.as_deref(),
        args.md.as_deref(),
        &render_record_markdown(&report),
    )?;
    Ok(())
}

fn write_report<T: Serialize>(
    report: &T,
    out: Option<&str>,
    md: Option<&str>,
    rendered_md: &str,
) -> Result<()> {
    if let Some(path) = out {
        crate::render::write_json(path, &serde_json::to_string_pretty(report)?)?;
    } else {
        println!("{}", serde_json::to_string_pretty(report)?);
    }
    if let Some(path) = md {
        crate::render::write_markdown(path, rendered_md)?;
    }
    Ok(())
}

fn render_record_markdown(report: &PostmortemRecordReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Postmortem");
    let _ = writeln!(out);
    let _ = writeln!(out, "- command: `{}`", report.command);
    let _ = writeln!(out, "- repo: `{}`", report.repo);
    let _ = writeln!(out, "- root: `{}`", report.root);
    let _ = writeln!(out, "- record path: `{}`", report.record_path);
    let record = &report.record;
    let _ = writeln!(out, "- id: `{}`", record.postmortem_id);
    let _ = writeln!(out, "- title: {}", record.title);
    let _ = writeln!(out, "- owner: `{}`", record.owner);
    let _ = writeln!(
        out,
        "- failure mode: `{}`",
        failure_mode_label(&record.failure_mode)
    );
    let _ = writeln!(out, "- severity: `{}`", record.severity);
    if let Some(blocker_type) = &record.blocker_type {
        let _ = writeln!(
            out,
            "- blocker type: `{}`",
            failure_mode_label(blocker_type)
        );
    }
    let _ = writeln!(out, "- summary: {}", record.summary);
    if !record.evidence.is_empty() {
        let _ = writeln!(out, "- evidence: `{}`", record.evidence.join(", "));
    }
    if !record.actions.is_empty() {
        let _ = writeln!(out, "- actions: `{}`", record.actions.join(", "));
    }
    if !record.notes.is_empty() {
        let _ = writeln!(out, "- notes: `{}`", record.notes.join(", "));
    }
    out
}

fn render_list_markdown(report: &PostmortemListReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Postmortem List");
    let _ = writeln!(out);
    let _ = writeln!(out, "- command: `{}`", report.command);
    let _ = writeln!(out, "- repo: `{}`", report.repo);
    let _ = writeln!(out, "- root: `{}`", report.root);
    let _ = writeln!(out, "- records: `{}`", report.records_total);
    let _ = writeln!(out);
    for record in &report.records {
        let _ = writeln!(
            out,
            "- `{}` `{}` `{}` [{}] - {}",
            record.postmortem_id,
            record.owner,
            failure_mode_label(&record.failure_mode),
            record.severity,
            record.title
        );
    }
    out
}

fn summary_from_entry(entry: PostmortemEntry) -> PostmortemEntrySummary {
    PostmortemEntrySummary {
        postmortem_id: entry.postmortem_id,
        title: entry.title,
        owner: entry.owner,
        failure_mode: entry.failure_mode,
        severity: entry.severity,
        blocker_type: entry.blocker_type,
        summary: entry.summary,
        source: entry.source,
    }
}

fn parse_entry(repo: &Path, text: &str) -> Result<PostmortemEntry> {
    let entry: PostmortemEntry = toml::from_str(text).context("parse postmortem TOML")?;
    validation::validate_serializable(repo, ArtifactSchema::Postmortem, &entry)?;
    if entry.postmortem_id.trim().is_empty() {
        bail!("postmortem_id must not be empty");
    }
    if entry.title.trim().is_empty() {
        bail!("title must not be empty");
    }
    if entry.owner.trim().is_empty() {
        bail!("owner must not be empty");
    }
    if entry.summary.trim().is_empty() {
        bail!("summary must not be empty");
    }
    Ok(entry)
}

fn read_entry(repo: &Path, path: &Path) -> Result<PostmortemEntry> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if is_qo_schema_text(&text) {
        bail!(
            "{} is a QO-schema postmortem ([postmortem] table); legacy \
             show/read operate on schema_version entries only",
            path.display()
        );
    }
    parse_entry(repo, &text)
}

fn collect_entries(repo: &Path, root: &Path) -> Result<Vec<PostmortemEntry>> {
    let mut entries = Vec::new();
    if !root.exists() {
        return Ok(entries);
    }
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !entry.file_type().is_dir() || !is_skipped_dir(entry.path()))
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if entry.file_type().is_dir() || path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        // QO-schema records live in the same root but are not legacy
        // entries; skip them so one `postmortem record` cannot poison the
        // entire legacy listing.
        if is_qo_schema_text(&text) {
            continue;
        }
        entries.push(parse_entry(repo, &text)?);
    }
    entries.sort_by(|left, right| left.postmortem_id.cmp(&right.postmortem_id));
    Ok(entries)
}

fn postmortem_root(repo: &Path) -> PathBuf {
    repo.join(".jankurai/postmortems")
}

fn is_skipped_dir(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(part) => matches!(part.to_string_lossy().as_ref(), ".git" | "target"),
        _ => false,
    })
}

fn canonicalize_repo(repo: &Path) -> Result<PathBuf> {
    let canonical =
        fs::canonicalize(repo).with_context(|| format!("canonicalize {}", repo.display()))?;
    if !canonical.is_dir() {
        bail!("{} is not a directory", repo.display());
    }
    Ok(canonical)
}

fn resolve_repo_relative_existing(repo: &Path, rel: &str) -> Result<PathBuf> {
    let path = resolve_repo_relative(repo, rel)?;
    let canonical =
        fs::canonicalize(&path).with_context(|| format!("resolve {}", path.display()))?;
    if !canonical.starts_with(repo) {
        bail!("path escapes repo root: {}", path.display());
    }
    Ok(canonical)
}

fn resolve_repo_relative(repo: &Path, rel: &str) -> Result<PathBuf> {
    let candidate = PathBuf::from(rel);
    let abs = if candidate.is_absolute() {
        candidate
    } else {
        repo.join(candidate)
    };
    Ok(abs)
}

fn failure_mode_label(mode: &FailureMode) -> &'static str {
    match mode {
        FailureMode::AspirationalSpec => "aspirational-spec",
        FailureMode::EnvPrerequisite => "env-prerequisite",
        FailureMode::InteropRuntime => "interop-runtime",
        FailureMode::EquivalenceGap => "equivalence-gap",
        FailureMode::CutoverRollback => "cutover-rollback",
        FailureMode::PerfRegression => "perf-regression",
    }
}

pub fn parse_failure_mode(value: &str) -> Option<FailureMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "aspirational-spec" => Some(FailureMode::AspirationalSpec),
        "env-prerequisite" => Some(FailureMode::EnvPrerequisite),
        "interop-runtime" => Some(FailureMode::InteropRuntime),
        "equivalence-gap" => Some(FailureMode::EquivalenceGap),
        "cutover-rollback" => Some(FailureMode::CutoverRollback),
        "perf-regression" => Some(FailureMode::PerfRegression),
        _ => None,
    }
}
