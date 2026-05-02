use crate::commands::context_data::{push_unique, RepoCatalog};
use crate::validation::{self, ArtifactSchema};
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct RepairPlanArgs {
    pub repo: PathBuf,
    pub from: String,
    pub out: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct RepairPlan {
    pub schema_version: String,
    pub source_report: String,
    pub generated_at: String,
    pub target_stack_id: String,
    pub packets: Vec<RepairPacket>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_edits: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct RepairPacket {
    pub finding_fingerprint: String,
    pub finding_path: String,
    pub rule_id: String,
    pub check_id: String,
    pub severity: String,
    pub owner: String,
    pub lane: String,
    pub problem: String,
    pub why: String,
    pub permission_profile: String,
    pub allowed_paths: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub expected_patch_shape: String,
    pub required_proof: Vec<String>,
    pub stop_conditions: Vec<String>,
    pub human_review_required: bool,
    pub rollback_guidance: String,
}

pub fn run(args: RepairPlanArgs) -> Result<()> {
    let plan = build_repair_plan(&args.repo, &args.from)?;
    match args.out.as_deref() {
        Some(path) => {
            validation::write_json(&args.repo, ArtifactSchema::RepairPlan, path, &plan)?;
        }
        None => {
            validation::validate_serializable(&args.repo, ArtifactSchema::RepairPlan, &plan)?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
    }
    if let Some(path) = args.md.as_deref() {
        crate::render::write_markdown(path, &render_markdown(&plan))?;
    }
    Ok(())
}

pub fn build_repair_plan(repo: &Path, report_path: &str) -> Result<RepairPlan> {
    let catalog = RepoCatalog::load(repo)?;
    let text = fs::read_to_string(report_path)
        .with_context(|| format!("read repair source {}", report_path))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("parse repair source {}", report_path))?;
    let findings = value
        .get("findings")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let mut packets = Vec::new();
    for finding in findings {
        packets.push(packet_from_finding(&catalog, &finding));
    }
    Ok(RepairPlan {
        schema_version: "1.0.0".to_string(),
        source_report: report_path.to_string(),
        generated_at: now_string(),
        target_stack_id: crate::model::TARGET_STACK_ID.to_string(),
        packets,
        plan_mode: None,
        planned_edits: None,
    })
}

fn packet_from_finding(catalog: &RepoCatalog, finding: &serde_json::Value) -> RepairPacket {
    let finding_path = str_field(finding, "path");
    let rule_id = str_field(finding, "rule_id");
    let severity = str_field(finding, "severity");
    let owner = str_field(finding, "owner");
    let lane = str_field(finding, "lane");
    let problem = str_field(finding, "problem");
    let why = finding
        .get("reason")
        .and_then(|value| value.as_str())
        .unwrap_or(&problem)
        .to_string();
    let permission_profile = infer_permission_profile(&severity, &rule_id, &finding_path, &owner);
    let mut allowed_paths = repair_allowed_paths(catalog, &finding_path, &owner);
    if allowed_paths.is_empty() {
        push_unique(&mut allowed_paths, finding_path.clone());
    }
    let forbidden_paths = repair_forbidden_paths(catalog);
    let expected_patch_shape = expected_patch_shape(&rule_id, &severity, &finding_path);
    let mut required_proof = string_or_array_field(finding, "rerun_command");
    if required_proof.is_empty() {
        required_proof.extend(catalog.commands_for_paths(&allowed_paths));
    }
    if required_proof.is_empty() {
        required_proof.push("just fast".to_string());
    }
    let mut stop_conditions = vec![
        "stop if the fix broadens permission scope or touches a generated zone".to_string(),
        "stop if the repair requires a migration, secret rotation, or external service change"
            .to_string(),
    ];
    if permission_profile == "security-investigation" {
        push_unique(
            &mut stop_conditions,
            "stop and hand off any secret, credential, or token exposure",
        );
    }
    if permission_profile == "generated-regeneration" {
        push_unique(
            &mut stop_conditions,
            "stop if the source contract or generator is not identified first",
        );
    }
    let human_review_required =
        human_review_required(&severity, &rule_id, &finding_path, &permission_profile);
    let rollback_guidance = rollback_guidance(&permission_profile, &finding_path, &rule_id);
    RepairPacket {
        finding_fingerprint: str_field(finding, "fingerprint"),
        finding_path,
        rule_id,
        check_id: str_field(finding, "check_id"),
        severity,
        owner,
        lane,
        problem,
        why,
        permission_profile,
        allowed_paths,
        forbidden_paths,
        expected_patch_shape,
        required_proof,
        stop_conditions,
        human_review_required,
        rollback_guidance,
    }
}

fn render_markdown(plan: &RepairPlan) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Repair Plan");
    let _ = writeln!(out);
    let _ = writeln!(out, "- source report: `{}`", plan.source_report);
    let _ = writeln!(out, "- generated at: `{}`", plan.generated_at);
    let _ = writeln!(out, "- target stack: `{}`", plan.target_stack_id);
    for packet in &plan.packets {
        let _ = writeln!(out);
        let _ = writeln!(out, "## {} {}", packet.rule_id, packet.finding_path);
        let _ = writeln!(out, "- fingerprint: `{}`", packet.finding_fingerprint);
        let _ = writeln!(out, "- severity: `{}`", packet.severity);
        let _ = writeln!(out, "- owner: `{}`", packet.owner);
        let _ = writeln!(out, "- lane: `{}`", packet.lane);
        let _ = writeln!(out, "- profile: `{}`", packet.permission_profile);
        let _ = writeln!(out, "- problem: {}", packet.problem);
        let _ = writeln!(out, "- why: {}", packet.why);
        let _ = writeln!(out, "- allowed: `{}`", join_or_none(&packet.allowed_paths));
        let _ = writeln!(
            out,
            "- forbidden: `{}`",
            join_or_none(&packet.forbidden_paths)
        );
        let _ = writeln!(out, "- patch shape: {}", packet.expected_patch_shape);
        let _ = writeln!(out, "- proof: `{}`", join_or_none(&packet.required_proof));
        let _ = writeln!(out, "- stop: `{}`", join_or_none(&packet.stop_conditions));
        let _ = writeln!(out, "- human review: `{}`", packet.human_review_required);
        let _ = writeln!(out, "- rollback: {}", packet.rollback_guidance);
    }
    out
}

fn infer_permission_profile(severity: &str, rule_id: &str, path: &str, owner: &str) -> String {
    if matches!(
        rule_id,
        "HLT-010-SECRET-SPRAWL" | "HLT-011-PROMPT-INJECTION" | "HLT-012-OVERBROAD-AGENCY"
    ) || severity == "critical"
    {
        "security-investigation".to_string()
    } else if rule_id == "HLT-002-GENERATED-MUTATION" || path.starts_with("agent/") {
        "generated-regeneration".to_string()
    } else if path.starts_with("docs/") || path.starts_with("tips/") || owner == "paper" {
        "docs-only".to_string()
    } else if path.starts_with(".github/") || path == "Justfile" {
        "release".to_string()
    } else {
        "code-edit".to_string()
    }
}

fn repair_allowed_paths(catalog: &RepoCatalog, path: &str, owner: &str) -> Vec<String> {
    let mut out = Vec::new();
    for prefix in catalog.prefixes_for_owner(owner) {
        push_unique(&mut out, prefix);
    }
    if let Some(prefix) = parent_prefix(path) {
        push_unique(&mut out, prefix);
    }
    push_unique(&mut out, path);
    out
}

fn repair_forbidden_paths(catalog: &RepoCatalog) -> Vec<String> {
    let mut out = vec!["reference/".to_string(), "target/".to_string()];
    for path in catalog.forbidden_generated_paths() {
        push_unique(&mut out, path);
    }
    out
}

fn expected_patch_shape(rule_id: &str, severity: &str, path: &str) -> String {
    match rule_id {
        "HLT-010-SECRET-SPRAWL" => {
            "remove secrets, rotate credentials, and add secret scanning".to_string()
        }
        "HLT-011-PROMPT-INJECTION" => {
            "separate trusted policy from untrusted input and strip bypass language".to_string()
        }
        "HLT-012-OVERBROAD-AGENCY" => {
            "narrow the tool profile and route through least-privilege approval".to_string()
        }
        "HLT-002-GENERATED-MUTATION" => {
            "edit the source contract and regenerate the protected artifact".to_string()
        }
        "HLT-006-DIRECT-DB-WRONG-LAYER" => {
            "move SQL into the adapter or migration layer and keep domain code pure".to_string()
        }
        "HLT-007-HANDWRITTEN-CONTRACT" => {
            "replace handwritten mirrors with source contract or generated client output"
                .to_string()
        }
        "HLT-013-RENDERED-UX-GAP" => {
            "add rendered UX proof, state coverage, and accessibility receipts".to_string()
        }
        "HLT-017-OPAQUE-OBSERVABILITY" => {
            "add typed errors, traces, and repairable boundary evidence".to_string()
        }
        "HLT-019-STREAMING-RUNTIME-DRIFT" => {
            "move broker clients behind an adapter and keep contracts generated".to_string()
        }
        _ if path.starts_with("docs/") || path.starts_with("tips/") => {
            "make the document precise, short, and aligned with the canonical standard".to_string()
        }
        _ if severity == "critical" => {
            "narrow to the smallest safe fix and require human review".to_string()
        }
        _ => "scoped fix with targeted proof and no authority expansion".to_string(),
    }
}

fn human_review_required(
    severity: &str,
    rule_id: &str,
    path: &str,
    permission_profile: &str,
) -> bool {
    severity == "critical"
        || matches!(
            rule_id,
            "HLT-010-SECRET-SPRAWL"
                | "HLT-011-PROMPT-INJECTION"
                | "HLT-012-OVERBROAD-AGENCY"
                | "HLT-002-GENERATED-MUTATION"
                | "HLT-006-DIRECT-DB-WRONG-LAYER"
                | "HLT-007-HANDWRITTEN-CONTRACT"
                | "HLT-013-RENDERED-UX-GAP"
                | "HLT-019-STREAMING-RUNTIME-DRIFT"
        )
        || path.starts_with("reference/")
        || permission_profile == "security-investigation"
}

fn rollback_guidance(permission_profile: &str, path: &str, rule_id: &str) -> String {
    if rule_id == "HLT-002-GENERATED-MUTATION" || permission_profile == "generated-regeneration" {
        "revert the source contract or template, then regenerate the output".to_string()
    } else if permission_profile == "security-investigation" {
        "revert the scoped policy change and confirm secret scan evidence again".to_string()
    } else if path.starts_with("docs/") || path.starts_with("tips/") {
        "restore the previous text and rerun the narrow proof lane".to_string()
    } else {
        "revert the scoped files and rerun the required proof before retrying".to_string()
    }
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn parent_prefix(path: &str) -> Option<String> {
    path.rsplit_once('/')
        .map(|(prefix, _)| format!("{prefix}/"))
}

fn str_field(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string()
}

fn string_or_array_field(value: &serde_json::Value, key: &str) -> Vec<String> {
    if let Some(text) = value.get(key).and_then(|value| value.as_str()) {
        return vec![text.to_string()];
    }
    value
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(|item| item.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
