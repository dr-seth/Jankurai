use serde::Serialize;

pub const STANDARD_VERSION: &str = "0.2.0";
pub const AUDITOR_VERSION: &str = "0.2.0";
pub const SCHEMA_VERSION: &str = "1.0.0";
pub const PAPER_EDITION: &str = "2026.05-ed1";
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
pub struct UxQaReadiness {
    pub web_surface: bool,
    pub has_rendered_ux_lane: bool,
    pub missing_categories: Vec<String>,
    pub evidence: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub standard: String,
    pub standard_version: String,
    pub auditor_version: String,
    pub schema_version: String,
    pub paper_edition: String,
    pub target_stack_id: String,
    pub target_stack: String,
    pub repo: String,
    pub scope: Scope,
    pub score: i32,
    pub raw_score: i32,
    pub caps_applied: Vec<String>,
    pub hard_rules: Vec<HardRule>,
    pub dimensions: Vec<DimensionResult>,
    pub ux_qa: UxQaReadiness,
    pub findings: Vec<Finding>,
    pub agent_fix_queue: Vec<AgentFix>,
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
    pub rule_id: Option<String>,
    pub tlr: Option<String>,
    pub lane: Option<String>,
    pub owner: Option<String>,
    pub task: String,
    pub why: String,
}
