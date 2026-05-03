use crate::validation::{self, ArtifactSchema};
use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct MigrateArgs {
    pub repo: PathBuf,
    pub out: Option<String>,
    pub md: Option<String>,
    pub mode: MigrateMode,
}

#[derive(Debug, Clone)]
pub enum MigrateMode {
    Analyze,
    Plan,
}

// ---------------------------------------------------------------------------
// MigrationReport — matches schemas/migration-report.schema.json
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct MigrationReport {
    pub schema_version: String,
    pub command: String,
    pub status: String,
    pub generated_at: String,
    pub source_root: String,
    pub source_stack: String,
    pub target_stack: String,
    pub liability_score: u32,
    pub module_inventory: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_guesses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_boundaries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_surfaces: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_surfaces: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_logic: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high_risk_areas: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_tests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strangler_candidates: Option<Vec<String>>,
    pub recommended_slice_order: Vec<String>,
    pub required_proof_lanes: Vec<String>,
    pub rollback_cutover_notes: Vec<String>,
}

// ---------------------------------------------------------------------------
// MigrationPlan — matches schemas/migration-plan.schema.json
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct MigrationPlan {
    pub schema_version: String,
    pub command: String,
    pub status: String,
    pub generated_at: String,
    pub source_report: String,
    pub target_stack: String,
    pub plan_mode: String,
    pub slices: Vec<MigrationSlice>,
    pub human_approval_requirements: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationSlice {
    pub slice_id: String,
    pub owner: String,
    pub status: String,
    pub allowed_paths: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub contracts: Vec<String>,
    pub tests: Vec<String>,
    pub proof_lanes: Vec<String>,
    pub rollback_notes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cutover_notes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

// ---------------------------------------------------------------------------
// Stack Detection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
struct StackDetection {
    languages: Vec<String>,
    frameworks: Vec<String>,
    db_clients: Vec<String>,
    test_frameworks: Vec<String>,
    package_managers: Vec<String>,
    ci_systems: Vec<String>,
}

impl StackDetection {
    fn primary_language(&self) -> &str {
        self.languages
            .first()
            .map(|s| s.as_str())
            .unwrap_or("unknown")
    }

    fn summary(&self) -> String {
        let mut parts = vec![];
        if !self.languages.is_empty() {
            parts.push(self.languages.join("+"));
        }
        if !self.frameworks.is_empty() {
            parts.push(self.frameworks.join("+"));
        }
        if parts.is_empty() {
            "unknown".to_string()
        } else {
            parts.join("/")
        }
    }

    fn has_tests(&self) -> bool {
        !self.test_frameworks.is_empty()
    }

    fn has_ci(&self) -> bool {
        !self.ci_systems.is_empty()
    }
}

fn detect_stack(repo: &Path) -> StackDetection {
    let mut det = StackDetection::default();

    // Language + package manager detection from manifest files
    if repo.join("Cargo.toml").exists() {
        push_unique(&mut det.languages, "rust");
        push_unique(&mut det.package_managers, "cargo");
        push_unique(&mut det.test_frameworks, "cargo-test");

        // Framework detection from Cargo.toml content
        if let Ok(text) = fs::read_to_string(repo.join("Cargo.toml")) {
            let lower = text.to_ascii_lowercase();
            for fw in ["actix", "axum", "rocket", "warp"] {
                if lower.contains(fw) {
                    push_unique(&mut det.frameworks, fw);
                }
            }
        }
    }

    if repo.join("package.json").exists() {
        push_unique(&mut det.languages, "typescript");
        push_unique(&mut det.package_managers, "npm");

        if let Ok(text) = fs::read_to_string(repo.join("package.json")) {
            let lower = text.to_ascii_lowercase();
            for fw in [
                "express", "fastify", "next", "nuxt", "react", "vue", "angular", "svelte",
            ] {
                if lower.contains(fw) {
                    push_unique(&mut det.frameworks, fw);
                }
            }
            for tf in ["jest", "vitest", "mocha", "playwright", "cypress"] {
                if lower.contains(tf) {
                    push_unique(&mut det.test_frameworks, tf);
                }
            }
            for db in ["prisma", "knex", "typeorm", "sequelize", "drizzle"] {
                if lower.contains(db) {
                    push_unique(&mut det.db_clients, db);
                }
            }
        }
    }

    if repo.join("requirements.txt").exists() || repo.join("pyproject.toml").exists() {
        push_unique(&mut det.languages, "python");
        push_unique(&mut det.package_managers, "pip");
        push_unique(&mut det.test_frameworks, "pytest");

        for manifest in ["requirements.txt", "pyproject.toml"] {
            if let Ok(text) = fs::read_to_string(repo.join(manifest)) {
                let lower = text.to_ascii_lowercase();
                for fw in ["fastapi", "django", "flask"] {
                    if lower.contains(fw) {
                        push_unique(&mut det.frameworks, fw);
                    }
                }
                for db in ["psycopg", "sqlalchemy", "asyncpg"] {
                    if lower.contains(db) {
                        push_unique(&mut det.db_clients, db);
                    }
                }
            }
        }
    }

    if repo.join("pom.xml").exists() || repo.join("build.gradle").exists() {
        push_unique(&mut det.languages, "java");
        if repo.join("pom.xml").exists() {
            push_unique(&mut det.package_managers, "maven");
        }
        if repo.join("build.gradle").exists() {
            push_unique(&mut det.package_managers, "gradle");
        }
        push_unique(&mut det.test_frameworks, "junit");

        if let Ok(text) = fs::read_to_string(repo.join("pom.xml")) {
            if text.to_ascii_lowercase().contains("spring") {
                push_unique(&mut det.frameworks, "spring");
            }
        }
    }

    if repo.join("Gemfile").exists() {
        push_unique(&mut det.languages, "ruby");
        push_unique(&mut det.package_managers, "bundler");
        push_unique(&mut det.test_frameworks, "rspec");
        push_unique(&mut det.frameworks, "rails");
    }

    if repo.join("composer.json").exists() {
        push_unique(&mut det.languages, "php");
        push_unique(&mut det.package_managers, "composer");

        if let Ok(text) = fs::read_to_string(repo.join("composer.json")) {
            if text.to_ascii_lowercase().contains("laravel") {
                push_unique(&mut det.frameworks, "laravel");
            }
        }
    }

    if repo.join("go.mod").exists() {
        push_unique(&mut det.languages, "go");
        push_unique(&mut det.package_managers, "go-modules");
        push_unique(&mut det.test_frameworks, "go-test");
    }

    // CI detection
    if repo.join(".github/workflows").exists() {
        push_unique(&mut det.ci_systems, "github-actions");
    }
    if repo.join(".gitlab-ci.yml").exists() {
        push_unique(&mut det.ci_systems, "gitlab-ci");
    }
    if repo.join(".circleci").exists() {
        push_unique(&mut det.ci_systems, "circleci");
    }

    // DB client detection from Rust source
    if repo.join("Cargo.toml").exists() {
        if let Ok(text) = fs::read_to_string(repo.join("Cargo.toml")) {
            let lower = text.to_ascii_lowercase();
            for db in ["sqlx", "diesel", "sea-orm", "tokio-postgres"] {
                if lower.contains(db) {
                    push_unique(&mut det.db_clients, db);
                }
            }
        }
    }

    // Lockfile check — influences liability
    if !repo.join("Cargo.lock").exists()
        && !repo.join("package-lock.json").exists()
        && !repo.join("pnpm-lock.yaml").exists()
        && !repo.join("yarn.lock").exists()
        && !repo.join("Gemfile.lock").exists()
        && !repo.join("composer.lock").exists()
        && !repo.join("go.sum").exists()
    {
        // No lockfile — tracked by liability score, not stored in detection
    }

    det
}

fn has_lockfile(repo: &Path) -> bool {
    repo.join("Cargo.lock").exists()
        || repo.join("package-lock.json").exists()
        || repo.join("pnpm-lock.yaml").exists()
        || repo.join("yarn.lock").exists()
        || repo.join("Gemfile.lock").exists()
        || repo.join("composer.lock").exists()
        || repo.join("go.sum").exists()
}

fn push_unique(vec: &mut Vec<String>, val: &str) {
    if !vec.iter().any(|v| v == val) {
        vec.push(val.to_string());
    }
}

// ---------------------------------------------------------------------------
// Liability Score
// ---------------------------------------------------------------------------

fn compute_liability(repo: &Path, det: &StackDetection) -> u32 {
    let mut score: i32 = 50;

    if !has_lockfile(repo) {
        score += 10;
    }
    if !det.has_tests() {
        score += 10;
    }
    if !det.has_ci() {
        score += 5;
    }
    if det.db_clients.len() > 1 {
        score += 5;
    }
    if det.primary_language() == "rust" {
        score -= 10;
    }
    if det.has_tests() {
        score -= 5;
    }
    if det.languages.len() > 2 {
        score += 5; // polyglot complexity
    }

    score.clamp(0, 100) as u32
}

// ---------------------------------------------------------------------------
// Report Builder
// ---------------------------------------------------------------------------

pub fn build_migration_report(repo: &Path) -> Result<MigrationReport> {
    let det = detect_stack(repo);
    let liability = compute_liability(repo, &det);

    let mut module_inventory = vec![];
    for lang in &det.languages {
        module_inventory.push(format!("language:{lang}"));
    }
    for fw in &det.frameworks {
        module_inventory.push(format!("framework:{fw}"));
    }
    for pm in &det.package_managers {
        module_inventory.push(format!("package-manager:{pm}"));
    }

    let db_surfaces = if det.db_clients.is_empty() {
        None
    } else {
        Some(
            det.db_clients
                .iter()
                .map(|c| format!("db-client:{c}"))
                .collect(),
        )
    };

    let api_surfaces = if det.frameworks.is_empty() {
        None
    } else {
        Some(
            det.frameworks
                .iter()
                .map(|f| format!("api-framework:{f}"))
                .collect(),
        )
    };

    let strangler_candidates = if det.db_clients.is_empty() && det.frameworks.is_empty() {
        None
    } else {
        let mut candidates = vec![];
        for db in &det.db_clients {
            candidates.push(format!("isolate-db-layer:{db}"));
        }
        for fw in &det.frameworks {
            candidates.push(format!("isolate-api-surface:{fw}"));
        }
        Some(candidates)
    };

    let mut recommended_slice_order = vec![
        "inventory-and-classify".to_string(),
        "extract-contracts".to_string(),
    ];
    if !det.db_clients.is_empty() {
        recommended_slice_order.push("isolate-db-adapter-layer".to_string());
    }
    recommended_slice_order.push("port-business-logic".to_string());
    recommended_slice_order.push("prove-equivalence".to_string());
    recommended_slice_order.push("cutover-and-retire".to_string());

    let required_proof_lanes = vec!["fast".to_string(), "contract".to_string()];

    let missing_tests = if !det.has_tests() {
        Some(vec![
            "no test framework detected — migration risk is elevated".to_string(),
        ])
    } else {
        None
    };

    let high_risk_areas = if !det.has_ci() {
        Some(vec![
            "no CI system detected — migration cannot be verified automatically".to_string(),
        ])
    } else {
        None
    };

    Ok(MigrationReport {
        schema_version: "1.0.0".to_string(),
        command: "jankurai migrate".to_string(),
        status: "complete".to_string(),
        generated_at: now_string(),
        source_root: repo.display().to_string(),
        source_stack: det.summary(),
        target_stack: "rust-ts-postgres".to_string(),
        liability_score: liability,
        module_inventory,
        owner_guesses: None,
        external_boundaries: None,
        db_surfaces,
        api_surfaces,
        duplicate_logic: None,
        high_risk_areas,
        missing_tests,
        strangler_candidates,
        recommended_slice_order,
        required_proof_lanes,
        rollback_cutover_notes: vec![
            "migration planning is derived from repo inventory and audit evidence".to_string(),
            "each slice should have explicit rollback before cutover".to_string(),
        ],
    })
}

// ---------------------------------------------------------------------------
// Plan Builder
// ---------------------------------------------------------------------------

pub fn build_migration_plan(repo: &Path) -> Result<MigrationPlan> {
    let report = build_migration_report(repo)?;

    let mut slices = vec![];

    // Generate one slice per DB surface
    if let Some(ref db_surfaces) = report.db_surfaces {
        for (i, db) in db_surfaces.iter().enumerate() {
            slices.push(MigrationSlice {
                slice_id: format!("db-isolation-{}", i + 1),
                owner: "tools".to_string(),
                status: "candidate".to_string(),
                allowed_paths: vec![format!("crates/adapters/db/{}", db.replace("db-client:", ""))],
                forbidden_paths: vec!["crates/domain/".to_string()],
                contracts: vec!["adapter boundary interface".to_string()],
                tests: vec!["adapter integration test".to_string()],
                proof_lanes: vec!["db".to_string(), "fast".to_string()],
                rollback_notes: vec!["revert adapter extraction if contract tests fail".to_string()],
                cutover_notes: None,
                notes: Some(format!("isolate {db} behind adapter boundary")),
            });
        }
    }

    // Generate one slice per API surface
    if let Some(ref api_surfaces) = report.api_surfaces {
        for (i, api) in api_surfaces.iter().enumerate() {
            slices.push(MigrationSlice {
                slice_id: format!("api-contract-{}", i + 1),
                owner: "tools".to_string(),
                status: "candidate".to_string(),
                allowed_paths: vec!["contracts/".to_string()],
                forbidden_paths: vec![],
                contracts: vec!["OpenAPI or JSON Schema contract".to_string()],
                tests: vec!["consumer/provider contract test".to_string()],
                proof_lanes: vec!["contract".to_string(), "fast".to_string()],
                rollback_notes: vec![
                    "revert contract extraction if provider tests fail".to_string()
                ],
                cutover_notes: None,
                notes: Some(format!("extract contract for {api}")),
            });
        }
    }

    // Always add a final equivalence proof slice
    slices.push(MigrationSlice {
        slice_id: "equivalence-proof".to_string(),
        owner: "tools".to_string(),
        status: if slices.is_empty() {
            "blocked".to_string()
        } else {
            "candidate".to_string()
        },
        allowed_paths: vec!["tests/equivalence/".to_string()],
        forbidden_paths: vec![],
        contracts: vec!["golden input/output equivalence".to_string()],
        tests: vec!["equivalence comparison test".to_string()],
        proof_lanes: vec!["fast".to_string()],
        rollback_notes: vec!["equivalence failures block cutover".to_string()],
        cutover_notes: Some(vec![
            "shadow reads recommended before cutover".to_string(),
            "parallel-run comparison for critical paths".to_string(),
        ]),
        notes: Some("prove equivalent behavior before retiring old code".to_string()),
    });

    let mut human_approvals = vec!["high-risk cutovers require human review".to_string()];
    if report.liability_score > 70 {
        human_approvals
            .push("liability score above 70 — all slices require human approval".to_string());
    }

    Ok(MigrationPlan {
        schema_version: "1.0.0".to_string(),
        command: "jankurai migrate".to_string(),
        status: "complete".to_string(),
        generated_at: now_string(),
        source_report: format!("target/jankurai/migration-report.json"),
        target_stack: report.target_stack.clone(),
        plan_mode: "dry-run".to_string(),
        slices,
        human_approval_requirements: human_approvals,
        commands: Some(vec![
            "jankurai migrate analyze . --json target/jankurai/migration-report.json".to_string(),
            "jankurai migrate plan . --json target/jankurai/migration-plan.json".to_string(),
        ]),
        warnings: if report.liability_score > 60 {
            Some(vec![format!(
                "liability score {} indicates elevated migration risk",
                report.liability_score
            )])
        } else {
            None
        },
    })
}

// ---------------------------------------------------------------------------
// CLI Entry Points
// ---------------------------------------------------------------------------

pub fn run(args: MigrateArgs) -> Result<()> {
    match args.mode {
        MigrateMode::Analyze => run_analyze(&args.repo, args.out.as_deref(), args.md.as_deref()),
        MigrateMode::Plan => run_plan(&args.repo, args.out.as_deref(), args.md.as_deref()),
    }
}

fn run_analyze(repo: &Path, out: Option<&str>, md: Option<&str>) -> Result<()> {
    let report = build_migration_report(repo)?;
    if let Some(path) = out {
        validation::write_json(repo, ArtifactSchema::MigrationReport, path, &report)?;
    } else {
        println!("{}", serde_json::to_string_pretty(&report)?);
    }
    if let Some(path) = md {
        crate::render::write_markdown(path, &render_report_markdown(&report))?;
    }
    Ok(())
}

fn run_plan(repo: &Path, out: Option<&str>, md: Option<&str>) -> Result<()> {
    let plan = build_migration_plan(repo)?;
    if let Some(path) = out {
        validation::write_json(repo, ArtifactSchema::MigrationPlan, path, &plan)?;
    } else {
        println!("{}", serde_json::to_string_pretty(&plan)?);
    }
    if let Some(path) = md {
        crate::render::write_markdown(path, &render_plan_markdown(&plan))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Markdown Rendering
// ---------------------------------------------------------------------------

fn render_report_markdown(report: &MigrationReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Migration Report");
    let _ = writeln!(out);
    let _ = writeln!(out, "- source stack: `{}`", report.source_stack);
    let _ = writeln!(out, "- target stack: `{}`", report.target_stack);
    let _ = writeln!(out, "- liability score: `{}`", report.liability_score);
    let _ = writeln!(
        out,
        "- module inventory: `{}`",
        report.module_inventory.join(", ")
    );
    if let Some(ref db) = report.db_surfaces {
        let _ = writeln!(out, "- DB surfaces: `{}`", db.join(", "));
    }
    if let Some(ref api) = report.api_surfaces {
        let _ = writeln!(out, "- API surfaces: `{}`", api.join(", "));
    }
    if let Some(ref strangler) = report.strangler_candidates {
        let _ = writeln!(out, "- strangler candidates: `{}`", strangler.join(", "));
    }
    let _ = writeln!(
        out,
        "- recommended slice order: `{}`",
        report.recommended_slice_order.join(" → ")
    );
    let _ = writeln!(
        out,
        "- required proof lanes: `{}`",
        report.required_proof_lanes.join(", ")
    );
    let _ = writeln!(
        out,
        "- rollback notes: `{}`",
        report.rollback_cutover_notes.join("; ")
    );
    out
}

fn render_plan_markdown(plan: &MigrationPlan) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai Migration Plan");
    let _ = writeln!(out);
    let _ = writeln!(out, "- target stack: `{}`", plan.target_stack);
    let _ = writeln!(out, "- plan mode: `{}`", plan.plan_mode);
    let _ = writeln!(out, "- slices: `{}`", plan.slices.len());
    let _ = writeln!(out);
    for slice in &plan.slices {
        let _ = writeln!(out, "## Slice: {}", slice.slice_id);
        let _ = writeln!(out, "- owner: `{}`", slice.owner);
        let _ = writeln!(out, "- status: `{}`", slice.status);
        let _ = writeln!(out, "- proof lanes: `{}`", slice.proof_lanes.join(", "));
        if let Some(ref notes) = slice.notes {
            let _ = writeln!(out, "- notes: {notes}");
        }
        let _ = writeln!(out);
    }
    let _ = writeln!(out, "## Human Approval Requirements");
    for req in &plan.human_approval_requirements {
        let _ = writeln!(out, "- {req}");
    }
    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
