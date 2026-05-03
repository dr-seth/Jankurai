use crate::commands::context_data::{push_unique, RepoCatalog};
use crate::model::{ProofReceipt, RuleCoverage, STANDARD_VERSION};
use crate::validation::{self, ArtifactSchema};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ProofPlanArgs {
    pub repo: PathBuf,
    pub changed: Vec<PathBuf>,
    pub changed_from: Option<String>,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProveArgs {
    pub repo: PathBuf,
    pub plan: Option<String>,
    pub changed: Vec<PathBuf>,
    pub changed_from: Option<String>,
    pub plan_out: String,
    pub plan_md: String,
    pub out_dir: String,
    pub evidence_index: String,
    pub continue_on_error: bool,
    /// When set with `JANKURAI_ALLOW_UNSIGNED_PROOF_COMMANDS=1`, run commands not in proof-lanes/test-map.
    pub allow_unsigned_commands: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofPlan {
    pub schema_version: String,
    pub standard_version: String,
    pub repo_root: String,
    pub git_head: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_ref: Option<String>,
    pub changed_paths: Vec<String>,
    pub matched_owner_map: Vec<String>,
    pub matched_test_map: Vec<String>,
    pub required_lanes: Vec<String>,
    pub optional_lanes: Vec<String>,
    pub skipped_lanes: Vec<String>,
    pub commands: Vec<String>,
    pub expected_artifacts: Vec<String>,
    pub risk_notes: Vec<String>,
    pub human_approval_requirements: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub planned_runs: Vec<PlannedRun>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped_lane_entries: Vec<SkippedLaneEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedLaneEntry {
    pub lane: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedRun {
    pub lane: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub changed_paths: Vec<String>,
    pub artifacts: Vec<String>,
    pub residual_risk: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofEvidenceIndex {
    pub schema_version: String,
    pub generated_at: String,
    pub repo_root: String,
    pub git_head: String,
    pub plan_path: String,
    pub receipt_dir: String,
    pub log_dir: String,
    pub commands: Vec<String>,
    pub receipts: Vec<String>,
    pub logs: Vec<String>,
    pub failed_receipts: Vec<String>,
    pub skipped_lanes: Vec<String>,
    pub risk_notes: Vec<String>,
    pub human_approval_requirements: Vec<String>,
    pub changed_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ux_qa_report_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ux_qa_report_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_evidence_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_score_json_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sarif_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_step_summary_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repair_queue_jsonl_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundaries_manifest_path: Option<String>,
}

pub fn run_lane(args: ProofPlanArgs) -> Result<()> {
    let plan = build_proof_plan(&args.repo, &args.changed, args.changed_from.as_deref())?;
    write_plan(&args.repo, &plan, args.out.as_deref(), args.md.as_deref())?;
    Ok(())
}

pub fn run_proof(args: ProofPlanArgs) -> Result<()> {
    run_lane(args)
}

pub fn run_prove(args: ProveArgs) -> Result<()> {
    let has_changed_input = !args.changed.is_empty() || args.changed_from.is_some();
    if args.plan.is_some() && has_changed_input {
        anyhow::bail!("use either --plan or --changed/--changed-from, not both");
    }

    let (plan, plan_path_str) = if let Some(plan_path) = args.plan.as_deref() {
        (load_proof_plan(&args.repo, plan_path)?, plan_path.to_string())
    } else if has_changed_input {
        if args.plan_out == "-" {
            anyhow::bail!("--plan-out must be a file path when prove builds a plan");
        }
        let plan = build_proof_plan(&args.repo, &args.changed, args.changed_from.as_deref())?;
        write_plan(
            &args.repo,
            &plan,
            Some(args.plan_out.as_str()),
            Some(args.plan_md.as_str()),
        )?;
        let persisted_plan = load_proof_plan(&args.repo, args.plan_out.as_str())?;
        (persisted_plan, args.plan_out.clone())
    } else {
        anyhow::bail!("provide --plan, --changed, or --changed-from");
    };

    execute_proof_plan(args, plan, plan_path_str)
}

fn load_proof_plan(repo: &Path, plan_path: &str) -> Result<ProofPlan> {
    let plan_text =
        fs::read_to_string(plan_path).with_context(|| format!("read proof plan {plan_path}"))?;
    let plan_json: Value = serde_json::from_str(&plan_text)
        .with_context(|| format!("parse proof plan {plan_path}"))?;
    validation::validate_value(repo, ArtifactSchema::ProofPlan, &plan_json)?;
    Ok(serde_json::from_value(plan_json)?)
}

fn execute_proof_plan(args: ProveArgs, plan: ProofPlan, plan_path_str: String) -> Result<()> {
    let receipt_dir = PathBuf::from(&args.out_dir);
    let log_dir = args.repo.join("target/jankurai/logs");
    fs::create_dir_all(&receipt_dir)?;
    fs::create_dir_all(&log_dir)?;
    let evidence_index_path = PathBuf::from(&args.evidence_index);
    if let Some(parent) = evidence_index_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let runs = if plan.planned_runs.is_empty() {
        fallback_runs_from_plan(&plan)
    } else {
        plan.planned_runs.clone()
    };

    ensure_planned_commands_allowed(&args.repo, &runs, args.allow_unsigned_commands)?;

    let mut receipts = Vec::new();
    let mut receipt_paths = Vec::new();
    let mut failed_receipts = Vec::new();
    let mut failure: Option<anyhow::Error> = None;

    for (index, run) in runs.iter().enumerate() {
        let receipt = execute_run(
            &args.repo,
            &receipt_dir,
            &log_dir,
            index,
            run,
            plan_path_str.as_str(),
        )?;
        let receipt_name = receipt_file_name(index, &run.lane, &run.command);
        let receipt_path = receipt_dir.join(receipt_name);
        validation::write_json(
            &args.repo,
            ArtifactSchema::ProofReceipt,
            receipt_path.to_string_lossy().as_ref(),
            &receipt,
        )?;
        receipt_paths.push(display_relative(&args.repo, &receipt_path));
        if receipt.exit_code != 0 {
            failed_receipts.push(display_relative(&args.repo, receipt_path.as_path()));
            if args.continue_on_error {
                receipts.push(receipt);
                continue;
            }
            failure = Some(anyhow::anyhow!(
                "proof command `{}` failed in lane `{}` with exit {}",
                run.command,
                run.lane,
                receipt.exit_code
            ));
            receipts.push(receipt);
            break;
        }
        receipts.push(receipt);
    }

    let evidence = ProofEvidenceIndex {
        schema_version: "1.2.0".to_string(),
        generated_at: now_string(),
        repo_root: args.repo.display().to_string(),
        git_head: git_head(&args.repo).unwrap_or_else(|_| "unknown".to_string()),
        plan_path: plan_path_str.clone(),
        receipt_dir: receipt_dir.display().to_string(),
        log_dir: log_dir.display().to_string(),
        commands: runs.iter().map(|run| run.command.clone()).collect(),
        receipts: receipt_paths,
        logs: receipts
            .iter()
            .filter_map(|receipt| receipt.log_path.clone())
            .collect(),
        failed_receipts,
        skipped_lanes: plan.skipped_lanes.clone(),
        risk_notes: plan.risk_notes.clone(),
        human_approval_requirements: plan.human_approval_requirements.clone(),
        changed_paths: plan.changed_paths.clone(),
        ux_qa_report_path: optional_repo_relative_existing(
            &args.repo,
            "target/jankurai/ux-qa.json",
        ),
        ux_qa_report_digest: sha256_file_if_exists(&args.repo, "target/jankurai/ux-qa.json"),
        security_evidence_path: optional_repo_relative_existing(
            &args.repo,
            "target/jankurai/security/evidence.json",
        ),
        repo_score_json_path: optional_repo_relative_existing(&args.repo, "agent/repo-score.json"),
        sarif_path: optional_repo_relative_existing(&args.repo, "target/jankurai/jankurai.sarif"),
        github_step_summary_path: optional_repo_relative_existing(
            &args.repo,
            "target/jankurai/summary.md",
        ),
        repair_queue_jsonl_path: optional_repo_relative_existing(
            &args.repo,
            "target/jankurai/repair-queue.jsonl",
        ),
        boundaries_manifest_path: optional_repo_relative_existing(
            &args.repo,
            "agent/boundaries.toml",
        ),
    };
    write_evidence_index(&args.repo, &evidence_index_path, &evidence)?;

    if runs.is_empty() {
        anyhow::bail!(
            "proof plan contains no runnable proof commands; update agent/test-map.json \
             or provide a persisted plan with planned_runs"
        );
    }

    if let Some(error) = failure {
        return Err(error);
    }

    if receipts.iter().any(|receipt| receipt.exit_code != 0) {
        anyhow::bail!("one or more proof commands failed");
    }

    Ok(())
}

pub fn build_proof_plan(
    repo: &Path,
    changed: &[PathBuf],
    changed_from: Option<&str>,
) -> Result<ProofPlan> {
    let catalog = RepoCatalog::load(repo)?;
    let changed_paths = normalize_changed_paths(repo, changed, changed_from)?;
    if changed_paths.is_empty() {
        anyhow::bail!("provide at least one --changed path or --changed-from ref");
    }

    let mut matched_owner_map = Vec::new();
    let mut matched_test_map = Vec::new();
    let mut risk_notes = Vec::new();
    let mut human_approval_requirements = BTreeSet::new();
    let mut required_lanes = Vec::new();
    let mut optional_lanes = catalog.proof_lane_names();
    let mut skipped_lanes = BTreeSet::new();
    let mut planned_runs: BTreeMap<String, PlannedRun> = BTreeMap::new();
    let lane_names_by_command = lane_names_by_command(&catalog);

    for path in &changed_paths {
        if let Some(prefix) = catalog.owner_prefix_for_path(path) {
            push_unique(&mut matched_owner_map, prefix);
        } else {
            risk_notes.push(format!("path `{path}` has no owner-map route"));
            human_approval_requirements
                .insert("review unmapped path coverage before merge".to_string());
            skipped_lanes.insert("full".to_string());
        }
        if let Some((prefix, spec)) = catalog.test_route_for_path(path) {
            push_unique(&mut matched_test_map, prefix.clone());
            let lane_label = lane_names_by_command
                .get(&spec.command)
                .cloned()
                .unwrap_or_else(|| format!("test-map:{}", prefix));
            if lane_names_by_command.contains_key(&spec.command) {
                push_unique(&mut required_lanes, lane_label.clone());
            } else {
                push_unique(&mut required_lanes, lane_label.clone());
                risk_notes.push(format!(
                    "test command `{}` for `{}` is not backed by a named proof lane",
                    spec.command, path
                ));
                human_approval_requirements
                    .insert("approve proof commands that are not named lanes".to_string());
                skipped_lanes.insert("full".to_string());
            }
            let owner = catalog.owner_for_path(path).map(|owner| owner.to_string());
            let route_note = route_risk_note(path);
            let entry = planned_runs
                .entry(spec.command.clone())
                .or_insert_with(|| PlannedRun {
                    lane: lane_label,
                    command: spec.command.clone(),
                    owner: owner.clone(),
                    changed_paths: vec![path.clone()],
                    artifacts: vec![
                        "target/jankurai/logs/*.log".to_string(),
                        "target/jankurai/proof-receipts/*.json".to_string(),
                    ],
                    residual_risk: vec![route_note.clone()],
                    skipped_reason: None,
                });
            push_unique(&mut entry.changed_paths, path.clone());
            push_unique(&mut entry.residual_risk, route_note);
            if entry.owner.is_none() {
                entry.owner = owner;
            }
            continue;
        }
        risk_notes.push(format!("path `{path}` has no test-map proof route"));
        human_approval_requirements.insert("approve proof coverage for unmapped paths".to_string());
        skipped_lanes.insert("full".to_string());
    }

    let used_real_lanes: BTreeSet<String> = required_lanes
        .iter()
        .filter(|lane| {
            catalog
                .proof_lanes
                .iter()
                .any(|candidate| candidate.name == **lane)
        })
        .cloned()
        .collect();
    for lane in &catalog.proof_lanes {
        if !used_real_lanes.contains(&lane.name) {
            skipped_lanes.insert(lane.name.clone());
        }
    }
    optional_lanes.retain(|lane| !required_lanes.iter().any(|required| required == lane));
    let mut commands = Vec::new();
    let mut planned_runs = planned_runs.into_values().collect::<Vec<_>>();
    planned_runs.sort_by(|left, right| {
        left.lane
            .cmp(&right.lane)
            .then(left.command.cmp(&right.command))
    });
    for run in &planned_runs {
        push_unique(&mut commands, run.command.clone());
    }
    let expected_artifacts = vec![
        "target/jankurai/proof-plan.json".to_string(),
        "target/jankurai/proof-plan.md".to_string(),
        "target/jankurai/proof-receipts/*.json".to_string(),
        "target/jankurai/logs/*.log".to_string(),
        "target/jankurai/evidence-index.json".to_string(),
    ];
    let skipped_lanes_vec: Vec<String> = skipped_lanes.iter().cloned().collect();
    let skipped_lane_entries: Vec<SkippedLaneEntry> = skipped_lanes_vec
        .iter()
        .map(|lane| SkippedLaneEntry {
            lane: lane.clone(),
            reason: skipped_lane_reason(lane.as_str(), &risk_notes),
        })
        .collect();
    let git_head = git_head(repo).unwrap_or_else(|_| "unknown".to_string());
    Ok(ProofPlan {
        schema_version: "1.0.0".to_string(),
        standard_version: STANDARD_VERSION.to_string(),
        repo_root: repo.display().to_string(),
        git_head,
        base_ref: changed_from.map(|value| value.to_string()),
        changed_paths,
        matched_owner_map,
        matched_test_map,
        required_lanes,
        optional_lanes,
        skipped_lanes: skipped_lanes_vec,
        commands,
        expected_artifacts,
        risk_notes,
        human_approval_requirements: human_approval_requirements.into_iter().collect(),
        planned_runs,
        skipped_lane_entries,
    })
}

fn ensure_planned_commands_allowed(
    repo: &Path,
    runs: &[PlannedRun],
    allow_unsigned: bool,
) -> Result<()> {
    let env_ok = std::env::var("JANKURAI_ALLOW_UNSIGNED_PROOF_COMMANDS")
        .map(|v| v == "1")
        .unwrap_or(false);
    if allow_unsigned && env_ok {
        return Ok(());
    }
    let catalog = RepoCatalog::load(repo)?;
    let allow = catalog.allowed_proof_commands();
    for run in runs {
        let normalized = RepoCatalog::normalize_proof_command(&run.command);
        if !allow.contains(&normalized) {
            anyhow::bail!(
                "proof command not in agent proof-lanes or test-map allowlist: `{}`\n\
                 hint: entries must match after trimming and collapsing whitespace; \
                 or pass --allow-unsigned-commands with JANKURAI_ALLOW_UNSIGNED_PROOF_COMMANDS=1",
                run.command
            );
        }
    }
    Ok(())
}

fn skipped_lane_reason(lane: &str, risk_notes: &[String]) -> String {
    if lane == "full" {
        if risk_notes
            .iter()
            .any(|n| n.contains("no test-map proof route"))
        {
            "changed path lacks test-map proof route".into()
        } else if risk_notes.iter().any(|n| n.contains("no owner-map route")) {
            "changed path lacks owner-map route".into()
        } else if risk_notes
            .iter()
            .any(|n| n.contains("not backed by a named proof lane"))
        {
            "test-map command not listed in proof-lanes.toml".into()
        } else {
            "merge-grade proof lane blocked for current routing".into()
        }
    } else {
        "not required for current changed paths".into()
    }
}

fn proof_run_id(
    plan_path: &str,
    index: usize,
    lane: &str,
    command: &str,
    started_secs: u64,
) -> String {
    let raw = format!("{plan_path}|{index}|{lane}|{command}|{started_secs}");
    let digest = Sha256::digest(raw.as_bytes());
    digest
        .iter()
        .take(8)
        .map(|b| format!("{:02x}", b))
        .collect()
}

fn write_plan(repo: &Path, plan: &ProofPlan, out: Option<&str>, md: Option<&str>) -> Result<()> {
    if let Some(path) = out {
        validation::write_json(repo, ArtifactSchema::ProofPlan, path, plan)?;
    } else {
        println!("{}", serde_json::to_string_pretty(plan)?);
    }
    if let Some(path) = md {
        crate::render::write_markdown(path, &render_markdown(plan))?;
    }
    Ok(())
}

fn write_evidence_index(repo: &Path, path: &Path, evidence: &ProofEvidenceIndex) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    validation::write_json(
        repo,
        ArtifactSchema::EvidenceIndex,
        path.to_string_lossy().as_ref(),
        evidence,
    )?;
    Ok(())
}

fn render_markdown(plan: &ProofPlan) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Proof Plan");
    let _ = writeln!(out);
    let _ = writeln!(out, "- repo: `{}`", plan.repo_root);
    let _ = writeln!(out, "- git head: `{}`", plan.git_head);
    if let Some(base_ref) = &plan.base_ref {
        let _ = writeln!(out, "- base ref: `{}`", base_ref);
    }
    let _ = writeln!(out, "- changed: `{}`", join_or_none(&plan.changed_paths));
    let _ = writeln!(
        out,
        "- owner map: `{}`",
        join_or_none(&plan.matched_owner_map)
    );
    let _ = writeln!(
        out,
        "- test map: `{}`",
        join_or_none(&plan.matched_test_map)
    );
    let _ = writeln!(
        out,
        "- required lanes: `{}`",
        join_or_none(&plan.required_lanes)
    );
    let _ = writeln!(
        out,
        "- optional lanes: `{}`",
        join_or_none(&plan.optional_lanes)
    );
    let _ = writeln!(
        out,
        "- skipped lanes: `{}`",
        join_or_none(&plan.skipped_lanes)
    );
    if !plan.skipped_lane_entries.is_empty() {
        let _ = writeln!(out, "- skipped lane reasons:");
        for entry in &plan.skipped_lane_entries {
            let _ = writeln!(out, "  - `{}`: {}", entry.lane, entry.reason);
        }
    }
    let _ = writeln!(out, "- commands: `{}`", join_or_none(&plan.commands));
    let _ = writeln!(
        out,
        "- expected artifacts: `{}`",
        join_or_none(&plan.expected_artifacts)
    );
    let _ = writeln!(out, "- risk notes: `{}`", join_or_none(&plan.risk_notes));
    let _ = writeln!(
        out,
        "- human approval: `{}`",
        join_or_none(&plan.human_approval_requirements)
    );
    for run in &plan.planned_runs {
        let _ = writeln!(out);
        let _ = writeln!(out, "## {}", run.lane);
        let _ = writeln!(out, "- command: `{}`", run.command);
        let _ = writeln!(
            out,
            "- owner: `{}`",
            run.owner.as_deref().unwrap_or("unknown")
        );
        let _ = writeln!(out, "- changed: `{}`", join_or_none(&run.changed_paths));
        let _ = writeln!(out, "- artifacts: `{}`", join_or_none(&run.artifacts));
        let _ = writeln!(
            out,
            "- residual risk: `{}`",
            join_or_none(&run.residual_risk)
        );
        if let Some(reason) = &run.skipped_reason {
            let _ = writeln!(out, "- skipped reason: `{}`", reason);
        }
    }
    out
}

fn fallback_runs_from_plan(plan: &ProofPlan) -> Vec<PlannedRun> {
    plan.commands
        .iter()
        .enumerate()
        .map(|(index, command)| PlannedRun {
            lane: format!("command-{index}"),
            command: command.clone(),
            owner: None,
            changed_paths: plan.changed_paths.clone(),
            artifacts: vec![
                "target/jankurai/logs/*.log".to_string(),
                "target/jankurai/proof-receipts/*.json".to_string(),
            ],
            residual_risk: plan.risk_notes.clone(),
            skipped_reason: None,
        })
        .collect()
}

fn execute_run(
    repo: &Path,
    receipt_dir: &Path,
    log_dir: &Path,
    index: usize,
    run: &PlannedRun,
    plan_path: &str,
) -> Result<ProofReceipt> {
    let started = Instant::now();
    let started_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let run_id = proof_run_id(plan_path, index, &run.lane, &run.command, started_secs);
    let command_output = Command::new("bash")
        .arg("-lc")
        .arg(&run.command)
        .current_dir(repo)
        .output()
        .with_context(|| format!("run proof command `{}`", run.command))?;
    let log_file = log_dir.join(log_file_name(index, &run.lane, &run.command));
    let mut log_text = String::new();
    log_text.push_str(&format!("lane: {}\n", run.lane));
    log_text.push_str(&format!("command: {}\n", run.command));
    log_text.push_str(&format!("status: {}\n", command_output.status));
    log_text.push('\n');
    log_text.push_str(&String::from_utf8_lossy(&command_output.stdout));
    if !command_output.stdout.is_empty() && !command_output.stdout.ends_with(b"\n") {
        log_text.push('\n');
    }
    if !command_output.stderr.is_empty() {
        log_text.push_str("\n[stderr]\n");
        log_text.push_str(&String::from_utf8_lossy(&command_output.stderr));
        if !command_output.stderr.ends_with(b"\n") {
            log_text.push('\n');
        }
    }
    fs::write(&log_file, log_text)?;
    let exit_code = command_output.status.code().unwrap_or(-1);
    let stdout_stderr_bytes = fs::metadata(&log_file).map(|m| m.len()).ok();
    let retryable = if exit_code != 0 { Some(true) } else { None };
    Ok(ProofReceipt {
        lane: run.lane.clone(),
        command: run.command.clone(),
        exit_code,
        elapsed_ms: started.elapsed().as_millis(),
        artifacts: vec![display_relative(repo, &log_file)],
        changed_paths: run.changed_paths.clone(),
        owner: run.owner.clone(),
        skipped_reason: run.skipped_reason.clone(),
        residual_risk: run.residual_risk.clone(),
        log_path: Some(display_relative(repo, &log_file)),
        receipt_path: Some(display_relative(
            repo,
            &receipt_dir.join(receipt_file_name(index, &run.lane, &run.command)),
        )),
        generated_at: Some(now_string()),
        repo_root: Some(repo.display().to_string()),
        git_head: git_head(repo).ok(),
        run_id: Some(run_id),
        plan_path: Some(plan_path.to_string()),
        rules_covered: rules_covered_for_run(run),
        retryable,
        stdout_stderr_bytes,
    })
}

fn lane_names_by_command(catalog: &RepoCatalog) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for lane in &catalog.proof_lanes {
        map.entry(lane.command.clone())
            .or_insert_with(|| lane.name.clone());
    }
    map
}

fn normalize_changed_paths(
    repo: &Path,
    changed: &[PathBuf],
    changed_from: Option<&str>,
) -> Result<Vec<String>> {
    let mut paths = BTreeSet::new();
    for path in changed {
        if let Some(rel) = normalize_changed_path(repo, path) {
            insert_changed_path(&mut paths, rel, path)?;
        }
    }
    if let Some(base_ref) = changed_from {
        for path in crate::audit::changed_paths_from_git(repo, base_ref)? {
            if let Some(rel) = normalize_changed_path(repo, &path) {
                insert_changed_path(&mut paths, rel, path.as_path())?;
            }
        }
    }
    Ok(paths.into_iter().collect())
}

fn normalize_changed_path(root: &Path, path: &Path) -> Option<String> {
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    candidate
        .strip_prefix(root)
        .ok()
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
}

fn insert_changed_path(
    paths: &mut BTreeSet<String>,
    rel: String,
    original: &Path,
) -> Result<()> {
    let normalized = rel
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_string();
    if normalized.is_empty() || normalized == "." {
        anyhow::bail!(
            "changed path `{}` resolves to the repository root; pass explicit changed files \
             or a non-root subdirectory",
            original.display()
        );
    }
    paths.insert(normalized);
    Ok(())
}

fn rules_covered_for_run(run: &PlannedRun) -> Vec<RuleCoverage> {
    let mut rules = Vec::new();
    match run.lane.as_str() {
        "fast" | "audit" => {
            push_rule(&mut rules, "HLT-003-OWNERLESS-PATH");
            push_rule(&mut rules, "HLT-004-UNMAPPED-PROOF");
        }
        "contract" => {
            push_rule(&mut rules, "HLT-002-GENERATED-MUTATION");
            push_rule(&mut rules, "HLT-007-HANDWRITTEN-CONTRACT");
        }
        "db" => {
            push_rule(&mut rules, "HLT-006-DIRECT-DB-WRONG-LAYER");
            push_rule(&mut rules, "HLT-019-STREAMING-RUNTIME-DRIFT");
        }
        "db-migration-analyze" => {
            push_rule(&mut rules, "HLT-021-DESTRUCTIVE-MIGRATION");
        }
        "web" | "ux-qa" => {
            push_rule(&mut rules, "HLT-013-RENDERED-UX-GAP");
            push_rule(&mut rules, "HLT-014-A11Y-GAP");
        }
        "security" => {
            push_rule(&mut rules, "HLT-009-GENERATED-SECURITY");
            push_rule(&mut rules, "HLT-010-SECRET-SPRAWL");
            push_rule(&mut rules, "HLT-011-PROMPT-INJECTION");
            push_rule(&mut rules, "HLT-012-OVERBROAD-AGENCY");
            push_rule(&mut rules, "HLT-016-SUPPLY-CHAIN-DRIFT");
            push_rule(&mut rules, "HLT-020-CI-HARDENING-GAP");
        }
        "observability" => {
            push_rule(&mut rules, "HLT-017-OPAQUE-OBSERVABILITY");
        }
        _ => {}
    }
    rules
}

fn push_rule(rules: &mut Vec<RuleCoverage>, rule_id: &str) {
    if crate::audit::rules::lookup(rule_id).is_none() {
        return;
    }
    if rules.iter().any(|coverage| rule_coverage_id(coverage) == rule_id) {
        return;
    }
    rules.push(RuleCoverage::Rich {
        rule_id: rule_id.to_string(),
        status: "covered".to_string(),
    });
}

fn rule_coverage_id(coverage: &RuleCoverage) -> &str {
    match coverage {
        RuleCoverage::Rich { rule_id, .. } => rule_id.as_str(),
        RuleCoverage::Simple(rule_id) => rule_id.as_str(),
    }
}

fn receipt_file_name(index: usize, lane: &str, command: &str) -> String {
    format!(
        "{:02}-{}-{}.json",
        index + 1,
        slugify(lane),
        short_hash(command)
    )
}

fn log_file_name(index: usize, lane: &str, command: &str) -> String {
    format!(
        "{:02}-{}-{}.log",
        index + 1,
        slugify(lane),
        short_hash(command)
    )
}

fn slugify(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn short_hash(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest
        .iter()
        .take(6)
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

fn display_relative(repo: &Path, path: &Path) -> String {
    path.strip_prefix(repo)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn optional_repo_relative_existing(repo: &Path, rel_posix: &str) -> Option<String> {
    let path = repo.join(rel_posix);
    if path.is_file() {
        Some(display_relative(repo, path.as_path()))
    } else {
        None
    }
}

fn sha256_file_if_exists(repo: &Path, rel_posix: &str) -> Option<String> {
    let path = repo.join(rel_posix);
    let bytes = fs::read(&path).ok()?;
    Some(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn route_risk_note(path: &str) -> String {
    format!("route derived from `{path}`")
}

fn git_head(repo: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .with_context(|| format!("resolve git HEAD in {}", repo.display()))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        anyhow::bail!("unable to resolve git HEAD in {}", repo.display());
    }
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}
