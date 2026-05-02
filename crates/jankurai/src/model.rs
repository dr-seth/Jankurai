use serde::Serialize;
use std::collections::BTreeMap;

pub const STANDARD_VERSION: &str = "0.4.0";
pub const AUDITOR_VERSION: &str = "0.4.0";
pub const SCHEMA_VERSION: &str = "1.2.0";
pub const PAPER_EDITION: &str = "2026.05-ed3";
pub const TARGET_STACK_ID: &str = "rust-ts-vite-react-postgres-bounded-python";
pub const TARGET_STACK: &str = "Rust core + TypeScript/React/Vite + PostgreSQL + generated contracts + bounded Python AI/data service";

#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    pub rel_path: String,
    pub name: String,
    pub suffix: String,
    pub size: u64,
    pub line_count: usize,
    pub text: String,
    pub is_generated: bool,
    pub is_code: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DimensionResult {
    pub name: String,
    pub weight: u32,
    pub score: i32,
    pub weighted_points: f64,
    pub evidence: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub severity: String,
    pub category: String,
    pub path: String,
    pub problem: String,
    pub agent_fix: String,
    pub evidence: Vec<String>,
    pub check_id: String,
    pub hardness: String,
    pub confidence: f64,
    pub evidence_kind: String,
    pub rerun_command: String,
    pub fingerprint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tlr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lane: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportDecision {
    pub status: String,
    pub minimum_score: i32,
    pub passed: bool,
    pub hard_findings: usize,
    pub soft_findings: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratchet: Option<ReportRatchet>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportRatchet {
    pub baseline_score: i32,
    pub allowed_drop: i32,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitSummary {
    pub head: Option<String>,
    pub base: Option<String>,
    pub changed_files: usize,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dirty_worktree: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct PolicySummary {
    pub path: String,
    pub minimum_score: i32,
    pub fail_on: Vec<String>,
    pub advisory_on: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auditor_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper_edition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_stack: Option<String>,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RuleCoverage {
    Rich { rule_id: String, status: String },
    Simple(String),
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ProofReceipt {
    pub lane: String,
    pub command: String,
    pub exit_code: i32,
    pub elapsed_ms: u128,
    pub artifacts: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub changed_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_reason: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub residual_risk: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_head: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_path: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules_covered: Vec<RuleCoverage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout_stderr_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UxQaReportArtifactSummary {
    /// Repo-relative path (POSIX slashes).
    pub path: String,
    pub report_count: usize,
    /// Worst decision across reports: `block` > `review` > `warn` > `pass`.
    pub worst_decision: String,
    pub total_violations: usize,
    /// Sum of per-report `summary.errors`.
    pub summary_errors: u64,
    /// Sum of per-report `summary.warnings`.
    pub summary_warnings: u64,
    pub reports_missing_required_states: usize,
    pub missing_state_names: Vec<String>,
    pub artifact_counts_by_kind: BTreeMap<String, usize>,
    pub reports_missing_required_artifacts: usize,
    pub missing_artifact_kinds: Vec<String>,
    pub reports_missing_required_accessibility_artifact: usize,
    pub accessibility_violation_total: u64,
    pub accessibility_incomplete_total: u64,
    pub accessibility_pass_total: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UxQaReadiness {
    pub web_surface: bool,
    pub has_rendered_ux_lane: bool,
    pub missing_categories: Vec<String>,
    pub evidence: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<UxQaReportArtifactSummary>,
}

/// Compact summary of validated `target/jankurai/security/evidence.json` for repo-score (audit-only).
#[derive(Debug, Clone, Serialize)]
pub struct SecurityEvidenceArtifactSummary {
    /// Repo-relative path (POSIX slashes).
    pub path: String,
    pub envelope_exit_code: i32,
    pub elapsed_ms: u64,
    pub wrapper_strict: bool,
    pub commands_ran: usize,
    pub commands_skipped: usize,
    pub commands_failed: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_head: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SecurityEvidenceReadiness {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<SecurityEvidenceArtifactSummary>,
}

/// Compact summary of validated `agent/boundaries.toml` for repo-score (audit-only).
#[derive(Debug, Clone, Serialize)]
pub struct BoundariesManifestSummary {
    /// Repo-relative path (POSIX slashes).
    pub path: String,
    /// SHA-256 fingerprint of raw manifest bytes (same convention as `manifest_fingerprints`).
    pub content_fingerprint: String,
    pub stack_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_version: Option<String>,
    pub adapter_path_count: usize,
    pub event_contract_path_count: usize,
    pub generated_type_path_count: usize,
    pub client_marker_count: usize,
    pub streaming_exception_count: usize,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BoundariesReadiness {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<BoundariesManifestSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub report_fingerprint: String,
    pub input_fingerprint: String,
    pub policy_fingerprint: String,
    pub manifest_fingerprints: ManifestFingerprints,
    pub dirty_worktree: bool,
    pub generated_at: String,
    pub schema_url: String,
    pub standard: String,
    pub standard_version: String,
    pub auditor_version: String,
    pub schema_version: String,
    pub paper_edition: String,
    pub target_stack_id: String,
    pub target_stack: String,
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u128>,
    pub scope: Scope,
    pub score: i32,
    pub raw_score: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<ReportDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<GitSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<PolicySummary>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub proof_receipts: Vec<ProofReceipt>,
    pub caps_applied: Vec<String>,
    pub hard_rules: Vec<HardRule>,
    pub dimensions: Vec<DimensionResult>,
    pub ux_qa: UxQaReadiness,
    pub security_evidence: SecurityEvidenceReadiness,
    pub boundaries: BoundariesReadiness,
    pub findings: Vec<Finding>,
    pub agent_fix_queue: Vec<AgentFix>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ManifestFingerprints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_map: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_map: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_zones: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundaries: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_lanes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Scope {
    pub mode: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HardRule {
    pub id: String,
    pub max_score: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentFix {
    pub path: String,
    pub priority: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tlr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lane: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub task: String,
    pub why: String,
}
